<template>
  <div class="download-page instagram-theme page-container">
    <div class="page-header">
      <div class="header-left">
        <h1 class="page-title">Instagram Downloader</h1>
        <p class="page-subtitle">Tải Stories, Reels và Video Instagram chất lượng cao</p>
      </div>
    </div>

    <div class="content-grid-dl">
      <div class="glass-panel input-panel">
        <div class="mode-selector ig-mode">
          <button class="mode-btn active">
            <span class="m-icon">📸</span>
            <span>Instagram URL</span>
          </button>
        </div>

        <div class="url-input-section">
          <label class="premium-label ig-label">Instagram URL</label>
          <div class="premium-input-group">
            <input 
              v-model="url" 
              type="text" 
              placeholder="Dán link Reels, Story hoặc Video Instagram..."
              class="glow-input ig-input"
            />
          </div>
        </div>

        <div class="settings-stack">
          <div class="setting-row">
            <label>Thư mục lưu:</label>
            <div class="folder-picker">
              <input type="text" v-model="outputDir" readonly />
              <button @click="selectDir" class="btn-picker ig-picker">Chọn</button>
            </div>
          </div>
          
          <div class="setting-row" style="margin-top: 1.5rem;">
            <label>Chất lượng:</label>
            <select v-model="quality" class="premium-select-dl ig-input">
              <option value="best">Cao nhất (Tự động)</option>
              <option value="1080p">1080p (Full HD)</option>
              <option value="720p">720p (HD)</option>
            </select>
          </div>
        </div>

        <div class="actions-group">
          <button 
            v-if="!isDownloading"
            class="btn-download-ultra ig-btn" 
            :disabled="!url || !outputDir"
            @click="startDownload"
          >
            🚀 BẮT ĐẦU TẢI INSTAGRAM
          </button>
          <button v-else class="btn-stop-dl" @click="stopDownload">🛑 DỪNG TẢI NGAY</button>
        </div>

        <!-- Pro Tips Panel -->
        <div class="pro-tips-panel">
          <h4 class="tips-title">💡 Mẹo tải Instagram:</h4>
          <ul class="tips-list">
            <li>Bắt buộc đăng nhập IG trên <strong>Chrome</strong> để tải được Stories và Reels.</li>
            <li>Link <strong>Stories</strong> thường hết hạn nhanh, hãy tải ngay khi có link.</li>
            <li>Tool tự động lấy bản gốc nét nhất từ máy chủ Instagram.</li>
          </ul>
        </div>
      </div>

      <div class="glass-panel progress-panel">
        <div class="panel-header-dl">
          <h3>Tiến trình Instagram</h3>
          <span :class="statusClass">{{ statusText }}</span>
        </div>

        <div class="log-viewport custom-scrollbar" ref="logContainer">
          <div v-for="(log, i) in logs" :key="i" class="dl-log" :class="log.type">
            <span class="log-time">{{ log.time }}</span>
            <span class="log-msg">{{ log.msg }}</span>
          </div>
          <div v-if="logs.length === 0" class="log-empty">Dán link Instagram để bắt đầu...</div>
        </div>

        <div v-if="isDownloading" class="dl-progress-box">
          <div class="dl-info"><span>{{ progressMsg }}</span><span>{{ progress }}%</span></div>
          <div class="dl-bar-bg"><div class="dl-bar-fill ig-bar-fill" :style="{ width: progress + '%' }"></div></div>
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
  if (isDownloading.value) return 'status-badge processing'
  if (logs.value.some(l => l.type === 'success')) return 'status-badge success'
  return 'status-badge'
})

const addLog = (msg, type = 'info') => {
  logs.value.push({ time: new Date().toLocaleTimeString([], { hour12: false }), msg, type })
  scrollToBottom()
}

const selectDir = async () => {
  const selected = await open({ directory: true })
  if (selected) { outputDir.value = selected; localStorage.setItem('dl_instagram_dir', selected) }
}

const stopDownload = async () => { try { const result = await invoke('stop_download'); addLog(result, 'warning') } catch (err) { addLog('Lỗi khi dừng: ' + String(err), 'error') } }

const startDownload = async () => {
  if (!url.value || !outputDir.value) return
  isDownloading.value = true; progress.value = 0
  addLog(`Bắt đầu tải Instagram...`, 'info')
  try {
    const result = await invoke('download_video', { url: url.value, mode: 'single', quality: quality.value, outputDir: outputDir.value })
    addLog(result, 'success'); progress.value = 100
  } catch (err) {
    if (String(err).includes('process killed')) { addLog('Đã dừng bởi người dùng.', 'warning') } else { addLog('Lỗi Instagram: ' + String(err), 'error') }
  } finally { isDownloading.value = false }
}

onMounted(async () => {
  const saved = localStorage.getItem('dl_instagram_dir')
  if (saved) outputDir.value = saved
  unlistenLog = await listen('download-log', (event) => {
    const logStr = event.payload; addLog(logStr, 'info')
    const match = logStr.match(/(\d+\.?\d*)%/); if (match) { progress.value = parseFloat(match[1]); progressMsg.value = 'Đang tải Instagram...' }
  })
})
onUnmounted(() => { if (unlistenLog) unlistenLog() })
</script>

<style scoped>
.instagram-theme {
  --primary: #e1306c;
  --secondary: #f77737;
  --bg: #121212;
  --panel: rgba(255, 255, 255, 0.05);
}

.page-container { padding: 1.5rem 2.5rem; height: 100vh; background: var(--bg); color: white; overflow: hidden; display: flex; flex-direction: column; }
.content-grid-dl { display: grid; grid-template-columns: 450px 1fr; gap: 2rem; margin-top: 1.5rem; flex: 1; min-height: 0; }
.glass-panel { background: var(--panel); border: 1px solid rgba(255, 255, 255, 0.05); border-radius: 2rem; padding: 2rem; backdrop-filter: blur(10px); display: flex; flex-direction: column; }
.mode-selector { display: flex; gap: 1rem; margin-bottom: 2rem; background: rgba(0, 0, 0, 0.3); padding: 0.5rem; border-radius: 1.25rem; }
.mode-btn { flex: 1; display: flex; align-items: center; justify-content: center; gap: 0.75rem; padding: 0.9rem; border: none; background: transparent; color: #94a3b8; font-weight: 700; border-radius: 1rem; cursor: pointer; transition: all 0.3s; }
.mode-btn.active { background: linear-gradient(45deg, #f09433 0%,#e6683c 25%,#dc2743 50%,#cc2366 75%,#bc1888 100%); color: white; box-shadow: 0 4px 15px rgba(225, 48, 108, 0.3); }
.premium-label { display: block; font-size: 0.75rem; font-weight: 800; text-transform: uppercase; letter-spacing: 0.1em; margin-bottom: 1rem; }
.ig-label { color: var(--primary); }
.glow-input { width: 100%; background: rgba(15, 23, 42, 0.6); border: 1px solid #334155; border-radius: 1rem; padding: 1.25rem; color: white; outline: none; transition: all 0.3s; }
.glow-input:focus { border-color: var(--primary); }
.settings-stack { margin: 2rem 0; }
.setting-row { display: flex; flex-direction: column; gap: 0.75rem; }
.setting-row label { font-size: 0.85rem; color: #64748b; }
.folder-picker { display: flex; gap: 0.75rem; }
.folder-picker input { flex: 1; background: rgba(0, 0, 0, 0.2); border: 1px solid #1e293b; border-radius: 0.75rem; padding: 0.75rem; color: #94a3b8; font-size: 0.8rem; }
.btn-picker { background: #1e293b; border: none; color: white; padding: 0 1.25rem; border-radius: 0.75rem; cursor: pointer; }
.premium-select-dl { width: 100%; background: rgba(0, 0, 0, 0.2); border: 1px solid #1e293b; border-radius: 0.75rem; padding: 0.75rem; color: #e2e8f0; font-size: 0.85rem; outline: none; }
.actions-group { margin-top: auto; }
.btn-download-ultra { width: 100%; padding: 1.5rem; border: none; border-radius: 1.25rem; color: white; font-weight: 900; font-size: 1rem; letter-spacing: 0.1em; cursor: pointer; transition: all 0.3s; }
.ig-btn { background: linear-gradient(45deg, #f09433 0%,#e6683c 25%,#dc2743 50%,#cc2366 75%,#bc1888 100%); box-shadow: 0 10px 30px rgba(225, 48, 108, 0.2); }
.btn-stop-dl { width: 100%; padding: 1.5rem; background: #1e293b; border: 1px solid rgba(255, 255, 255, 0.1); border-radius: 1.25rem; color: #ef4444; font-weight: 900; font-size: 1rem; letter-spacing: 0.1em; cursor: pointer; transition: all 0.3s; }
.panel-header-dl { display: flex; justify-content: space-between; align-items: center; margin-bottom: 2rem; }
.status-badge { background: #1e293b; padding: 4px 12px; border-radius: 20px; font-size: 0.7rem; font-weight: 800; text-transform: uppercase; color: #64748b; }
.status-badge.processing { background: rgba(225, 48, 108, 0.1); color: #e1306c; }
.status-badge.success { background: rgba(34, 197, 94, 0.1); color: #4ade80; }
.log-viewport { flex: 1; background: rgba(0, 0, 0, 0.2); border-radius: 1.5rem; padding: 1.5rem; overflow-y: auto; display: flex; flex-direction: column; gap: 0.75rem; }
.dl-log { font-family: 'JetBrains Mono', monospace; font-size: 0.8rem; padding-left: 1rem; border-left: 2px solid #1e293b; }
.log-time { opacity: 0.3; margin-right: 1rem; }
.log-empty { text-align: center; margin-top: 5rem; color: #334155; font-style: italic; }
.dl-progress-box { margin-top: 2rem; }
.dl-info { display: flex; justify-content: space-between; font-size: 0.85rem; color: #94a3b8; margin-bottom: 0.75rem; }
.dl-bar-bg { height: 8px; background: #0f172a; border-radius: 10px; overflow: hidden; }
.dl-bar-fill { height: 100%; transition: width 0.3s; }
.ig-bar-fill { background: linear-gradient(to right, #f09433, #e6683c, #dc2743, #cc2366, #bc1888); }
.loader-dl { width: 20px; height: 20px; border: 3px solid rgba(255, 255, 255, 0.3); border-top-color: white; border-radius: 50%; animation: spin 0.8s linear infinite; margin: 0 auto; }
@keyframes spin { to { transform: rotate(360deg); } }

.pro-tips-panel {
  margin-top: 2rem;
  padding: 1.25rem;
  background: rgba(225, 48, 108, 0.05);
  border: 1px dashed rgba(225, 48, 108, 0.2);
  border-radius: 1rem;
}

.tips-title {
  font-size: 0.85rem;
  color: #e1306c;
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
  color: #e1306c;
}

.custom-scrollbar::-webkit-scrollbar { width: 5px; }
.custom-scrollbar::-webkit-scrollbar-thumb { background: rgba(255, 255, 255, 0.1); border-radius: 10px; }
</style>
