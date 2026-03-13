use anyhow::{Context, Result};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;

use crate::ssh::client::connect;
use crate::types::{ForwardRule, SshServerConfig};

/// 启动端口转发。
/// 返回一个 Sender，调用 `send(())` 可停止转发。
pub async fn start_forward(
    server_config: SshServerConfig,
    rule: ForwardRule,
    status_tx: tokio::sync::watch::Sender<String>,
) -> Result<oneshot::Sender<()>> {
    let local_addr = format!("0.0.0.0:{}", rule.local_port);
    let listener = TcpListener::bind(&local_addr)
        .await
        .context(format!("无法绑定本地端口 {}", rule.local_port))?;

    let (stop_tx, stop_rx) = oneshot::channel::<()>();

    let _ = status_tx.send("active".to_string());

    tokio::spawn(accept_loop(listener, server_config, rule, stop_rx, status_tx));

    Ok(stop_tx)
}

/// 主循环：持续接受本地 TCP 连接
async fn accept_loop(
    listener: TcpListener,
    server_config: SshServerConfig,
    rule: ForwardRule,
    mut stop_rx: oneshot::Receiver<()>,
    status_tx: tokio::sync::watch::Sender<String>,
) {
    loop {
        tokio::select! {
            _ = &mut stop_rx => {
                log::info!("规则 {} 的转发已停止", rule.id);
                let _ = status_tx.send("stopped".to_string());
                return;
            }
            accept_result = listener.accept() => {
                match accept_result {
                    Ok((stream, addr)) => {
                        log::info!("接受来自 {} 的连接，转发规则：{}", addr, rule.id);
                        let cfg = server_config.clone();
                        let r = rule.clone();
                        tokio::spawn(async move {
                            if let Err(e) = handle_single_connection(stream, cfg, r).await {
                                log::warn!("转发连接错误：{}", e);
                            }
                        });
                    }
                    Err(e) => {
                        log::error!("Accept 错误：{}", e);
                        let _ = status_tx.send(format!("error:{}", e));
                        return;
                    }
                }
            }
        }
    }
}

/// 处理单个 TCP 连接：SSH direct-tcpip 通道 + 双向数据转发
async fn handle_single_connection(
    local_stream: TcpStream,
    server_config: SshServerConfig,
    rule: ForwardRule,
) -> Result<()> {
    // 建立 SSH 连接
    let session = connect(&server_config)
        .await
        .context("无法建立 SSH 连接")?;

    // 打开 direct-tcpip 通道（本地端口转发）
    let mut channel = session
        .channel_open_direct_tcpip(
            &rule.remote_host,
            rule.remote_port as u32,
            "127.0.0.1",
            0,
        )
        .await
        .context("无法打开 SSH 隧道通道")?;

    // 将本地 TCP 流拆分
    let (mut local_read, mut local_write) = tokio::io::split(local_stream);

    // make_writer 只需要 &self，先创建；make_reader 需要 &mut self，后创建
    let mut ssh_tx = channel.make_writer();
    let mut ssh_rx = channel.make_reader();

    // 并发双向转发，任意一端断开就结束
    tokio::select! {
        r = tokio::io::copy(&mut local_read, &mut ssh_tx) => {
            log::debug!("本地→远端连接关闭：{:?}", r);
        }
        r = tokio::io::copy(&mut ssh_rx, &mut local_write) => {
            log::debug!("远端→本地连接关闭：{:?}", r);
        }
    }

    Ok(())
}
