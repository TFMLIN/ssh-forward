<script setup lang="ts">
import { ref, computed, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Plus, Edit, Delete, VideoPlay, VideoPause, Connection } from '@element-plus/icons-vue'
import { useServerStore } from '../stores/servers'
import ForwardForm from './ForwardForm.vue'
import type { ForwardRule, SshServer } from '../types'

const props = defineProps<{
  server: SshServer
}>()

const store = useServerStore()

const rules = computed(() => store.getRulesForServer(props.server.id))

const showForm = ref(false)
const editingRule = ref<ForwardRule | null>(null)

// 定期轮询所有活跃会话的状态
const pollInterval = setInterval(async () => {
  try {
    const statuses = await invoke<Record<string, string>>('get_all_statuses')
    // 根据后端状态更新 store
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
  // 如果有活跃会话先停止
  await stopForward(rule)
  try {
    await ElMessageBox.confirm(
      `确认删除规则"本地:${rule.localPort} → ${rule.remoteHost}:${rule.remotePort}"吗？`,
      '删除确认',
      { confirmButtonText: '删除', cancelButtonText: '取消', type: 'warning' }
    )
    store.removeRule(rule.id)
    ElMessage.success('已删除')
  } catch {
    // 取消
  }
}

function handleSaveRule(data: Omit<ForwardRule, 'id'>) {
  if (editingRule.value) {
    store.updateRule(editingRule.value.id, data)
    ElMessage.success('已更新')
  } else {
    store.addRule(data)
    ElMessage.success('已添加')
  }
}

async function startForward(rule: ForwardRule) {
  store.setSession(rule.id, '')
  store.updateSessionStatus(rule.id, 'connecting')
  try {
    // 将 jumpEntries 解析为后端所需的 jumpHosts 格式
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
    ElMessage.success(`端口 ${rule.localPort} 转发已启动`)
  } catch (e: any) {
    store.updateSessionStatus(rule.id, 'error', String(e))
    ElMessage.error(`启动失败: ${e}`)
  }
}

async function stopForward(rule: ForwardRule) {
  const session = store.getSession(rule.id)
  if (!session || !session.sessionId) return
  try {
    await invoke('stop_forward_cmd', { sessionId: session.sessionId })
    store.removeSession(rule.id)
    ElMessage.success(`端口 ${rule.localPort} 转发已停止`)
  } catch (e: any) {
    ElMessage.error(`停止失败: ${e}`)
  }
}

async function testConnection() {
  ElMessage.info('测试连接中...')
  try {
    const serverWithJumps = {
      ...props.server,
      jumpHosts: store.buildJumpChain(props.server),
    }
    await invoke('test_connection', { server: serverWithJumps })
    ElMessage.success('连接成功！')
  } catch (e: any) {
    ElMessage.error(`连接失败: ${e}`)
  }
}

function getStatusTag(ruleId: string): { type: 'success' | 'info' | 'danger' | 'warning'; text: string } {
  const session = store.getSession(ruleId)
  if (!session) return { type: 'info', text: '未启动' }
  switch (session.status) {
    case 'active': return { type: 'success', text: '运行中' }
    case 'connecting': return { type: 'warning', text: '连接中' }
    case 'error': return { type: 'danger', text: '错误' }
    case 'stopped': return { type: 'info', text: '已停止' }
    default: return { type: 'info', text: '未知' }
  }
}

function isRunning(ruleId: string): boolean {
  const session = store.getSession(ruleId)
  return !!session && (session.status === 'active' || session.status === 'connecting')
}
</script>

<template>
  <div class="forward-list">
    <div class="list-header">
      <div class="header-left">
        <span class="server-title">{{ server.name }}</span>
        <span class="server-sub">{{ server.username }}@{{ server.host }}:{{ server.port }}</span>
      </div>
      <div class="header-right">
        <el-button size="small" :icon="Connection" @click="testConnection">测试连接</el-button>
        <el-button size="small" type="primary" :icon="Plus" @click="openAdd">添加转发</el-button>
      </div>
    </div>

    <el-scrollbar>
      <div v-if="rules.length === 0" class="empty-state">
        <el-empty description="暂无转发规则，点击「添加转发」创建一条" :image-size="80" />
      </div>

      <el-table v-else :data="rules" style="width: 100%">
        <el-table-column label="本地端口" width="100">
          <template #default="{ row }">
            <el-text>:{{ row.localPort }}</el-text>
          </template>
        </el-table-column>
        <el-table-column label="" width="30" align="center">
          <template #default>→</template>
        </el-table-column>
        <el-table-column label="远端地址" min-width="160">
          <template #default="{ row }">
            <el-text>{{ row.remoteHost }}:{{ row.remotePort }}</el-text>
          </template>
        </el-table-column>
        <el-table-column label="描述" min-width="120">
          <template #default="{ row }">
            <el-text type="info" size="small">{{ row.description || '-' }}</el-text>
          </template>
        </el-table-column>
        <el-table-column label="状态" width="90" align="center">
          <template #default="{ row }">
            <el-tag :type="getStatusTag(row.id).type" size="small" effect="plain">
              {{ getStatusTag(row.id).text }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column label="操作" width="140" align="right">
          <template #default="{ row }">
            <el-button
              v-if="!isRunning(row.id)"
              :icon="VideoPlay"
              size="small"
              type="success"
              text
              @click="startForward(row)"
              title="启动"
            />
            <el-button
              v-else
              :icon="VideoPause"
              size="small"
              type="warning"
              text
              @click="stopForward(row)"
              title="停止"
            />
            <el-button
              :icon="Edit"
              size="small"
              text
              @click="openEdit(row)"
              :disabled="isRunning(row.id)"
              title="编辑"
            />
            <el-button
              :icon="Delete"
              size="small"
              text
              type="danger"
              @click="confirmDelete(row)"
              title="删除"
            />
          </template>
        </el-table-column>
      </el-table>
    </el-scrollbar>

    <ForwardForm
      v-model="showForm"
      :server-id="server.id"
      :rule="editingRule"
      @save="handleSaveRule"
    />
  </div>
</template>

<style scoped>
.forward-list {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.list-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid var(--el-border-color-light);
  flex-shrink: 0;
}

.header-left {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.server-title {
  font-weight: 600;
  font-size: 15px;
  color: var(--el-text-color-primary);
}

.server-sub {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.header-right {
  display: flex;
  gap: 8px;
}

.empty-state {
  padding: 40px 0;
  display: flex;
  justify-content: center;
}
</style>
