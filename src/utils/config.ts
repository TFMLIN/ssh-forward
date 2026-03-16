import { invoke } from '@tauri-apps/api/core'
import {
  exists,
  mkdir,
  readTextFile,
  writeTextFile,
} from '@tauri-apps/plugin-fs'
import type { SshServer, ForwardRule } from '../types'

const CONFIG_FILENAME = 'config.json'

/** 配置数据结构 */
export interface ConfigData {
  servers: SshServer[]
  rules: ForwardRule[]
}

/** 空配置 */
export const emptyConfig: ConfigData = {
  servers: [],
  rules: [],
}

/** 获取配置文件目录路径 */
export async function getConfigDir(): Promise<string> {
  return await invoke<string>('get_config_dir')
}

/** 获取配置文件完整路径 */
export async function getConfigPath(): Promise<string> {
  const dir = await getConfigDir()
  // 处理 Windows 和 Unix 路径分隔符
  const separator = dir.includes('\\') ? '\\' : '/'
  return `${dir}${separator}${CONFIG_FILENAME}`
}

/** 确保配置目录存在 */
async function ensureConfigDir(): Promise<void> {
  const dir = await getConfigDir()
  const dirExists = await exists(dir)
  if (!dirExists) {
    await mkdir(dir, { recursive: true })
  }
}

/** 从文件加载配置 */
export async function loadConfig(): Promise<ConfigData> {
  try {
    const path = await getConfigPath()
    const fileExists = await exists(path)
    if (!fileExists) {
      return { ...emptyConfig }
    }
    const content = await readTextFile(path)
    const data = JSON.parse(content) as Partial<ConfigData>
    return {
      servers: data.servers ?? [],
      rules: data.rules ?? [],
    }
  } catch (e) {
    console.error('加载配置失败:', e)
    return { ...emptyConfig }
  }
}

/** 保存配置到文件 */
export async function saveConfig(data: ConfigData): Promise<void> {
  await ensureConfigDir()
  const path = await getConfigPath()
  const content = JSON.stringify(data, null, 2)
  await writeTextFile(path, content)
}

/** 从指定路径导入配置 */
export async function importConfigFromPath(path: string): Promise<ConfigData> {
  const content = await readTextFile(path)
  const data = JSON.parse(content) as Partial<ConfigData>
  return {
    servers: data.servers ?? [],
    rules: data.rules ?? [],
  }
}

/** 导出配置到指定路径 */
export async function exportConfigToPath(
  path: string,
  data: ConfigData
): Promise<void> {
  const content = JSON.stringify(data, null, 2)
  await writeTextFile(path, content)
}
