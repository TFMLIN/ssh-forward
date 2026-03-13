use anyhow::{anyhow, Context, Result};
use russh::client::{self, Handle};
use russh_keys::key::PrivateKeyWithHashAlg;
use std::sync::Arc;

use crate::types::{AuthType, JumpHost, SshServerConfig};

/// russh 客户端事件处理器
pub struct SshClient;

#[async_trait::async_trait]
impl client::Handler for SshClient {
    type Error = anyhow::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &russh_keys::PublicKey,
    ) -> Result<bool, Self::Error> {
        // TODO: 生产环境中应验证主机密钥指纹
        // 当前版本接受所有主机密钥（TOFU 模式）
        Ok(true)
    }
}

/// 建立 SSH 连接（支持跳板机链）
/// 若 config.jump_hosts 不为空，则依次通过跳板机建立隧道后连接目标
pub async fn connect(config: &SshServerConfig) -> Result<Handle<SshClient>> {
    let jumps = config.jump_hosts.as_deref().unwrap_or(&[]);
    if jumps.is_empty() {
        connect_direct(&config.host, config.port, config).await
    } else {
        connect_via_jumps(jumps, &config.host, config.port, config).await
    }
}

/// 直连目标服务器（无跳板）
async fn connect_direct(
    host: &str,
    port: u16,
    config: &SshServerConfig,
) -> Result<Handle<SshClient>> {
    let ssh_config = make_ssh_config();
    let addr = format!("{}:{}", host, port);
    let mut session = client::connect(ssh_config, addr, SshClient)
        .await
        .context(format!("无法连接到 {}:{}", host, port))?;
    authenticate_server(&mut session, config).await?;
    Ok(session)
}

/// 通过跳板机链连接目标服务器（迭代方式，避免递归 async 的 boxing 问题）
/// jumps: 有序跳板链（第一个是最近的跳板机）
/// final_host/final_port: 目标服务器地址
/// final_config: 目标服务器的认证信息
async fn connect_via_jumps(
    jumps: &[JumpHost],
    final_host: &str,
    final_port: u16,
    final_config: &SshServerConfig,
) -> Result<Handle<SshClient>> {
    let ssh_config = make_ssh_config();

    // 第一步：连接第一个跳板机（直接 TCP 连接）
    let first_jump = &jumps[0];
    let first_addr = format!("{}:{}", first_jump.host, first_jump.port);
    let mut current_session = client::connect(ssh_config.clone(), first_addr, SshClient)
        .await
        .context(format!("无法连接到跳板机 {}:{}", first_jump.host, first_jump.port))?;
    authenticate_jump(&mut current_session, first_jump).await?;

    // 后续跳板机：通过当前会话打开通道，在通道上建立下一跳的 SSH 连接
    for i in 1..jumps.len() {
        let next_jump = &jumps[i];
        let channel = current_session
            .channel_open_direct_tcpip(&next_jump.host, next_jump.port as u32, "127.0.0.1", 0)
            .await
            .context(format!("无法打开到跳板机 {}:{} 的通道", next_jump.host, next_jump.port))?;

        let stream = bridge_channel_to_duplex(channel).await?;
        let mut next_session = client::connect_stream(ssh_config.clone(), stream, SshClient)
            .await
            .context(format!("无法在隧道上连接跳板机 {}:{}", next_jump.host, next_jump.port))?;
        authenticate_jump(&mut next_session, next_jump).await?;
        current_session = next_session;
    }

    // 最终：通过最后一个跳板机连接目标服务器
    let final_channel = current_session
        .channel_open_direct_tcpip(final_host, final_port as u32, "127.0.0.1", 0)
        .await
        .context(format!("无法打开到目标服务器 {}:{} 的通道", final_host, final_port))?;

    let final_stream = bridge_channel_to_duplex(final_channel).await?;
    let mut final_session = client::connect_stream(ssh_config, final_stream, SshClient)
        .await
        .context(format!("无法通过隧道连接到目标 {}:{}", final_host, final_port))?;
    authenticate_server(&mut final_session, final_config).await?;

    Ok(final_session)
}

/// 将 russh Channel 桥接到 tokio DuplexStream（Unpin + Send + 'static）
/// 在后台 task 中双向转发数据
async fn bridge_channel_to_duplex(
    channel: russh::Channel<russh::client::Msg>,
) -> Result<tokio::io::DuplexStream> {
    // 创建双向管道，缓冲区 256KB
    let (local_end, remote_end) = tokio::io::duplex(256 * 1024);

    tokio::spawn(async move {
        // 将 ChannelStream（不是 Unpin）Pin 住后转发
        let channel_stream = channel.into_stream();
        tokio::pin!(channel_stream);
        let (mut ch_read, mut ch_write) = tokio::io::split(channel_stream);
        let (mut pipe_read, mut pipe_write) = tokio::io::split(remote_end);

        tokio::select! {
            _ = tokio::io::copy(&mut ch_read, &mut pipe_write) => {}
            _ = tokio::io::copy(&mut pipe_read, &mut ch_write) => {}
        }
    });

    Ok(local_end)
}

/// 创建 SSH 客户端配置
fn make_ssh_config() -> Arc<client::Config> {
    Arc::new(client::Config {
        inactivity_timeout: Some(std::time::Duration::from_secs(3600)),
        keepalive_interval: Some(std::time::Duration::from_secs(30)),
        keepalive_max: 3,
        ..Default::default()
    })
}

/// 对目标服务器执行认证
async fn authenticate_server(
    session: &mut Handle<SshClient>,
    config: &SshServerConfig,
) -> Result<()> {
    let ok = match &config.auth_type {
        AuthType::Password => {
            let password = config
                .password
                .as_deref()
                .ok_or_else(|| anyhow!("密码认证需要提供密码"))?;
            session
                .authenticate_password(&config.username, password)
                .await
                .context("密码认证失败")?
        }

        AuthType::PrivateKey => {
            let key_path = config
                .private_key_path
                .as_deref()
                .ok_or_else(|| anyhow!("私钥认证需要提供私钥路径"))?;
            let expanded = expand_tilde(key_path);
            let key_pair = russh_keys::load_secret_key(&expanded, config.passphrase.as_deref())
                .context("无法加载私钥文件")?;
            let key_with_hash = PrivateKeyWithHashAlg::new(Arc::new(key_pair), None)
                .context("无法创建带哈希算法的私钥")?;
            session
                .authenticate_publickey(&config.username, key_with_hash)
                .await
                .context("公钥认证失败")?
        }

        AuthType::Agent => authenticate_with_agent(session, &config.username).await?,
    };

    if !ok {
        return Err(anyhow!("认证失败，用户名：'{}'", config.username));
    }
    Ok(())
}

/// 对跳板机执行认证
async fn authenticate_jump(
    session: &mut Handle<SshClient>,
    jump: &JumpHost,
) -> Result<()> {
    let ok = match &jump.auth_type {
        AuthType::Password => {
            let password = jump
                .password
                .as_deref()
                .ok_or_else(|| anyhow!("跳板机 {} 密码认证需要提供密码", jump.host))?;
            session
                .authenticate_password(&jump.username, password)
                .await
                .context(format!("跳板机 {} 密码认证失败", jump.host))?
        }

        AuthType::PrivateKey => {
            let key_path = jump
                .private_key_path
                .as_deref()
                .ok_or_else(|| anyhow!("跳板机 {} 私钥认证需要提供私钥路径", jump.host))?;
            let expanded = expand_tilde(key_path);
            let key_pair = russh_keys::load_secret_key(&expanded, jump.passphrase.as_deref())
                .context(format!("无法加载跳板机 {} 的私钥", jump.host))?;
            let key_with_hash = PrivateKeyWithHashAlg::new(Arc::new(key_pair), None)
                .context("无法创建带哈希算法的私钥")?;
            session
                .authenticate_publickey(&jump.username, key_with_hash)
                .await
                .context(format!("跳板机 {} 公钥认证失败", jump.host))?
        }

        AuthType::Agent => authenticate_with_agent(session, &jump.username).await?,
    };

    if !ok {
        return Err(anyhow!("跳板机 {} 认证失败，用户名：'{}'", jump.host, jump.username));
    }
    Ok(())
}

/// 通过 SSH Agent 认证
async fn authenticate_with_agent(session: &mut Handle<SshClient>, username: &str) -> Result<bool> {
    let agent_socket = std::env::var("SSH_AUTH_SOCK")
        .map_err(|_| anyhow!("SSH_AUTH_SOCK 未设置，请确认 ssh-agent 正在运行"))?;

    let mut agent = russh_keys::agent::client::AgentClient::connect_uds(&agent_socket)
        .await
        .context("无法连接到 ssh-agent")?;

    let identities = agent
        .request_identities()
        .await
        .context("无法从 ssh-agent 获取密钥列表")?;

    for key in identities {
        let result = session
            .authenticate_publickey_with(username, key, &mut agent)
            .await;
        if let Ok(true) = result {
            return Ok(true);
        }
    }

    Ok(false)
}

/// 展开路径中的 ~ 为 home 目录
fn expand_tilde(path: &str) -> std::path::PathBuf {
    if path.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(&path[2..]);
        }
    }
    std::path::PathBuf::from(path)
}
