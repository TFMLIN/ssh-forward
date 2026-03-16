<script setup lang="ts">
import { ref, watch } from 'vue'
import { getToast } from '../utils/toast'
import type { ForwardRule } from '../types'

const toast = getToast()

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
    toast.warning('本地端口无效（1-65535）')
    return
  }
  if (!form.value.remoteHost.trim()) {
    toast.warning('请输入远端主机')
    return
  }
  if (!form.value.remotePort || form.value.remotePort < 1 || form.value.remotePort > 65535) {
    toast.warning('远端端口无效（1-65535）')
    return
  }
  emit('save', { ...form.value, serverId: props.serverId })
  emit('update:modelValue', false)
}
</script>

<template>
  <dialog class="modal" :class="{ 'modal-open': modelValue }">
    <div class="modal-box w-[440px] max-w-[90vw]">
      <h3 class="font-bold text-lg mb-4">{{ rule ? '编辑转发规则' : '添加转发规则' }}</h3>

      <form @submit.prevent="handleSave">
        <!-- 本地端口 -->
        <div class="form-control mb-3">
          <label class="label py-1">
            <span class="label-text">本地端口 <span class="text-error">*</span></span>
          </label>
          <div class="flex items-center gap-2">
            <input
              v-model.number="form.localPort"
              type="number"
              min="1"
              max="65535"
              class="input input-bordered input-sm w-32"
              placeholder="8080"
            />
            <span class="text-xs text-gray-400">本机监听端口</span>
          </div>
        </div>

        <!-- 远端主机 -->
        <div class="form-control mb-3">
          <label class="label py-1">
            <span class="label-text">远端主机 <span class="text-error">*</span></span>
          </label>
          <input
            v-model="form.remoteHost"
            type="text"
            class="input input-bordered input-sm w-52"
            placeholder="localhost"
          />
        </div>

        <!-- 远端端口 -->
        <div class="form-control mb-3">
          <label class="label py-1">
            <span class="label-text">远端端口 <span class="text-error">*</span></span>
          </label>
          <input
            v-model.number="form.remotePort"
            type="number"
            min="1"
            max="65535"
            class="input input-bordered input-sm w-32"
            placeholder="8080"
          />
        </div>

        <!-- 描述 -->
        <div class="form-control mb-4">
          <label class="label py-1">
            <span class="label-text">描述</span>
          </label>
          <input
            v-model="form.description"
            type="text"
            class="input input-bordered input-sm"
            placeholder="可选备注"
          />
        </div>
      </form>

      <div class="modal-action">
        <button type="button" class="btn btn-sm" @click="handleClose">取消</button>
        <button type="button" class="btn btn-sm btn-primary" @click="handleSave">保存</button>
      </div>
    </div>
    <div class="modal-backdrop bg-black/30" @click="handleClose"></div>
  </dialog>
</template>
