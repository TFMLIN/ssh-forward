use serde::{Deserialize, Serialize};

/// 认证方式
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AuthType {
    Password,
    PrivateKey,
    Agent,
}

/// 跳板机配置（内联，不需要在服务器列表中存在）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JumpHost {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth_type: AuthType,
    pub password: Option<String>,
    pub private_key_path: Option<String>,
    pub passphrase: Option<String>,
}

/// SSH 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SshServerConfig {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth_type: AuthType,
    pub password: Option<String>,
    pub private_key_path: Option<String>,
    pub passphrase: Option<String>,
    /// 有序跳板机链（前端已将 jump_server_ids 解析为完整配置后传入）
    /// 连接时依次通过每个跳板机，最终到达目标服务器
    pub jump_hosts: Option<Vec<JumpHost>>,
}

/// 端口转发规则
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ForwardRule {
    pub id: String,
    pub server_id: String,
    pub local_port: u16,
    pub remote_host: String,
    pub remote_port: u16,
    pub description: Option<String>,
}

/// 从 ~/.ssh/config 导入的服务器信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportedServer {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub identity_file: Option<String>,
}
