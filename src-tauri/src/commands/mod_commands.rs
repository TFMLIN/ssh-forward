use std::collections::HashMap;
use std::sync::Arc;

use tauri::State;
use tokio::sync::{oneshot, watch, Mutex};
use uuid::Uuid;

use crate::ssh::client::connect;
use crate::ssh::forwarder::start_forward;
use crate::types::{ForwardRule, ImportedServer, SshServerConfig};

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
        .parse(&mut reader, ssh2_config::ParseRule::STRICT)
        .map_err(|e| format!("Failed to parse ssh config: {}", e))?;

    let mut servers = Vec::new();
    for host in ssh_config.get_hosts() {
        // 跳过通配符主机（Host *）
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

            servers.push(ImportedServer {
                name: alias,
                host: hostname,
                port,
                username,
                identity_file,
            });
        }
    }

    Ok(servers)
}
