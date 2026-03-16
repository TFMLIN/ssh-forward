<script setup lang="ts">
import { ref, computed, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { openUrl } from '@tauri-apps/plugin-opener'
import { Plus, Pencil, Trash2, Play, Pause, Link2, Activity } from 'lucide-vue-next'
import { useServerStore } from '../stores/servers'
import { getToast } from '../utils/toast'
import ForwardForm from './ForwardForm.vue'
import type { ForwardRule, SshServer } from '../types'

const props = defineProps<{
  server: SshServer
}>()

const store = useServerStore()
const toast = getToast()

const rules = computed(() => store.getRulesForServer(props.server.id))

const showForm = ref(false)
const editingRule = ref<ForwardRule | null>(null)

// 定期轮询所有活跃会话的状态
const pollInterval = setInterval(async () => {
  try {
    const statuses = await invoke<Record<string, string>>('get_all_statuses')
    for (const rule of rules.value) {
      const session = store.getSession(rule.id)
      if (!session) continue
      const backendStatus = statuses[session.sessionId]
      if (!backendStatus || backendStatus === 'stopped') {
        store.updateSessionStatus(rule.id, 'stopped')
      } else if (backendStatus === 'active') {
        store.updateSessionStatus(rule.id, 'active')
      } else if (backendStatus.startsWith('error:')) {
        store.updateSessionStatus(rule.id, 'error', backendStatus.slice(6))
      }
    }
  } catch {
    // 忽略轮询错误
  }
}, 2000)

onUnmounted(() => clearInterval(pollInterval))

function openAdd() {
  editingRule.value = null
  showForm.value = true
}

function openEdit(rule: ForwardRule) {
  editingRule.value = rule
  showForm.value = true
}

async function confirmDelete(rule: ForwardRule) {
  await stopForward(rule)
  if (confirm(`确认删除规则"本地:${rule.localPort} → ${rule.remoteHost}:${rule.remotePort}"吗？`)) {
    store.removeRule(rule.id)
    toast.success('已删除')
  }
}

function handleSaveRule(data: Omit<ForwardRule, 'id'>) {
  if (editingRule.value) {
    store.updateRule(editingRule.value.id, data)
    toast.success('已更新')
  } else {
    store.addRule(data)
    toast.success('已添加')
  }
}

async function startForward(rule: ForwardRule) {
  store.setSession(rule.id, '')
  store.updateSessionStatus(rule.id, 'connecting')
  try {
    const serverWithJumps = {
      ...props.server,
      jumpHosts: store.buildJumpChain(props.server),
    }
    const sessionId = await invoke<string>('start_forward_cmd', {
      server: serverWithJumps,
      rule,
    })
    store.setSession(rule.id, sessionId)
    store.updateSessionStatus(rule.id, 'active')
    toast.success(`端口 ${rule.localPort} 转发已启动`)
  } catch (e: any) {
    store.updateSessionStatus(rule.id, 'error', String(e))
    toast.error(`启动失败: ${e}`)
  }
}

async function stopForward(rule: ForwardRule) {
  const session = store.getSession(rule.id)
  if (!session || !session.sessionId) return
  try {
    await invoke('stop_forward_cmd', { sessionId: session.sessionId })
    store.removeSession(rule.id)
    toast.success(`端口 ${rule.localPort} 转发已停止`)
  } catch (e: any) {
    toast.error(`停止失败: ${e}`)
  }
}

async function testConnection() {
  toast.info('测试连接中...')
  try {
    const serverWithJumps = {
      ...props.server,
      jumpHosts: store.buildJumpChain(props.server),
    }
    await invoke('test_connection', { server: serverWithJumps })
    toast.success('连接成功！')
  } catch (e: any) {
    toast.error(`连接失败: ${e}`)
  }
}

function getStatusBadge(ruleId: string): { class: string; text: string } {
  const session = store.getSession(ruleId)
  if (!session) return { class: 'badge-ghost', text: '未启动' }
  switch (session.status) {
    case 'active': return { class: 'badge-success', text: '运行中' }
    case 'connecting': return { class: 'badge-warning', text: '连接中' }
    case 'error': return { class: 'badge-error', text: '错误' }
    case 'stopped': return { class: 'badge-ghost', text: '已停止' }
    default: return { class: 'badge-ghost', text: '未知' }
  }
}

function isRunning(ruleId: string): boolean {
  const session = store.getSession(ruleId)
  return !!session && (session.status === 'active' || session.status === 'connecting')
}

function getLocalUrl(rule: ForwardRule): string {
  return `http://localhost:${rule.localPort}`
}

async function openLocalLink(rule: ForwardRule) {
  const url = getLocalUrl(rule)
  try {
    await openUrl(url)
  } catch (e: any) {
    toast.error(`打开链接失败: ${e}`)
  }
}
</script>

<template>
  <div class="flex flex-col h-full">
    <div class="flex items-center justify-between px-4 py-3 border-b border-gray-200">
      <div class="flex flex-col gap-0.5">
        <span class="font-semibold text-base">{{ server.name }}</span>
        <span class="text-xs text-gray-500">{{ server.username }}@{{ server.host }}:{{ server.port }}</span>
      </div>
      <div class="flex gap-2">
        <button class="btn btn-sm btn-outline gap-1" @click="testConnection">
          <Activity class="w-4 h-4" />
          测试连接
        </button>
        <button class="btn btn-sm btn-primary gap-1" @click="openAdd">
          <Plus class="w-4 h-4" />
          添加转发
        </button>
      </div>
    </div>

    <div class="flex-1 overflow-auto">
      <div v-if="rules.length === 0" class="flex justify-center py-10">
        <div class="text-center">
          <div class="text-5xl mb-3">🔄</div>
          <p class="text-gray-500">暂无转发规则，点击「添加转发」创建一条</p>
        </div>
      </div>

      <table v-else class="table table-zebra w-full">
        <thead>
          <tr>
            <th class="w-32">本地端口</th>
            <th class="w-8 text-center"></th>
            <th>远端地址</th>
            <th>描述</th>
            <th class="w-24 text-center">状态</th>
            <th class="w-32 text-right">操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="rule in rules" :key="rule.id">
            <td>
              <div class="flex items-center gap-2">
                <span>:{{ rule.localPort }}</span>
                <button
                  v-if="isRunning(rule.id)"
                  class="btn btn-xs btn-link"
                  @click="openLocalLink(rule)"
                  title="在浏览器中打开"
                >
                  <Link2 class="w-3.5 h-3.5" />
                  打开
                </button>
              </div>
            </td>
            <td class="text-center">→</td>
            <td>{{ rule.remoteHost }}:{{ rule.remotePort }}</td>
            <td class="text-gray-500 text-sm">{{ rule.description || '-' }}</td>
            <td class="text-center">
              <span class="badge whitespace-nowrap px-2.5 py-0.5" :class="getStatusBadge(rule.id).class">
                {{ getStatusBadge(rule.id).text }}
              </span>
            </td>
            <td class="text-right">
              <button
                v-if="!isRunning(rule.id)"
                class="btn btn-xs btn-ghost text-green-600"
                @click="startForward(rule)"
                title="启动"
              >
                <Play class="w-4 h-4" />
              </button>
              <button
                v-else
                class="btn btn-xs btn-ghost text-yellow-600"
                @click="stopForward(rule)"
                title="停止"
              >
                <Pause class="w-4 h-4" />
              </button>
              <button
                class="btn btn-xs btn-ghost"
                :disabled="isRunning(rule.id)"
                @click="openEdit(rule)"
                title="编辑"
              >
                <Pencil class="w-4 h-4" />
              </button>
              <button
                class="btn btn-xs btn-ghost text-red-600"
                @click="confirmDelete(rule)"
                title="删除"
              >
                <Trash2 class="w-4 h-4" />
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <ForwardForm v-model="showForm" :server-id="server.id" :rule="editingRule" @save="handleSaveRule" />
  </div>
</template>