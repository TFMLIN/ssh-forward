<script setup lang="ts">
import { ref, computed } from 'vue'
import { ElMessage } from 'element-plus'
import { Upload } from '@element-plus/icons-vue'
import { useServerStore } from './stores/servers'
import ServerList from './components/ServerList.vue'
import ForwardList from './components/ForwardList.vue'
import ImportSshConfig from './components/ImportSshConfig.vue'
import type { SshServer } from './types'

const store = useServerStore()

const selectedServer = computed(() => store.getSelectedServer())
const showImport = ref(false)

function handleImport(servers: Omit<SshServer, 'id'>[]) {
  let count = 0
  for (const s of servers) {
    store.addServer(s)
    count++
  }
  ElMessage.success(`已导入 ${count} 个服务器`)
}
</script>

<template>
  <div class="app-layout">
    <!-- 顶部栏 -->
    <header class="app-header">
      <span class="app-title">SSH 端口转发管理器</span>
      <el-button
        size="small"
        :icon="Upload"
        @click="showImport = true"
        title="从 ~/.ssh/config 导入"
      >
        导入 SSH Config
      </el-button>
    </header>

    <!-- 主体 -->
    <div class="app-body">
      <!-- 左侧服务器列表 -->
      <aside class="sidebar">
        <ServerList />
      </aside>

      <!-- 右侧转发规则区 -->
      <main class="main-content">
        <ForwardList v-if="selectedServer" :server="selectedServer" />
        <div v-else class="no-server">
          <el-empty description="请在左侧选择一个 SSH 服务器" :image-size="100" />
        </div>
      </main>
    </div>

    <!-- 导入对话框 -->
    <ImportSshConfig v-model="showImport" @import="handleImport" />
  </div>
</template>

<style>
* {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
}

html,
body,
#app {
  height: 100%;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
  font-size: 14px;
  background-color: var(--el-bg-color);
  color: var(--el-text-color-primary);
}
</style>

<style scoped>
.app-layout {
  height: 100vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.app-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 16px;
  height: 46px;
  border-bottom: 1px solid var(--el-border-color);
  background-color: var(--el-bg-color);
  flex-shrink: 0;
}

.app-title {
  font-weight: 700;
  font-size: 15px;
  color: var(--el-text-color-primary);
}

.app-body {
  display: flex;
  flex: 1;
  overflow: hidden;
}

.sidebar {
  width: 220px;
  flex-shrink: 0;
  border-right: 1px solid var(--el-border-color-light);
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.main-content {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.no-server {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}
</style>
