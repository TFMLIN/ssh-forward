export type AuthType = 'password' | 'privateKey' | 'agent'

/** 跳板机配置（内联，不需要在服务器列表中存在） */
export interface JumpHost {
  host: string
  port: number
  username: string
  authType: AuthType
  password?: string
  privateKeyPath?: string
  passphrase?: string
}

/** 跳板机配置条目（UI 中使用，支持两种来源） */
export interface JumpEntry {
  /** 配置来源：引用已有服务器 or 手动输入 */
  type: 'server' | 'inline'
  /** type='server' 时：引用的服务器 id */
  serverId?: string
  /** type='inline' 时：内联配置 */
  inline?: JumpHost
}

/** SSH 服务器配置（持久化到 localStorage） */
export interface SshServer {
  id: string
  name: string
  host: string
  port: number
  username: string
  authType: AuthType
  password?: string
  privateKeyPath?: string
  passphrase?: string
  /** 有序跳板机链（UI 配置，持久化） */
  jumpEntries?: JumpEntry[]
}

/** 端口转发规则（持久化到 localStorage） */
export interface ForwardRule {
  id: string
  serverId: string
  localPort: number
  remoteHost: string
  remotePort: number
  description?: string
}

/** 运行时转发会话状态（不持久化） */
export interface ForwardSession {
  sessionId: string
  ruleId: string
  status: 'connecting' | 'active' | 'error' | 'stopped'
  error?: string
  startedAt: Date
}

/** 从 ~/.ssh/config 导入的主机信息 */
export interface ImportedServer {
  name: string
  host: string
  port: number
  username?: string
  identityFile?: string
}
