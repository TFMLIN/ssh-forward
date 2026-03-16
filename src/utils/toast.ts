import type { App } from 'vue'
import { reactive } from 'vue'

interface ToastItem {
  id: string
  message: string
  type: 'success' | 'error' | 'info' | 'warning'
}

const toasts = reactive<ToastItem[]>([])

export function useToast() {
  const add = (message: string, type: ToastItem['type'] = 'info', duration = 3000) => {
    const id = Math.random().toString(36).substring(7)
    toasts.push({ id, message, type })
    
    if (duration > 0) {
      setTimeout(() => remove(id), duration)
    }
  }
  
  const remove = (id: string) => {
    const index = toasts.findIndex(t => t.id === id)
    if (index > -1) {
      toasts.splice(index, 1)
    }
  }
  
  return {
    success: (msg: string, duration?: number) => add(msg, 'success', duration),
    error: (msg: string, duration?: number) => add(msg, 'error', duration),
    info: (msg: string, duration?: number) => add(msg, 'info', duration),
    warning: (msg: string, duration?: number) => add(msg, 'warning', duration),
    toasts,
    remove,
  }
}

// Global toast instance
let globalToast: ReturnType<typeof useToast> | null = null

export function initToast(app: App) {
  globalToast = useToast()
  app.config.globalProperties.$toast = globalToast
}

export function getToast() {
  if (!globalToast) {
    globalToast = useToast()
  }
  return globalToast
}
