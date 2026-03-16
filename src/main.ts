import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import { useServerStore } from './stores/servers'
import { initToast } from './utils/toast'
import './style.css'

const app = createApp(App)

// Pinia
const pinia = createPinia()
app.use(pinia)

// Initialize toast
initToast(app)

// 挂载应用
app.mount('#app')

// 启动后从文件加载配置
const store = useServerStore()
store.loadFromFile()
