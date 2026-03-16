import { defineStore } from 'pinia'
import { ref, watch } from 'vue'
import { v4 as uuidv4 } from 'uuid'
import { loadConfig, saveConfig, type ConfigData } from '../utils/config'
import type { SshServer, ForwardRule, ForwardSession, JumpHost } from '../types'

export const useServerStore = defineStore('servers', () => {
  const servers = ref<SshServer[]>([])
  const rules = ref<ForwardRule[]>([])
  const sessions = ref<Record<string, ForwardSession>>({})
  const selectedServerId = ref<string | null>(null)
  const isLoaded = ref(false)

  // --- 配置持久化 ---

  /** 从文件加载配置 */
  async function loadFromFile(): Promise<void> {
    try {
      const data = await loadConfig()
      servers.value = data.servers
      rules.value = data.rules
      isLoaded.value = true
    } catch (e) {
      console.error('加载配置失败:', e)
      isLoaded.value = true
    }
  }

  /** 保存配置到文件 */
  async function saveToFile(): Promise<void> {
    if (!isLoaded.value) return // 未加载完成时不保存
    try {
      await saveConfig({
        servers: servers.value,
        rules: rules.value,
      })
    } catch (e) {
      console.error('保存配置失败:', e)
    }
  }

  // 监听数据变化，自动保存
  watch(
    [servers, rules],
    () => {
      saveToFile()
    },
    { deep: true }
  )

  /** 导入配置（合并模式） */
  function mergeConfig(data: ConfigData): { servers: number; rules: number } {
    let serverCount = 0
    let ruleCount = 0

    // 合并服务器（按名称去重）
    const existingNames = new Set(servers.value.map((s) => s.name))
    for (const server of data.servers) {
      if (!existingNames.has(server.name)) {
        servers.value.push({ ...server, id: uuidv4() })
        serverCount++
      }
    }

    // 合并规则（按 localPort + serverId 去重）
    const existingRuleKeys = new Set(
      rules.value.map((r) => `${r.serverId}:${r.localPort}`)
    )
    for (const rule of data.rules) {
      // 尝试找到对应的服务器
      const serverName = data.servers.find((s) => s.id === rule.serverId)?.name
      const localServer = servers.value.find((s) => s.name === serverName)
      if (localServer) {
        const key = `${localServer.id}:${rule.localPort}`
        if (!existingRuleKeys.has(key)) {
          rules.value.push({ ...rule, id: uuidv4(), serverId: localServer.id })
          ruleCount++
        }
      }
    }

    return { servers: serverCount, rules: ruleCount }
  }

  /** 导入配置（覆盖模式） */
  function replaceConfig(data: ConfigData): void {
    // 清空现有数据
    servers.value = []
    rules.value = []
    sessions.value = {}
    selectedServerId.value = null

    // 重新生成 ID 并导入
    const idMap = new Map<string, string>()
    for (const server of data.servers) {
      const newId = uuidv4()
      idMap.set(server.id, newId)
      servers.value.push({ ...server, id: newId })
    }

    for (const rule of data.rules) {
      const newServerId = idMap.get(rule.serverId)
      if (newServerId) {
        rules.value.push({ ...rule, id: uuidv4(), serverId: newServerId })
      }
    }
  }

  /** 获取当前配置数据（用于导出） */
  function getConfigData(): ConfigData {
    return {
      servers: servers.value,
      rules: rules.value,
    }
  }

  // --- 服务器操作 ---

  function addServer(server: Omit<SshServer, 'id'>): SshServer {
    const newServer: SshServer = { ...server, id: uuidv4() }
    servers.value.push(newServer)
    return newServer
  }

  function updateServer(id: string, updates: Partial<Omit<SshServer, 'id'>>) {
    const idx = servers.value.findIndex((s) => s.id === id)
    if (idx !== -1) {
      servers.value[idx] = { ...servers.value[idx], ...updates }
    }
  }

  function removeServer(id: string) {
    servers.value = servers.value.filter((s) => s.id !== id)
    // 同时移除该服务器的所有转发规则
    rules.value = rules.value.filter((r) => r.serverId !== id)
    if (selectedServerId.value === id) {
      selectedServerId.value = null
    }
  }

  function selectServer(id: string | null) {
    selectedServerId.value = id
  }

  function getSelectedServer(): SshServer | undefined {
    if (!selectedServerId.value) return undefined
    return servers.value.find((s) => s.id === selectedServerId.value)
  }

  // --- 转发规则操作 ---

  function addRule(rule: Omit<ForwardRule, 'id'>): ForwardRule {
    const newRule: ForwardRule = { ...rule, id: uuidv4() }
    rules.value.push(newRule)
    return newRule
  }

  function updateRule(id: string, updates: Partial<Omit<ForwardRule, 'id'>>) {
    const idx = rules.value.findIndex((r) => r.id === id)
    if (idx !== -1) {
      rules.value[idx] = { ...rules.value[idx], ...updates }
    }
  }

  function removeRule(id: string) {
    rules.value = rules.value.filter((r) => r.id !== id)
  }

  function getRulesForServer(serverId: string): ForwardRule[] {
    return rules.value.filter((r) => r.serverId === serverId)
  }

  // --- 会话状态操作（不持久化）---

  function setSession(ruleId: string, sessionId: string) {
    sessions.value[ruleId] = {
      sessionId,
      ruleId,
      status: 'connecting',
      startedAt: new Date(),
    }
  }

  function updateSessionStatus(
    ruleId: string,
    status: ForwardSession['status'],
    error?: string
  ) {
    if (sessions.value[ruleId]) {
      sessions.value[ruleId].status = status
      if (error) sessions.value[ruleId].error = error
    }
  }

  function removeSession(ruleId: string) {
    delete sessions.value[ruleId]
  }

  function getSession(ruleId: string): ForwardSession | undefined {
    return sessions.value[ruleId]
  }

  /**
   * 将服务器的 jumpEntries 解析为有序的 JumpHost 列表，传给 Rust 后端。
   * - type='server'：从服务器列表中查找对应服务器并转换
   * - type='inline'：直接使用内联配置
   */
  function buildJumpChain(server: SshServer): JumpHost[] {
    if (!server.jumpEntries || server.jumpEntries.length === 0) return []
    const result: JumpHost[] = []
    for (const entry of server.jumpEntries) {
      if (entry.type === 'inline' && entry.inline) {
        result.push(entry.inline)
      } else if (entry.type === 'server' && entry.serverId) {
        const jumpServer = servers.value.find((s) => s.id === entry.serverId)
        if (jumpServer) {
          result.push({
            host: jumpServer.host,
            port: jumpServer.port,
            username: jumpServer.username,
            authType: jumpServer.authType,
            password: jumpServer.password,
            privateKeyPath: jumpServer.privateKeyPath,
            passphrase: jumpServer.passphrase,
          })
        }
      }
    }
    return result
  }

  return {
    // state
    servers,
    rules,
    sessions,
    selectedServerId,
    isLoaded,
    // config actions
    loadFromFile,
    saveToFile,
    mergeConfig,
    replaceConfig,
    getConfigData,
    // server actions
    addServer,
    updateServer,
    removeServer,
    selectServer,
    getSelectedServer,
    // rule actions
    addRule,
    updateRule,
    removeRule,
    getRulesForServer,
    // session actions
    setSession,
    updateSessionStatus,
    removeSession,
    getSession,
    // jump helpers
    buildJumpChain,
  }
})
