<script setup lang="ts">
import { ref } from 'vue'
import type { ConfigData } from '../utils/config'

defineProps<{
  modelValue: boolean
  configData: ConfigData | null
}>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  import: [mode: 'merge' | 'replace']
}>()

const importMode = ref<'merge' | 'replace'>('merge')

function handleClose() {
  emit('update:modelValue', false)
}

function handleImport() {
  emit('import', importMode.value)
  emit('update:modelValue', false)
}
</script>

<template>
  <dialog class="modal" :class="{ 'modal-open': modelValue }">
    <div class="modal-box w-[400px] max-w-[90vw]">
      <h3 class="font-bold text-lg mb-4">导入配置</h3>

      <div v-if="configData" class="mb-4">
        <p class="text-sm text-gray-600 mb-4">
          发现 <span class="font-medium text-gray-900">{{ configData.servers.length }}</span> 个服务器和
          <span class="font-medium text-gray-900">{{ configData.rules.length }}</span> 条转发规则
        </p>

        <p class="text-sm text-gray-600 mb-3">请选择导入方式：</p>

        <div class="space-y-2">
          <label class="flex items-start gap-3 cursor-pointer p-2 rounded hover:bg-gray-50">
            <input
              v-model="importMode"
              type="radio"
              value="merge"
              class="radio radio-sm mt-0.5"
            />
            <div>
              <div class="font-medium text-sm">合并</div>
              <div class="text-xs text-gray-500">保留现有配置，添加新项目（按名称去重）</div>
            </div>
          </label>

          <label class="flex items-start gap-3 cursor-pointer p-2 rounded hover:bg-gray-50">
            <input
              v-model="importMode"
              type="radio"
              value="replace"
              class="radio radio-sm mt-0.5"
            />
            <div>
              <div class="font-medium text-sm">覆盖</div>
              <div class="text-xs text-gray-500">清空现有配置，使用导入数据</div>
            </div>
          </label>
        </div>
      </div>

      <div class="modal-action">
        <button class="btn btn-sm" @click="handleClose">取消</button>
        <button class="btn btn-sm btn-primary" @click="handleImport">确认导入</button>
      </div>
    </div>
    <div class="modal-backdrop bg-black/30" @click="handleClose"></div>
  </dialog>
</template>
