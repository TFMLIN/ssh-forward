import { defineStore } from 'pinia'
import { ref } from 'vue'
import { v4 as uuidv4 } from 'uuid'
import type { SshServer, ForwardRule, ForwardSession, JumpHost } from '../types'

export const useServerStore = defineStore(
  'servers',
  () => {
    const servers = ref<SshServer[]>([])
    const rules = ref<ForwardRule[]>([])
    const sessions = ref<Record<string, ForwardSession>>({})
    const selectedServerId = ref<string | null>(null)

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
  },
  {
    persist: {
      // 只持久化 servers 和 rules，sessions 是运行时状态
      pick: ['servers', 'rules'],
    },
  }
)
