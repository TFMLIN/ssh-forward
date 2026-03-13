<script setup lang="ts">
import { ref } from 'vue'
import { ElMessageBox, ElMessage } from 'element-plus'
import { Plus, Edit, Delete } from '@element-plus/icons-vue'
import { useServerStore } from '../stores/servers'
import ServerForm from './ServerForm.vue'
import type { SshServer } from '../types'

const store = useServerStore()

const showForm = ref(false)
const editingServer = ref<SshServer | null>(null)

function openAdd() {
  editingServer.value = null
  showForm.value = true
}

function openEdit(server: SshServer) {
  editingServer.value = server
  showForm.value = true
}

async function confirmDelete(server: SshServer) {
  try {
    await ElMessageBox.confirm(`确认删除服务器"${server.name}"及其所有转发规则吗？`, '删除确认', {
      confirmButtonText: '删除',
      cancelButtonText: '取消',
      type: 'warning',
    })
    store.removeServer(server.id)
    ElMessage.success('已删除')
  } catch {
    // 用户取消
  }
}

function handleSave(data: Omit<SshServer, 'id'>) {
  if (editingServer.value) {
    store.updateServer(editingServer.value.id, data)
    ElMessage.success('已更新')
  } else {
    const newServer = store.addServer(data)
    store.selectServer(newServer.id)
    ElMessage.success('已添加')
  }
}
</script>

<template>
  <div class="server-list">
    <div class="list-header">
      <span class="list-title">SSH 服务器</span>
      <el-button :icon="Plus" size="small" circle @click="openAdd" title="添加服务器" />
    </div>

    <el-scrollbar>
      <div
        v-for="server in store.servers"
        :key="server.id"
        class="server-item"
        :class="{ active: store.selectedServerId === server.id }"
        @click="store.selectServer(server.id)"
      >
        <div class="server-info">
          <div class="server-name">{{ server.name }}</div>
          <div class="server-addr">{{ server.username }}@{{ server.host }}:{{ server.port }}</div>
        </div>
        <div class="server-actions" @click.stop>
          <el-button
            :icon="Edit"
            size="small"
            circle
            text
            @click="openEdit(server)"
            title="编辑"
          />
          <el-button
            :icon="Delete"
            size="small"
            circle
            text
            type="danger"
            @click="confirmDelete(server)"
            title="删除"
          />
        </div>
      </div>

      <div v-if="store.servers.length === 0" class="empty-hint">
        <el-text type="info" size="small">暂无服务器，点击 + 添加</el-text>
      </div>
    </el-scrollbar>

    <ServerForm
      v-model="showForm"
      :server="editingServer"
      @save="handleSave"
    />
  </div>
</template>

<style scoped>
.server-list {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.list-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 12px 8px;
  border-bottom: 1px solid var(--el-border-color-light);
}

.list-title {
  font-weight: 600;
  font-size: 14px;
  color: var(--el-text-color-primary);
}

.server-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  cursor: pointer;
  border-bottom: 1px solid var(--el-border-color-lighter);
  transition: background-color 0.15s;
}

.server-item:hover {
  background-color: var(--el-fill-color-light);
}

.server-item.active {
  background-color: var(--el-color-primary-light-9);
  border-left: 3px solid var(--el-color-primary);
}

.server-info {
  flex: 1;
  min-width: 0;
}

.server-name {
  font-size: 13px;
  font-weight: 500;
  color: var(--el-text-color-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.server-addr {
  font-size: 11px;
  color: var(--el-text-color-secondary);
  margin-top: 2px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.server-actions {
  display: flex;
  gap: 2px;
  opacity: 0;
  transition: opacity 0.15s;
}

.server-item:hover .server-actions {
  opacity: 1;
}

.empty-hint {
  padding: 20px;
  text-align: center;
}
</style>
