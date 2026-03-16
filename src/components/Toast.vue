<script setup lang="ts">
import { X, CheckCircle, AlertCircle, Info, AlertTriangle } from 'lucide-vue-next'
import { getToast } from '../utils/toast'

const { toasts, remove } = getToast()

const toastIcons = {
  success: CheckCircle,
  error: AlertCircle,
  info: Info,
  warning: AlertTriangle,
}

const toastClasses = {
  success: 'bg-green-50 border-green-200 text-green-800',
  error: 'bg-red-50 border-red-200 text-red-800',
  info: 'bg-blue-50 border-blue-200 text-blue-800',
  warning: 'bg-yellow-50 border-yellow-200 text-yellow-800',
}
</script>

<template>
  <teleport to="body">
    <div class="fixed top-4 right-4 z-50 flex flex-col gap-2">
      <TransitionGroup enter-active-class="toast-enter" leave-active-class="toast-leave">
        <div
          v-for="toast in toasts"
          :key="toast.id"
          class="flex items-center gap-2 px-4 py-3 rounded border shadow-sm min-w-[300px]"
          :class="toastClasses[toast.type]"
        >
          <component :is="toastIcons[toast.type]" class="w-5 h-5 flex-shrink-0" />
          <span class="flex-1 text-sm">{{ toast.message }}</span>
          <button
            @click="remove(toast.id)"
            class="p-1 hover:bg-black/5 rounded transition-colors"
          >
            <X class="w-4 h-4" />
          </button>
        </div>
      </TransitionGroup>
    </div>
  </teleport>
</template>