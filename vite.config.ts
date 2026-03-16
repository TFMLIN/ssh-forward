import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(async () => ({
  plugins: [vue()],

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 5173,
    strictPort: false,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },

  // 优化构建输出，代码分割减少主包体积
  build: {
    rollupOptions: {
      output: {
        manualChunks: (id) => {
          // 将 node_modules 中的依赖拆分成独立的 chunks
          if (id.includes("node_modules")) {
            // Element Plus 组件库
            if (id.includes("element-plus")) {
              return "vendor-element";
            }
            // Element Plus 图标
            if (id.includes("@element-plus/icons-vue")) {
              return "vendor-icons";
            }
            // Tauri 相关 API
            if (id.includes("@tauri-apps")) {
              return "vendor-tauri";
            }
            // Pinia 状态管理
            if (id.includes("pinia")) {
              return "vendor-pinia";
            }
            // Vue 核心
            if (id.includes("vue")) {
              return "vendor-vue";
            }
            // 其他第三方库
            return "vendor";
          }
        },
      },
    },
  },
}));
