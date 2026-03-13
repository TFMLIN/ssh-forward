<script setup lang="ts">
import { ref, watch } from 'vue'
import { ElMessage } from 'element-plus'
import type { ForwardRule } from '../types'

const props = defineProps<{
  modelValue: boolean
  serverId: string
  rule?: ForwardRule | null
}>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  save: [rule: Omit<ForwardRule, 'id'>]
}>()

const form = ref<Omit<ForwardRule, 'id'>>({
  serverId: props.serverId,
  localPort: 8080,
  remoteHost: 'localhost',
  remotePort: 8080,
  description: '',
})

watch(
  () => props.rule,
  (r) => {
    if (r) {
      form.value = {
        serverId: r.serverId,
        localPort: r.localPort,
        remoteHost: r.remoteHost,
        remotePort: r.remotePort,
        description: r.description ?? '',
      }
    } else {
      form.value = {
        serverId: props.serverId,
        localPort: 8080,
        remoteHost: 'localhost',
        remotePort: 8080,
        description: '',
      }
    }
  },
  { immediate: true }
)

function handleClose() {
  emit('update:modelValue', false)
}

function handleSave() {
  if (!form.value.localPort || form.value.localPort < 1 || form.value.localPort > 65535) {
    ElMessage.warning('本地端口无效（1-65535）')
    return
  }
  if (!form.value.remoteHost.trim()) {
    ElMessage.warning('请输入远端主机')
    return
  }
  if (!form.value.remotePort || form.value.remotePort < 1 || form.value.remotePort > 65535) {
    ElMessage.warning('远端端口无效（1-65535）')
    return
  }
  emit('save', { ...form.value, serverId: props.serverId })
  emit('update:modelValue', false)
}
</script>

<template>
  <el-dialog
    :model-value="modelValue"
    :title="rule ? '编辑转发规则' : '添加转发规则'"
    width="440px"
    @close="handleClose"
  >
    <el-form :model="form" label-width="90px" label-position="left">
      <el-form-item label="本地端口" required>
        <el-input-number v-model="form.localPort" :min="1" :max="65535" style="width: 130px" />
        <el-text type="info" size="small" style="margin-left: 8px">本机监听端口</el-text>
      </el-form-item>
      <el-form-item label="远端主机" required>
        <el-input v-model="form.remoteHost" placeholder="localhost" style="width: 200px" />
      </el-form-item>
      <el-form-item label="远端端口" required>
        <el-input-number v-model="form.remotePort" :min="1" :max="65535" style="width: 130px" />
      </el-form-item>
      <el-form-item label="描述">
        <el-input v-model="form.description" placeholder="可选备注" />
      </el-form-item>
    </el-form>

    <template #footer>
      <el-button @click="handleClose">取消</el-button>
      <el-button type="primary" @click="handleSave">保存</el-button>
    </template>
  </el-dialog>
</template>
