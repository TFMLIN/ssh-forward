use std::collections::HashMap;
use std::sync::Arc;

use tauri::State;
use tokio::sync::{oneshot, watch, Mutex};
use uuid::Uuid;

use crate::ssh::client::connect;
use crate::ssh::forwarder::start_forward;
use crate::types::{ForwardRule, ImportedJumpHost, ImportedServer, SshServerConfig};

/// 单个转发会话的运行时数据
pub struct ForwardSession {
    pub stop_tx: Option<oneshot::Sender<()>>,
    pub status_rx: watch::Receiver<String>,
}

/// 应用全局状态
pub struct AppState {
    pub sessions: Mutex<HashMap<String, ForwardSession>>,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            sessions: Mutex::new(HashMap::new()),
        }
    }
}

/// 测试 SSH 连接
#[tauri::command]
pub async fn test_connection(server: SshServerConfig) -> Result<(), String> {
    connect(&server).await.map_err(|e| e.to_string())?;
    Ok(())
}

/// 启动端口转发，返回 session_id
#[tauri::command]
pub async fn start_forward_cmd(
    state: State<'_, Arc<AppState>>,
    server: SshServerConfig,
    rule: ForwardRule,
) -> Result<String, String> {
    let session_id = Uuid::new_v4().to_string();

    let (status_tx, status_rx) = watch::channel("connecting".to_string());

    let stop_tx = start_forward(server, rule, status_tx)
        .await
        .map_err(|e| e.to_string())?;

    let session = ForwardSession {
        stop_tx: Some(stop_tx),
        status_rx,
    };

    state
        .sessions
        .lock()
        .await
        .insert(session_id.clone(), session);

    Ok(session_id)
}

/// 停止指定的端口转发
#[tauri::command]
pub async fn stop_forward_cmd(
    state: State<'_, Arc<AppState>>,
    session_id: String,
) -> Result<(), String> {
    let mut sessions = state.sessions.lock().await;
    if let Some(mut session) = sessions.remove(&session_id) {
        if let Some(stop_tx) = session.stop_tx.take() {
            let _ = stop_tx.send(());
        }
    }
    Ok(())
}

/// 获取转发会话状态
#[tauri::command]
pub async fn get_forward_status(
    state: State<'_, Arc<AppState>>,
    session_id: String,
) -> Result<String, String> {
    let sessions = state.sessions.lock().await;
    if let Some(session) = sessions.get(&session_id) {
        Ok(session.status_rx.borrow().clone())
    } else {
        Ok("stopped".to_string())
    }
}

/// 获取所有活跃会话的状态
#[tauri::command]
pub async fn get_all_statuses(
    state: State<'_, Arc<AppState>>,
) -> Result<HashMap<String, String>, String> {
    let sessions = state.sessions.lock().await;
    let statuses = sessions
        .iter()
        .map(|(id, session)| (id.clone(), session.status_rx.borrow().clone()))
        .collect();
    Ok(statuses)
}

/// 解析 ProxyJump 值中的单个跳板机（格式: [user@]host[:port]）
fn parse_jump_destination(dest: &str) -> (String, u16, Option<String>) {
    let dest = dest.trim();
    let (user, host_port) = if let Some(at_pos) = dest.find('@') {
        (Some(dest[..at_pos].to_string()), &dest[at_pos + 1..])
    } else {
        (None, dest)
    };

    // 处理 [IPv6]:port 格式
    let (host, port) = if host_port.starts_with('[') {
        if let Some(bracket_end) = host_port.find(']') {
            let h = &host_port[1..bracket_end];
            let rest = &host_port[bracket_end + 1..];
            if let Some(port_str) = rest.strip_prefix(':') {
                (h.to_string(), port_str.parse::<u16>().unwrap_or(22))
            } else {
                (h.to_string(), 22)
            }
        } else {
            (host_port.to_string(), 22)
        }
    } else if let Some(colon_pos) = host_port.rfind(':') {
        let h = &host_port[..colon_pos];
        let p = host_port[colon_pos + 1..].parse::<u16>().unwrap_or(22);
        (h.to_string(), p)
    } else {
        (host_port.to_string(), 22)
    };

    (host, port, user)
}

/// 将 ProxyJump 值中引用的 alias 递归解析为 ImportedJumpHost 列表。
/// 支持逗号分隔的多级跳板机和对其他 Host 别名的引用。
fn resolve_proxy_jump(
    proxy_jump_value: &str,
    ssh_config: &ssh2_config::SshConfig,
    proxy_jump_map: &std::collections::HashMap<String, String>,
    depth: usize,
) -> Vec<ImportedJumpHost> {
    if depth > 10 {
        return vec![]; // 防止无限递归
    }

    let mut result = Vec::new();

    for dest in proxy_jump_value.split(',') {
        let dest = dest.trim();
        if dest.is_empty() || dest.eq_ignore_ascii_case("none") {
            continue;
        }

        // 检查是否为已知 Host 别名（不含 @ 和 : 的纯名称）
        let is_alias = !dest.contains('@') && !dest.contains(':');

        if is_alias {
            // 先递归解析该别名自己的 ProxyJump
            if let Some(nested_pj) = proxy_jump_map.get(dest) {
                let nested = resolve_proxy_jump(nested_pj, ssh_config, proxy_jump_map, depth + 1);
                result.extend(nested);
            }

            // 用 ssh_config.query 获取该别名的真实参数
            let params = ssh_config.query(dest);
            let hostname = params
                .host_name
                .clone()
                .unwrap_or_else(|| dest.to_string());
            let port = params.port.unwrap_or(22);
            let username = params.user.clone();
            let identity_file = params
                .identity_file
                .as_ref()
                .and_then(|files| files.first())
                .map(|p| p.to_string_lossy().to_string());

            result.push(ImportedJumpHost {
                host: hostname,
                port,
                username,
                identity_file,
            });
        } else {
            // 直接解析 [user@]host[:port] 格式
            let (host, port, user) = parse_jump_destination(dest);
            result.push(ImportedJumpHost {
                host,
                port,
                username: user,
                identity_file: None,
            });
        }
    }

    result
}

/// 导入 ~/.ssh/config 文件，返回主机列表
#[tauri::command]
pub async fn import_ssh_config() -> Result<Vec<ImportedServer>, String> {
    let home = dirs::home_dir().ok_or_else(|| "Cannot find home directory".to_string())?;
    let config_path = home.join(".ssh").join("config");

    if !config_path.exists() {
        return Ok(vec![]);
    }

    let file =
        std::fs::File::open(&config_path).map_err(|e| format!("Cannot open ssh config: {}", e))?;
    let mut reader = std::io::BufReader::new(file);

    let ssh_config = ssh2_config::SshConfig::default()
        .parse(
            &mut reader,
            ssh2_config::ParseRule::ALLOW_UNSUPPORTED_FIELDS,
        )
        .map_err(|e| format!("Failed to parse ssh config: {}", e))?;

    // 第一遍：收集所有 Host 别名及其 ProxyJump 值
    let mut proxy_jump_map = std::collections::HashMap::new();
    for host in ssh_config.get_hosts() {
        let host_name_clause = host.pattern.iter().find(|p| {
            p.pattern != "*" && !p.pattern.is_empty() && !p.negated
        });

        if let Some(pattern) = host_name_clause {
            let alias = pattern.pattern.clone();
            let params = ssh_config.query(&alias);

            if let Some(args) = params.unsupported_fields.get("proxyjump") {
                if let Some(value) = args.first() {
                    proxy_jump_map.insert(alias.clone(), value.clone());
                }
            }
        }
    }

    // 第二遍：构建导入列表
    let mut servers = Vec::new();
    for host in ssh_config.get_hosts() {
        let host_name_clause = host.pattern.iter().find(|p| {
            p.pattern != "*" && !p.pattern.is_empty() && !p.negated
        });

        if let Some(pattern) = host_name_clause {
            let alias = pattern.pattern.clone();
            let params = ssh_config.query(&alias);

            let hostname = params
                .host_name
                .clone()
                .unwrap_or_else(|| alias.clone());

            let port = params.port.unwrap_or(22);
            let username = params.user.clone();
            let identity_file = params
                .identity_file
                .as_ref()
                .and_then(|files| files.first())
                .map(|p| p.to_string_lossy().to_string());

            // 解析 ProxyJump
            let proxy_jump = proxy_jump_map.get(&alias).map(|pj_value| {
                resolve_proxy_jump(pj_value, &ssh_config, &proxy_jump_map, 0)
            });

            servers.push(ImportedServer {
                name: alias,
                host: hostname,
                port,
                username,
                identity_file,
                proxy_jump,
            });
        }
    }

    Ok(servers)
}
