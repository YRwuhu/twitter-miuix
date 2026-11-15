import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

// Tauri 在开发时使用固定端口，避免窗口加载空白页。
export default defineConfig({
  plugins: [vue()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: '127.0.0.1',
    // Rust 构建目录有大量二进制且会被链接器锁定，交给 Vite 监听会触发
    // EBUSY 并让 dev server 退出。
    watch: {
      ignored: ['**/src-tauri/**'],
    },
  },
  envPrefix: ['VITE_', 'TAURI_'],
  build: {
    target: 'chrome110',
    minify: 'esbuild',
    sourcemap: false,
  },
})
