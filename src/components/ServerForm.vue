<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { ElMessage } from 'element-plus'
import { Plus, Delete, ArrowUp, ArrowDown, FolderOpened } from '@element-plus/icons-vue'
import { open } from '@tauri-apps/plugin-dialog'
import { useServerStore } from '../stores/servers'
import type { SshServer, JumpEntry, JumpHost } from '../types'

const props = defineProps<{
  modelValue: boolean
  server?: SshServer | null
}>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  save: [server: Omit<SshServer, 'id'>]
}>()

const store = useServerStore()

// 可选的跳板机服务器列表（排除当前服务器自身）
const availableJumpServers = computed(() =>
  store.servers.filter((s) => s.id !== props.server?.id)
)

const emptyForm = (): Omit<SshServer, 'id'> => ({
  name: '',
  host: '',
  port: 22,
  username: '',
  authType: 'password',
  password: '',
  privateKeyPath: '',
  passphrase: '',
  jumpEntries: [],
})

const form = ref<Omit<SshServer, 'id'>>(emptyForm())

watch(
  () => props.server,
  (s) => {
    if (s) {
      form.value = {
        name: s.name,
        host: s.host,
        port: s.port,
        username: s.username,
        authType: s.authType,
        password: s.password ?? '',
        privateKeyPath: s.privateKeyPath ?? '',
        passphrase: s.passphrase ?? '',
        jumpEntries: s.jumpEntries ? JSON.parse(JSON.stringify(s.jumpEntries)) : [],
      }
    } else {
      form.value = emptyForm()
    }
  },
  { immediate: true }
)

// --- 跳板机管理 ---

function addJumpEntry() {
  form.value.jumpEntries = form.value.jumpEntries ?? []
  form.value.jumpEntries.push({
    type: 'server',
    serverId: availableJumpServers.value[0]?.id ?? undefined,
    inline: undefined,
  })
}

function removeJumpEntry(idx: number) {
  form.value.jumpEntries?.splice(idx, 1)
}

function moveJumpUp(idx: number) {
  const entries = form.value.jumpEntries
  if (!entries || idx === 0) return
  ;[entries[idx - 1], entries[idx]] = [entries[idx], entries[idx - 1]]
}

function moveJumpDown(idx: number) {
  const entries = form.value.jumpEntries
  if (!entries || idx === entries.length - 1) return
  ;[entries[idx], entries[idx + 1]] = [entries[idx + 1], entries[idx]]
}

function getInline(entry: JumpEntry): JumpHost {
  if (!entry.inline) {
    entry.inline = {
      host: '',
      port: 22,
      username: '',
      authType: 'password',
    }
  }
  return entry.inline
}

function onEntryTypeChange(entry: JumpEntry) {
  if (entry.type === 'server') {
    entry.serverId = availableJumpServers.value[0]?.id ?? undefined
    entry.inline = undefined
  } else {
    entry.serverId = undefined
    entry.inline = {
      host: '',
      port: 22,
      username: '',
      authType: 'password',
    }
  }
}

// --- 选择私钥文件 ---

async function selectPrivateKey(target: 'main' | number) {
  const selected = await open({
    title: '选择私钥文件',
    multiple: false,
    directory: false,
  })
  if (selected) {
    if (target === 'main') {
      form.value.privateKeyPath = selected as string
    } else {
      const entry = form.value.jumpEntries?.[target]
      if (entry?.inline) {
        entry.inline.privateKeyPath = selected as string
      }
    }
  }
}

// --- 保存 ---

function handleClose() {
  emit('update:modelValue', false)
}

function handleSave() {
  if (!form.value.name.trim()) {
    ElMessage.warning('请输入服务器名称')
    return
  }
  if (!form.value.host.trim()) {
    ElMessage.warning('请输入主机地址')
    return
  }
  if (!form.value.username.trim()) {
    ElMessage.warning('请输入用户名')
    return
  }
  // 验证内联跳板机配置
  for (const entry of form.value.jumpEntries ?? []) {
    if (entry.type === 'inline') {
      if (!entry.inline?.host?.trim()) {
        ElMessage.warning('跳板机主机地址不能为空')
        return
      }
      if (!entry.inline?.username?.trim()) {
        ElMessage.warning('跳板机用户名不能为空')
        return
      }
    }
  }
  emit('save', { ...form.value })
  emit('update:modelValue', false)
}
</script>

<template>
  <el-dialog
    :model-value="modelValue"
    :title="server ? '编辑服务器' : '添加服务器'"
    width="560px"
    @close="handleClose"
  >
    <el-scrollbar max-height="70vh">
      <el-form :model="form" label-width="90px" label-position="left" style="padding-right: 8px">
        <!-- 基本信息 -->
        <el-form-item label="名称" required>
          <el-input v-model="form.name" placeholder="如：生产服务器" />
        </el-form-item>
        <el-form-item label="主机" required>
          <el-input v-model="form.host" placeholder="如：192.168.1.100 或 example.com" />
        </el-form-item>
        <el-form-item label="端口">
          <el-input-number v-model="form.port" :min="1" :max="65535" style="width: 120px" />
        </el-form-item>
        <el-form-item label="用户名" required>
          <el-input v-model="form.username" placeholder="如：root" />
        </el-form-item>
        <el-form-item label="认证方式">
          <el-radio-group v-model="form.authType">
            <el-radio value="password">密码</el-radio>
            <el-radio value="privateKey">私钥</el-radio>
            <el-radio value="agent">SSH Agent</el-radio>
          </el-radio-group>
        </el-form-item>

        <template v-if="form.authType === 'password'">
          <el-form-item label="密码">
            <el-input v-model="form.password" type="password" show-password placeholder="SSH 密码" />
          </el-form-item>
        </template>

        <template v-if="form.authType === 'privateKey'">
          <el-form-item label="私钥路径">
            <el-input v-model="form.privateKeyPath" placeholder="如：~/.ssh/id_rsa">
              <template #append>
                <el-button :icon="FolderOpened" @click="selectPrivateKey('main')" />
              </template>
            </el-input>
          </el-form-item>
          <el-form-item label="私钥密码">
            <el-input v-model="form.passphrase" type="password" show-password placeholder="私钥密码（如有）" />
          </el-form-item>
        </template>

        <template v-if="form.authType === 'agent'">
          <el-form-item>
            <el-text type="info">将使用系统 SSH Agent（需设置 SSH_AUTH_SOCK）</el-text>
          </el-form-item>
        </template>

        <!-- 跳板机配置 -->
        <el-divider content-position="left">
          <el-text size="small" type="info">跳板机（ProxyJump）</el-text>
        </el-divider>

        <div v-if="(form.jumpEntries ?? []).length === 0" class="jump-empty">
          <el-text type="info" size="small">无跳板机，直接连接目标服务器</el-text>
        </div>

        <div
          v-for="(entry, idx) in form.jumpEntries ?? []"
          :key="idx"
          class="jump-entry"
        >
          <div class="jump-entry-header">
            <el-text size="small" type="primary">跳板机 {{ idx + 1 }}</el-text>
            <div class="jump-entry-actions">
              <el-button
                :icon="ArrowUp"
                size="small"
                text
                :disabled="idx === 0"
                @click="moveJumpUp(idx)"
                title="上移"
              />
              <el-button
                :icon="ArrowDown"
                size="small"
                text
                :disabled="idx === (form.jumpEntries ?? []).length - 1"
                @click="moveJumpDown(idx)"
                title="下移"
              />
              <el-button
                :icon="Delete"
                size="small"
                text
                type="danger"
                @click="removeJumpEntry(idx)"
                title="删除"
              />
            </div>
          </div>

          <el-form :model="entry" label-width="80px" label-position="left" class="jump-form">
            <el-form-item label="配置方式">
              <el-radio-group v-model="entry.type" @change="onEntryTypeChange(entry)">
                <el-radio value="server" :disabled="availableJumpServers.length === 0">
                  选择服务器
                </el-radio>
                <el-radio value="inline">手动输入</el-radio>
              </el-radio-group>
            </el-form-item>

            <!-- 引用已有服务器 -->
            <template v-if="entry.type === 'server'">
              <el-form-item label="服务器">
                <el-select v-model="entry.serverId" placeholder="选择跳板机" style="width: 100%">
                  <el-option
                    v-for="s in availableJumpServers"
                    :key="s.id"
                    :label="`${s.name} (${s.username}@${s.host}:${s.port})`"
                    :value="s.id"
                  />
                </el-select>
              </el-form-item>
            </template>

            <!-- 内联配置 -->
            <template v-if="entry.type === 'inline'">
              <el-form-item label="主机" required>
                <el-input v-model="getInline(entry).host" placeholder="跳板机地址" />
              </el-form-item>
              <el-form-item label="端口">
                <el-input-number v-model="getInline(entry).port" :min="1" :max="65535" style="width: 100px" />
              </el-form-item>
              <el-form-item label="用户名" required>
                <el-input v-model="getInline(entry).username" placeholder="跳板机用户名" />
              </el-form-item>
              <el-form-item label="认证方式">
                <el-radio-group v-model="getInline(entry).authType">
                  <el-radio value="password">密码</el-radio>
                  <el-radio value="privateKey">私钥</el-radio>
                  <el-radio value="agent">Agent</el-radio>
                </el-radio-group>
              </el-form-item>
              <template v-if="getInline(entry).authType === 'password'">
                <el-form-item label="密码">
                  <el-input v-model="getInline(entry).password" type="password" show-password placeholder="密码" />
                </el-form-item>
              </template>
              <template v-if="getInline(entry).authType === 'privateKey'">
                <el-form-item label="私钥路径">
                  <el-input v-model="getInline(entry).privateKeyPath" placeholder="如：~/.ssh/id_rsa">
                    <template #append>
                      <el-button :icon="FolderOpened" @click="selectPrivateKey(idx)" />
                    </template>
                  </el-input>
                </el-form-item>
                <el-form-item label="私钥密码">
                  <el-input v-model="getInline(entry).passphrase" type="password" show-password placeholder="私钥密码（如有）" />
                </el-form-item>
              </template>
            </template>
          </el-form>
        </div>

        <el-form-item>
          <el-button :icon="Plus" size="small" @click="addJumpEntry">添加跳板机</el-button>
        </el-form-item>
      </el-form>
    </el-scrollbar>

    <template #footer>
      <el-button @click="handleClose">取消</el-button>
      <el-button type="primary" @click="handleSave">保存</el-button>
    </template>
  </el-dialog>
</template>

<style scoped>
.jump-empty {
  padding: 8px 0 12px 90px;
}

.jump-entry {
  border: 1px solid var(--el-border-color-light);
  border-radius: 6px;
  padding: 10px 12px;
  margin-bottom: 10px;
  background-color: var(--el-fill-color-lighter);
}

.jump-entry-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}

.jump-entry-actions {
  display: flex;
  gap: 2px;
}

.jump-form {
  margin-bottom: 0;
}

.jump-form :deep(.el-form-item) {
  margin-bottom: 10px;
}
</style>
