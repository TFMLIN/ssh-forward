<script setup lang="ts">
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage } from 'element-plus'
import type { ImportedServer, SshServer } from '../types'

defineProps<{
  modelValue: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  import: [servers: Omit<SshServer, 'id'>[]]
}>()

const loading = ref(false)
const importedList = ref<ImportedServer[]>([])
const selectedNames = ref<string[]>([])

async function loadConfig() {
  loading.value = true
  try {
    importedList.value = await invoke<ImportedServer[]>('import_ssh_config')
    selectedNames.value = importedList.value.map((s) => s.name)
    if (importedList.value.length === 0) {
      ElMessage.info('~/.ssh/config 中未找到主机配置')
    }
  } catch (e: any) {
    ElMessage.error(`读取失败: ${e}`)
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
  selectedNames.value = []
}

function handleImport() {
  const selected = importedList.value.filter((s) => selectedNames.value.includes(s.name))
  const servers: Omit<SshServer, 'id'>[] = selected.map((s) => ({
    name: s.name,
    host: s.host,
    port: s.port,
    username: s.username ?? '',
    authType: s.identityFile ? 'privateKey' : 'password',
    privateKeyPath: s.identityFile,
  }))
  emit('import', servers)
  emit('update:modelValue', false)
}
</script>

<template>
  <el-dialog
    :model-value="modelValue"
    title="从 ~/.ssh/config 导入"
    width="520px"
    @open="open"
    @close="handleClose"
  >
    <div v-loading="loading">
      <el-alert
        v-if="importedList.length === 0 && !loading"
        title="未找到主机配置"
        type="info"
        description="请确认 ~/.ssh/config 文件存在且包含 Host 配置项"
        :closable="false"
      />
      <div v-else>
        <el-text type="info" size="small" style="display: block; margin-bottom: 12px">
          发现 {{ importedList.length }} 条主机配置，勾选要导入的项目：
        </el-text>
        <el-checkbox-group v-model="selectedNames">
          <el-table :data="importedList" style="width: 100%">
            <el-table-column width="48">
              <template #default="{ row }">
                <el-checkbox :value="row.name" />
              </template>
            </el-table-column>
            <el-table-column label="名称" prop="name" min-width="110" />
            <el-table-column label="主机" min-width="140">
              <template #default="{ row }">{{ row.host }}:{{ row.port }}</template>
            </el-table-column>
            <el-table-column label="用户名" prop="username" min-width="90">
              <template #default="{ row }">{{ row.username || '-' }}</template>
            </el-table-column>
            <el-table-column label="认证" width="70" align="center">
              <template #default="{ row }">
                <el-tag size="small" :type="row.identityFile ? 'success' : 'info'" effect="plain">
                  {{ row.identityFile ? '私钥' : '密码' }}
                </el-tag>
              </template>
            </el-table-column>
          </el-table>
        </el-checkbox-group>
      </div>
    </div>

    <template #footer>
      <el-button @click="handleClose">取消</el-button>
      <el-button
        type="primary"
        :disabled="selectedNames.length === 0"
        @click="handleImport"
      >
        导入选中 ({{ selectedNames.length }})
      </el-button>
    </template>
  </el-dialog>
</template>
