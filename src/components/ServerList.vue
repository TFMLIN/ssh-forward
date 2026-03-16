<script setup lang="ts">
import { ref } from 'vue'
import { Plus, Pencil, Trash2 } from 'lucide-vue-next'
import { useServerStore } from '../stores/servers'
import { getToast } from '../utils/toast'
import ServerForm from './ServerForm.vue'
import type { SshServer } from '../types'

const store = useServerStore()
const toast = getToast()

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
  if (confirm(`确认删除服务器"${server.name}"及其所有转发规则吗？`)) {
    store.removeServer(server.id)
    toast.success('已删除')
  }
}

function handleSave(data: Omit<SshServer, 'id'>) {
  if (editingServer.value) {
    store.updateServer(editingServer.value.id, data)
    toast.success('已更新')
  } else {
    const newServer = store.addServer(data)
    store.selectServer(newServer.id)
    toast.success('已添加')
  }
}
</script>

<template>
  <div class="flex flex-col h-full">
    <div class="flex items-center justify-between px-3 py-2 border-b border-gray-200">
      <span class="font-semibold text-sm text-gray-900">SSH 服务器</span>
      <button class="btn btn-sm btn-circle btn-ghost" @click="openAdd" title="添加服务器">
        <Plus class="w-4 h-4" />
      </button>
    </div>

    <div class="flex-1 overflow-y-auto">
      <div
        v-for="server in store.servers"
        :key="server.id"
        class="group flex items-center justify-between px-3 py-2.5 cursor-pointer border-b border-gray-100 transition-colors hover:bg-gray-50"
        :class="{ 'bg-blue-50 border-l-4 border-l-blue-500': store.selectedServerId === server.id }"
        @click="store.selectServer(server.id)"
      >
        <div class="flex-1 min-w-0">
          <div class="text-sm font-medium text-gray-900 truncate">{{ server.name }}</div>
          <div class="text-xs text-gray-500 truncate mt-0.5">{{ server.username }}@{{ server.host }}:{{ server.port }}</div>
        </div>
        <div class="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity" @click.stop>
          <button
            class="btn btn-xs btn-circle btn-ghost"
            @click="openEdit(server)"
            title="编辑"
          >
            <Pencil class="w-3.5 h-3.5" />
          </button>
          <button
            class="btn btn-xs btn-circle btn-ghost text-red-600 hover:bg-red-50"
            @click="confirmDelete(server)"
            title="删除"
          >
            <Trash2 class="w-3.5 h-3.5" />
          </button>
        </div>
      </div>

      <div v-if="store.servers.length === 0" class="p-5 text-center">
        <span class="text-sm text-gray-400">暂无服务器，点击 + 添加</span>
      </div>
    </div>

    <ServerForm v-model="showForm" :server="editingServer" @save="handleSave" />
  </div>
</template>
