<template>
  <div class="download-page x-theme page-container">
    <div class="page-header">
      <div class="header-left">
        <h1 class="page-title">X (Twitter) Downloader</h1>
        <p class="page-subtitle">Tải Video từ X (Twitter) cực nhanh và nét</p>
      </div>
    </div>

    <div class="content-grid-dl">
      <div class="glass-panel input-panel">
        <div class="mode-selector x-mode">
          <button class="mode-btn active">
            <span class="m-icon">𝕏</span>
            <span>X / Twitter URL</span>
          </button>
        </div>

        <div class="url-input-section">
          <label class="premium-label x-label">Tweet URL</label>
          <div class="premium-input-group">
            <input 
              v-model="url" 
              type="text" 
              placeholder="Dán link bài viết có chứa video trên X..."
              class="glow-input x-input"
            />
          </div>
        </div>

        <div class="settings-stack">
          <div class="setting-row">
            <label>Thư mục lưu:</label>
            <div class="folder-picker">
              <input type="text" v-model="outputDir" readonly />
              <button @click="selectDir" class="btn-picker x-picker">Chọn</button>
            </div>
          </div>
          
          <div class="setting-row" style="margin-top: 1.5rem;">
            <label>Chất lượng:</label>
            <select v-model="quality" class="premium-select-dl x-input">
              <option value="best">Cao nhất (Tự động)</option>
              <option value="1080p">1080p (Full HD)</option>
              <option value="720p">720p (HD)</option>
            </select>
          </div>
        </div>

        <div class="actions-group">
          <button 
            v-if="!isDownloading"
            class="btn-download-ultra x-btn" 
            :disabled="!url || !outputDir"
            @click="startDownload"
          >
            🚀 BẮT ĐẦU TẢI TỪ X
          </button>
          <button v-else class="btn-stop-dl" @click="stopDownload">🛑 DỪNG TẢI NGAY</button>
        </div>

        <!-- Pro Tips Panel -->
        <div class="pro-tips-panel">
          <h4 class="tips-title">💡 Mẹo tải từ X:</h4>
          <ul class="tips-list">
            <li>Nên đăng nhập X trên <strong>Chrome</strong> để tải ổn định hơn.</li>
            <li>Tool tự động quét và lấy bản <strong>HD/4K</strong> nét nhất của tweet.</li>
            <li>Nếu bài viết có nhiều video, tool sẽ tải video đầu tiên được tìm thấy.</li>
          </ul>
        </div>
      </div>

      <div class="glass-panel progress-panel">
        <div class="panel-header-dl">
          <h3>Tiến trình X (Twitter)</h3>
          <span :class="statusClass">{{ statusText }}</span>
        </div>

        <div class="log-viewport custom-scrollbar" ref="logContainer">
          <div v-for="(log, i) in logs" :key="i" class="dl-log" :class="log.type">
            <span class="log-time">{{ log.time }}</span>
            <span class="log-msg">{{ log.msg }}</span>
          </div>
          <div v-if="logs.length === 0" class="log-empty">Dán link X để bắt đầu...</div>
        </div>

        <div v-if="isDownloading" class="dl-progress-box">
          <div class="dl-info"><span>{{ progressMsg }}</span><span>{{ progress }}%</span></div>
          <div class="dl-bar-bg"><div class="dl-bar-fill x-bar-fill" :style="{ width: progress + '%' }"></div></div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { listen } from '@tauri-apps/api/event'

const url = ref('')
const outputDir = ref('')
const quality = ref('best')
const isDownloading = ref(false)
const progress = ref(0)
const progressMsg = ref('Đang kết nối...')
const logs = ref([])
const logContainer = ref(null)
let unlistenLog = null

const scrollToBottom = () => { if (logContainer.value) { setTimeout(() => { logContainer.value.scrollTop = logContainer.value.scrollHeight }, 50) } }

const statusText = computed(() => {
  if (isDownloading.value) return 'Đang tải...'
  if (logs.value.some(l => l.type === 'success')) return 'Hoàn tất'
  return 'Sẵn sàng'
})

const statusClass = computed(() => {
  if (isDownloading.value) return 'status-badge processing x-active'
  if (logs.value.some(l => l.type === 'success')) return 'status-badge success'
  return 'status-badge'
})

const addLog = (msg, type = 'info') => {
  logs.value.push({ time: new Date().toLocaleTimeString([], { hour12: false }), msg, type })
  scrollToBottom()
}

const selectDir = async () => {
  const selected = await open({ directory: true })
  if (selected) { outputDir.value = selected; localStorage.setItem('dl_x_dir', selected) }
}

const stopDownload = async () => { try { const result = await invoke('stop_download'); addLog(result, 'warning') } catch (err) { addLog('Lỗi khi dừng: ' + String(err), 'error') } }

const startDownload = async () => {
  if (!url.value || !outputDir.value) return
  isDownloading.value = true; progress.value = 0
  addLog(`Bắt đầu tải từ X...`, 'info')
  try {
    const result = await invoke('download_video', { url: url.value, mode: 'single', quality: quality.value, outputDir: outputDir.value })
    addLog(result, 'success'); progress.value = 100
  } catch (err) {
    if (String(err).includes('process killed')) { addLog('Đã dừng bởi người dùng.', 'warning') } else { addLog('Lỗi X: ' + String(err), 'error') }
  } finally { isDownloading.value = false }
}

onMounted(async () => {
  const saved = localStorage.getItem('dl_x_dir')
  if (saved) outputDir.value = saved
  unlistenLog = await listen('download-log', (event) => {
    const logStr = event.payload; addLog(logStr, 'info')
    const match = logStr.match(/(\d+\.?\d*)%/); if (match) { progress.value = parseFloat(match[1]); progressMsg.value = 'Đang tải từ X...' }
  })
})
onUnmounted(() => { if (unlistenLog) unlistenLog() })
</script>

<style scoped>
.x-theme {
  --primary: #ffffff;
  --bg: #000000;
  --panel: rgba(255, 255, 255, 0.05);
}

.page-container { padding: 1.5rem 2.5rem; height: 100vh; background: var(--bg); color: white; overflow: hidden; display: flex; flex-direction: column; }
.content-grid-dl { display: grid; grid-template-columns: 450px 1fr; gap: 2rem; margin-top: 1.5rem; flex: 1; min-height: 0; }
.glass-panel { background: var(--panel); border: 1px solid rgba(255, 255, 255, 0.1); border-radius: 2rem; padding: 2rem; backdrop-filter: blur(10px); display: flex; flex-direction: column; }
.mode-selector { display: flex; gap: 1rem; margin-bottom: 2rem; background: rgba(255, 255, 255, 0.1); padding: 0.5rem; border-radius: 1.25rem; }
.mode-btn { flex: 1; display: flex; align-items: center; justify-content: center; gap: 0.75rem; padding: 0.9rem; border: none; background: transparent; color: #94a3b8; font-weight: 700; border-radius: 1rem; cursor: pointer; transition: all 0.3s; }
.mode-btn.active { background: white; color: black; }
.premium-label { display: block; font-size: 0.75rem; font-weight: 800; text-transform: uppercase; letter-spacing: 0.1em; margin-bottom: 1rem; }
.glow-input { width: 100%; background: rgba(255, 255, 255, 0.05); border: 1px solid #333; border-radius: 1rem; padding: 1.25rem; color: white; outline: none; transition: all 0.3s; }
.glow-input:focus { border-color: white; }
.settings-stack { margin: 2rem 0; }
.setting-row { display: flex; flex-direction: column; gap: 0.75rem; }
.setting-row label { font-size: 0.85rem; color: #94a3b8; }
.folder-picker { display: flex; gap: 0.75rem; }
.folder-picker input { flex: 1; background: rgba(255, 255, 255, 0.05); border: 1px solid #333; border-radius: 0.75rem; padding: 0.75rem; color: #94a3b8; font-size: 0.8rem; }
.btn-picker { background: #333; border: none; color: white; padding: 0 1.25rem; border-radius: 0.75rem; cursor: pointer; }
.premium-select-dl { width: 100%; background: rgba(255, 255, 255, 0.05); border: 1px solid #333; border-radius: 0.75rem; padding: 0.75rem; color: #e2e8f0; font-size: 0.85rem; outline: none; }
.actions-group { margin-top: auto; }
.btn-download-ultra { width: 100%; padding: 1.5rem; border: none; border-radius: 1.25rem; color: black; font-weight: 900; font-size: 1rem; letter-spacing: 0.1em; cursor: pointer; transition: all 0.3s; }
.x-btn { background: white; }
.btn-stop-dl { width: 100%; padding: 1.5rem; background: #1e293b; border: 1px solid rgba(255, 255, 255, 0.1); border-radius: 1.25rem; color: #ef4444; font-weight: 900; font-size: 1rem; letter-spacing: 0.1em; cursor: pointer; transition: all 0.3s; }
.panel-header-dl { display: flex; justify-content: space-between; align-items: center; margin-bottom: 2rem; }
.status-badge { background: #1e293b; padding: 4px 12px; border-radius: 20px; font-size: 0.7rem; font-weight: 800; text-transform: uppercase; color: #64748b; }
.status-badge.processing { background: rgba(255, 255, 255, 0.1); color: white; }
.status-badge.success { background: rgba(34, 197, 94, 0.1); color: #4ade80; }
.log-viewport { flex: 1; background: rgba(255, 255, 255, 0.02); border-radius: 1.5rem; padding: 1.5rem; overflow-y: auto; display: flex; flex-direction: column; gap: 0.75rem; }
.dl-log { font-family: 'JetBrains Mono', monospace; font-size: 0.8rem; padding-left: 1rem; border-left: 2px solid #333; }
.log-time { opacity: 0.3; margin-right: 1rem; }
.log-empty { text-align: center; margin-top: 5rem; color: #333; font-style: italic; }
.dl-progress-box { margin-top: 2rem; }
.dl-info { display: flex; justify-content: space-between; font-size: 0.85rem; color: #94a3b8; margin-bottom: 0.75rem; }
.dl-bar-bg { height: 8px; background: #1a1a1a; border-radius: 10px; overflow: hidden; }
.dl-bar-fill { height: 100%; transition: width 0.3s; }
.x-bar-fill { background: white; }
.loader-dl { width: 20px; height: 20px; border: 3px solid rgba(0, 0, 0, 0.3); border-top-color: black; border-radius: 50%; animation: spin 0.8s linear infinite; margin: 0 auto; }
@keyframes spin { to { transform: rotate(360deg); } }

.pro-tips-panel {
  margin-top: 2rem;
  padding: 1.25rem;
  background: rgba(255, 255, 255, 0.05);
  border: 1px dashed rgba(255, 255, 255, 0.2);
  border-radius: 1rem;
}

.tips-title {
  font-size: 0.85rem;
  color: #ffffff;
  margin-bottom: 0.75rem;
  font-weight: 800;
}

.tips-list {
  list-style: none;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.tips-list li {
  font-size: 0.75rem;
  color: #94a3b8;
  line-height: 1.4;
  position: relative;
  padding-left: 1rem;
}

.tips-list li::before {
  content: "•";
  position: absolute;
  left: 0;
  color: #ffffff;
}

.custom-scrollbar::-webkit-scrollbar { width: 5px; }
.custom-scrollbar::-webkit-scrollbar-thumb { background: rgba(255, 255, 255, 0.1); border-radius: 10px; }
</style>
