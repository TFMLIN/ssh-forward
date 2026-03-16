<script setup lang="ts">
import { ref, computed } from 'vue'
import { open, save } from '@tauri-apps/plugin-dialog'
import { Upload, Download, FolderOpen } from 'lucide-vue-next'
import { useServerStore } from './stores/servers'
import { getToast } from './utils/toast'
import { importConfigFromPath, exportConfigToPath, type ConfigData } from './utils/config'
import ServerList from './components/ServerList.vue'
import ForwardList from './components/ForwardList.vue'
import ImportSshConfig from './components/ImportSshConfig.vue'
import ImportConfigDialog from './components/ImportConfigDialog.vue'
import Toast from './components/Toast.vue'
import type { SshServer } from './types'

const store = useServerStore()
const toast = getToast()

const selectedServer = computed(() => store.getSelectedServer())
const showSshImport = ref(false)
const showConfigImport = ref(false)
const pendingConfigData = ref<ConfigData | null>(null)

// 导入 SSH config
function handleSshImport(servers: Omit<SshServer, 'id'>[]) {
  let count = 0
  for (const s of servers) {
    store.addServer(s)
    count++
  }
  toast.success(`已导入 ${count} 个服务器`)
}

// 导出配置
async function handleExportConfig() {
  try {
    const filePath = await save({
      title: '导出配置',
      defaultPath: 'ssh-forward-config.json',
      filters: [{ name: 'JSON', extensions: ['json'] }],
    })
    if (!filePath) return

    const data = store.getConfigData()
    await exportConfigToPath(filePath, data)
    toast.success('配置已导出')
  } catch (e: any) {
    console.error('导出配置失败:', e)
    toast.error(`导出失败: ${e?.message || e}`)
  }
}

// 选择配置文件并打开导入对话框
async function handleSelectConfigFile() {
  try {
    const filePath = await open({
      title: '导入配置',
      multiple: false,
      filters: [{ name: 'JSON', extensions: ['json'] }],
    })
    if (!filePath) return

    const data = await importConfigFromPath(filePath as string)
    pendingConfigData.value = data
    showConfigImport.value = true
  } catch (e: any) {
    console.error('读取配置文件失败:', e)
    toast.error(`读取失败: ${e?.message || e}`)
  }
}

// 执行配置导入
function handleConfigImport(mode: 'merge' | 'replace') {
  if (!pendingConfigData.value) return

  if (mode === 'merge') {
    const result = store.mergeConfig(pendingConfigData.value)
    toast.success(`已合并 ${result.servers} 个服务器和 ${result.rules} 条规则`)
  } else {
    store.replaceConfig(pendingConfigData.value)
    toast.success('配置已覆盖')
  }

  pendingConfigData.value = null
}
</script>

<template>
  <div class="flex flex-col h-screen overflow-hidden bg-white">
    <!-- 顶部栏 -->
    <header class="flex items-center justify-between px-4 h-12 border-b border-gray-200 bg-white flex-shrink-0">
      <span class="font-semibold text-base text-gray-900">SSH 端口转发管理器</span>
      <div class="flex gap-2">
        <button
          class="btn btn-sm btn-outline gap-1"
          @click="handleSelectConfigFile"
          title="导入配置文件"
        >
          <FolderOpen class="w-4 h-4" />
          导入配置
        </button>
        <button
          class="btn btn-sm btn-outline gap-1"
          @click="handleExportConfig"
          title="导出配置文件"
        >
          <Download class="w-4 h-4" />
          导出配置
        </button>
        <button
          class="btn btn-sm btn-outline gap-1"
          @click="showSshImport = true"
          title="从 SSH config 导入"
        >
          <Upload class="w-4 h-4" />
          导入 SSH Config
        </button>
      </div>
    </header>

    <!-- 主体 -->
    <div class="flex flex-1 overflow-hidden">
      <!-- 左侧服务器列表 -->
      <aside class="w-56 flex-shrink-0 border-r border-gray-200 overflow-hidden flex flex-col">
        <ServerList />
      </aside>

      <!-- 右侧转发规则区 -->
      <main class="flex-1 overflow-hidden flex flex-col">
        <ForwardList v-if="selectedServer" :server="selectedServer" />
        <div v-else class="flex-1 flex items-center justify-center">
          <div class="text-center">
            <div class="text-6xl mb-4">📋</div>
            <p class="text-gray-500">请在左侧选择一个 SSH 服务器</p>
          </div>
        </div>
      </main>
    </div>

    <!-- SSH Config 导入对话框 -->
    <ImportSshConfig v-model="showSshImport" @import="handleSshImport" />

    <!-- 配置文件导入对话框 -->
    <ImportConfigDialog
      v-model="showConfigImport"
      :config-data="pendingConfigData"
      @import="handleConfigImport"
    />

    <!-- Toast 通知 -->
    <Toast />
  </div>
</template>
