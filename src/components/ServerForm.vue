<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { Plus, Trash2, ArrowUp, ArrowDown, FolderOpen } from 'lucide-vue-next'
import { open } from '@tauri-apps/plugin-dialog'
import { useServerStore } from '../stores/servers'
import { getToast } from '../utils/toast'
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
const toast = getToast()

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

function handleClose() {
  emit('update:modelValue', false)
}

function handleSave() {
  if (!form.value.name.trim()) {
    toast.warning('请输入服务器名称')
    return
  }
  if (!form.value.host.trim()) {
    toast.warning('请输入主机地址')
    return
  }
  if (!form.value.username.trim()) {
    toast.warning('请输入用户名')
    return
  }
  for (const entry of form.value.jumpEntries ?? []) {
    if (entry.type === 'inline') {
      if (!entry.inline?.host?.trim()) {
        toast.warning('跳板机主机地址不能为空')
        return
      }
      if (!entry.inline?.username?.trim()) {
        toast.warning('跳板机用户名不能为空')
        return
      }
    }
  }
  emit('save', { ...form.value })
  emit('update:modelValue', false)
}
</script>

<template>
  <div v-if="modelValue" class="modal modal-open">
    <div class="modal-box w-11/12 max-w-2xl">
      <h3 class="font-bold text-lg mb-4">{{ server ? '编辑服务器' : '添加服务器' }}</h3>
      
      <div class="max-h-[70vh] overflow-y-auto pr-2">
        <!-- 基本信息 -->
        <div class="space-y-4">
          <div class="form-control">
            <label class="label">
              <span class="label-text">名称 <span class="text-red-500">*</span></span>
            </label>
            <input v-model="form.name" type="text" placeholder="如：生产服务器" class="input input-bordered w-full" />
          </div>
          
          <div class="form-control">
            <label class="label">
              <span class="label-text">主机 <span class="text-red-500">*</span></span>
            </label>
            <input v-model="form.host" type="text" placeholder="如：192.168.1.100 或 example.com" class="input input-bordered w-full" />
          </div>
          
          <div class="form-control">
            <label class="label">
              <span class="label-text">端口</span>
            </label>
            <input v-model.number="form.port" type="number" min="1" max="65535" class="input input-bordered w-32" />
          </div>
          
          <div class="form-control">
            <label class="label">
              <span class="label-text">用户名 <span class="text-red-500">*</span></span>
            </label>
            <input v-model="form.username" type="text" placeholder="如：root" class="input input-bordered w-full" />
          </div>
          
          <div class="form-control">
            <label class="label">
              <span class="label-text">认证方式</span>
            </label>
            <div class="flex gap-4">
              <label class="label cursor-pointer gap-2">
                <input v-model="form.authType" type="radio" value="password" class="radio" />
                <span class="label-text">密码</span>
              </label>
              <label class="label cursor-pointer gap-2">
                <input v-model="form.authType" type="radio" value="privateKey" class="radio" />
                <span class="label-text">私钥</span>
              </label>
              <label class="label cursor-pointer gap-2">
                <input v-model="form.authType" type="radio" value="agent" class="radio" />
                <span class="label-text">SSH Agent</span>
              </label>
            </div>
          </div>
          
          <div v-if="form.authType === 'password'" class="form-control">
            <label class="label">
              <span class="label-text">密码</span>
            </label>
            <input v-model="form.password" type="password" placeholder="SSH 密码" class="input input-bordered w-full" />
          </div>
          
          <template v-if="form.authType === 'privateKey'">
            <div class="form-control">
              <label class="label">
                <span class="label-text">私钥路径</span>
              </label>
              <div class="flex gap-2">
                <input v-model="form.privateKeyPath" type="text" placeholder="如：~/.ssh/id_rsa" class="input input-bordered flex-1" />
                <button class="btn btn-square" @click="selectPrivateKey('main')">
                  <FolderOpen class="w-5 h-5" />
                </button>
              </div>
            </div>
            <div class="form-control">
              <label class="label">
                <span class="label-text">私钥密码</span>
              </label>
              <input v-model="form.passphrase" type="password" placeholder="私钥密码（如有）" class="input input-bordered w-full" />
            </div>
          </template>
          
          <div v-if="form.authType === 'agent'" class="text-sm text-gray-500">
            将使用系统 SSH Agent（需设置 SSH_AUTH_SOCK）
          </div>
        </div>
        
        <!-- 分隔线 -->
        <div class="divider text-sm text-gray-500">跳板机（ProxyJump）</div>
        
        <div v-if="(form.jumpEntries ?? []).length === 0" class="text-sm text-gray-500 mb-4">
          无跳板机，直接连接目标服务器
        </div>
        
        <div v-for="(entry, idx) in form.jumpEntries ?? []" :key="idx" class="border rounded-lg p-3 mb-3 bg-gray-50">
          <div class="flex items-center justify-between mb-3">
            <span class="text-sm font-medium text-blue-600">跳板机 {{ idx + 1 }}</span>
            <div class="flex gap-1">
              <button class="btn btn-xs btn-ghost" :disabled="idx === 0" @click="moveJumpUp(idx)" title="上移">
                <ArrowUp class="w-4 h-4" />
              </button>
              <button class="btn btn-xs btn-ghost" :disabled="idx === (form.jumpEntries ?? []).length - 1" @click="moveJumpDown(idx)" title="下移">
                <ArrowDown class="w-4 h-4" />
              </button>
              <button class="btn btn-xs btn-ghost text-red-600" @click="removeJumpEntry(idx)" title="删除">
                <Trash2 class="w-4 h-4" />
              </button>
            </div>
          </div>
          
          <div class="space-y-3">
            <div class="form-control">
              <label class="label">
                <span class="label-text">配置方式</span>
              </label>
              <div class="flex gap-4">
                <label class="label cursor-pointer gap-2">
                  <input v-model="entry.type" type="radio" value="server" :disabled="availableJumpServers.length === 0" @change="onEntryTypeChange(entry)" class="radio" />
                  <span class="label-text">选择服务器</span>
                </label>
                <label class="label cursor-pointer gap-2">
                  <input v-model="entry.type" type="radio" value="inline" @change="onEntryTypeChange(entry)" class="radio" />
                  <span class="label-text">手动输入</span>
                </label>
              </div>
            </div>
            
            <!-- 引用已有服务器 -->
            <div v-if="entry.type === 'server'" class="form-control">
              <label class="label">
                <span class="label-text">服务器</span>
              </label>
              <select v-model="entry.serverId" class="select select-bordered w-full">
                <option v-for="s in availableJumpServers" :key="s.id" :value="s.id">
                  {{ s.name }} ({{ s.username }}@{{ s.host }}:{{ s.port }})
                </option>
              </select>
            </div>
            
            <!-- 内联配置 -->
            <template v-if="entry.type === 'inline'">
              <div class="form-control">
                <label class="label">
                  <span class="label-text">主机 <span class="text-red-500">*</span></span>
                </label>
                <input v-model="getInline(entry).host" type="text" placeholder="跳板机地址" class="input input-bordered w-full" />
              </div>
              
              <div class="form-control">
                <label class="label">
                  <span class="label-text">端口</span>
                </label>
                <input v-model.number="getInline(entry).port" type="number" min="1" max="65535" class="input input-bordered w-24" />
              </div>
              
              <div class="form-control">
                <label class="label">
                  <span class="label-text">用户名 <span class="text-red-500">*</span></span>
                </label>
                <input v-model="getInline(entry).username" type="text" placeholder="跳板机用户名" class="input input-bordered w-full" />
              </div>
              
              <div class="form-control">
                <label class="label">
                  <span class="label-text">认证方式</span>
                </label>
                <div class="flex gap-4">
                  <label class="label cursor-pointer gap-2">
                    <input v-model="getInline(entry).authType" type="radio" value="password" class="radio" />
                    <span class="label-text">密码</span>
                  </label>
                  <label class="label cursor-pointer gap-2">
                    <input v-model="getInline(entry).authType" type="radio" value="privateKey" class="radio" />
                    <span class="label-text">私钥</span>
                  </label>
                  <label class="label cursor-pointer gap-2">
                    <input v-model="getInline(entry).authType" type="radio" value="agent" class="radio" />
                    <span class="label-text">Agent</span>
                  </label>
                </div>
              </div>
              
              <div v-if="getInline(entry).authType === 'password'" class="form-control">
                <label class="label">
                  <span class="label-text">密码</span>
                </label>
                <input v-model="getInline(entry).password" type="password" placeholder="密码" class="input input-bordered w-full" />
              </div>
              
              <template v-if="getInline(entry).authType === 'privateKey'">
                <div class="form-control">
                  <label class="label">
                    <span class="label-text">私钥路径</span>
                  </label>
                  <div class="flex gap-2">
                    <input v-model="getInline(entry).privateKeyPath" type="text" placeholder="如：~/.ssh/id_rsa" class="input input-bordered flex-1" />
                    <button class="btn btn-square" @click="selectPrivateKey(idx)">
                      <FolderOpen class="w-5 h-5" />
                    </button>
                  </div>
                </div>
                <div class="form-control">
                  <label class="label">
                    <span class="label-text">私钥密码</span>
                  </label>
                  <input v-model="getInline(entry).passphrase" type="password" placeholder="私钥密码（如有）" class="input input-bordered w-full" />
                </div>
              </template>
            </template>
          </div>
        </div>
        
        <button class="btn btn-sm btn-outline gap-1 mt-2" @click="addJumpEntry">
          <Plus class="w-4 h-4" />
          添加跳板机
        </button>
      </div>
      
      <div class="modal-action">
        <button class="btn" @click="handleClose">取消</button>
        <button class="btn btn-primary" @click="handleSave">保存</button>
      </div>
    </div>
    <div class="modal-backdrop" @click="handleClose"></div>
  </div>
</template>