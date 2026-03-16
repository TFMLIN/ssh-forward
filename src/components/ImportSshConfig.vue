<script setup lang="ts">
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { Loader2 } from 'lucide-vue-next'
import { getToast } from '../utils/toast'
import type { ImportedServer, SshServer, JumpEntry } from '../types'

const toast = getToast()

defineProps<{
  modelValue: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  import: [servers: Omit<SshServer, 'id'>[]]
}>()

const loading = ref(false)
const importedList = ref<ImportedServer[]>([])
const selectedNames = ref<Set<string>>(new Set())

// 根据操作系统显示不同的路径
const sshConfigPath = computed(() => {
  const isWindows = navigator.userAgent.includes('Windows')
  return isWindows ? '%USERPROFILE%\\.ssh\\config' : '~/.ssh/config'
})

async function loadConfig() {
  loading.value = true
  try {
    importedList.value = await invoke<ImportedServer[]>('import_ssh_config')
    selectedNames.value = new Set(importedList.value.map((s) => s.name))
    if (importedList.value.length === 0) {
      toast.info(`${sshConfigPath.value} 中未找到主机配置`)
    }
  } catch (e: any) {
    console.error('导入 SSH config 失败:', e)
    toast.error(`读取失败: ${e?.message || e}`)
  } finally {
    loading.value = false
  }
}

function open() {
  loadConfig()
}

function handleClose() {
  emit('update:modelValue', false)
  importedList.value = []
  selectedNames.value = new Set()
}

function toggleSelect(name: string) {
  if (selectedNames.value.has(name)) {
    selectedNames.value.delete(name)
  } else {
    selectedNames.value.add(name)
  }
  // 触发响应式更新
  selectedNames.value = new Set(selectedNames.value)
}

function handleImport() {
  const selected = importedList.value.filter((s) => selectedNames.value.has(s.name))
  const servers: Omit<SshServer, 'id'>[] = selected.map((s) => {
    // 将 proxyJump 转换为 jumpEntries（inline 类型）
    const jumpEntries: JumpEntry[] = (s.proxyJump ?? []).map((j) => ({
      type: 'inline' as const,
      inline: {
        host: j.host,
        port: j.port,
        username: j.username ?? '',
        authType: j.identityFile ? 'privateKey' as const : 'password' as const,
        privateKeyPath: j.identityFile,
      },
    }))

    return {
      name: s.name,
      host: s.host,
      port: s.port,
      username: s.username ?? '',
      authType: s.identityFile ? 'privateKey' : 'password',
      privateKeyPath: s.identityFile,
      jumpEntries: jumpEntries.length > 0 ? jumpEntries : undefined,
    }
  })
  emit('import', servers)
  emit('update:modelValue', false)
}
</script>

<template>
  <dialog class="modal" :class="{ 'modal-open': modelValue }" @transitionend="modelValue && open()">
    <div class="modal-box w-[540px] max-w-[90vw]">
      <h3 class="font-bold text-lg mb-4">从 {{ sshConfigPath }} 导入</h3>

      <div class="min-h-[120px]">
        <!-- Loading -->
        <div v-if="loading" class="flex items-center justify-center py-8">
          <Loader2 class="w-6 h-6 animate-spin text-gray-400" />
          <span class="ml-2 text-gray-500">加载中...</span>
        </div>

        <!-- Empty -->
        <div v-else-if="importedList.length === 0" class="alert">
          <div>
            <h4 class="font-medium">未找到主机配置</h4>
            <p class="text-sm text-gray-500">请确认 {{ sshConfigPath }} 文件存在且包含 Host 配置项</p>
          </div>
        </div>

        <!-- List -->
        <div v-else>
          <p class="text-sm text-gray-500 mb-3">
            发现 {{ importedList.length }} 条主机配置，勾选要导入的项目：
          </p>
          <div class="overflow-x-auto">
            <table class="table table-sm w-full">
              <thead>
                <tr class="text-xs">
                  <th class="w-10"></th>
                  <th>名称</th>
                  <th>主机</th>
                  <th>用户名</th>
                  <th class="text-center">认证</th>
                  <th>跳板机</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="row in importedList" :key="row.name" class="hover">
                  <td>
                    <input
                      type="checkbox"
                      class="checkbox checkbox-sm"
                      :checked="selectedNames.has(row.name)"
                      @change="toggleSelect(row.name)"
                    />
                  </td>
                  <td class="font-medium">{{ row.name }}</td>
                  <td>{{ row.host }}:{{ row.port }}</td>
                  <td>{{ row.username || '-' }}</td>
                  <td class="text-center">
                    <span
                      class="badge badge-sm"
                      :class="row.identityFile ? 'badge-success' : 'badge-ghost'"
                    >
                      {{ row.identityFile ? '私钥' : '密码' }}
                    </span>
                  </td>
                  <td>
                    <template v-if="row.proxyJump && row.proxyJump.length > 0">
                      <span
                        v-for="(j, i) in row.proxyJump"
                        :key="i"
                        class="badge badge-sm badge-warning badge-outline mr-1"
                      >
                        {{ j.host }}:{{ j.port }}
                      </span>
                    </template>
                    <span v-else class="text-gray-400">-</span>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </div>

      <div class="modal-action">
        <button class="btn btn-sm" @click="handleClose">取消</button>
        <button
          class="btn btn-sm btn-primary"
          :disabled="selectedNames.size === 0"
          @click="handleImport"
        >
          导入选中 ({{ selectedNames.size }})
        </button>
      </div>
    </div>
    <div class="modal-backdrop bg-black/30" @click="handleClose"></div>
  </dialog>
</template>
