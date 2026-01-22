<template>
  <div class="download-page youtube-theme page-container">
    <div class="page-header">
      <div class="header-left">
        <h1 class="page-title">YouTube Downloader Ultra</h1>
        <p class="page-subtitle">Tải YouTube Video, Shorts hoặc toàn bộ kênh chất lượng 4K</p>
      </div>
    </div>

    <div class="content-grid-dl">
      <!-- Input Panel -->
      <div class="glass-panel input-panel">
        <div class="mode-selector">
          <button 
            class="mode-btn" 
            :class="{ active: mode === 'single' }"
            @click="mode = 'single'"
          >
            <span class="m-icon">📹</span>
            <span>Video Đơn</span>
          </button>
          <button 
            class="mode-btn" 
            :class="{ active: mode === 'list' }"
            @click="mode = 'list'"
          >
            <span class="m-icon">📁</span>
            <span>Channel/Playlist</span>
          </button>
        </div>

        <div class="url-input-section">
          <label class="premium-label">{{ mode === 'single' ? 'Video URL' : 'Channel/Playlist URL' }}</label>
          <div class="premium-input-group">
            <input 
              v-model="url" 
              type="text" 
              :placeholder="mode === 'single' ? 'Dán link YouTube hoặc YouTube Shorts...' : 'Dán link Kênh hoặc Danh sách phát...'"
              class="glow-input"
            />
          </div>
        </div>

        <div class="settings-stack">
          <div class="setting-row">
            <label>Thư mục lưu:</label>
            <div class="folder-picker">
              <input type="text" v-model="outputDir" readonly />
              <button @click="selectDir" class="btn-picker">Chọn</button>
            </div>
          </div>
          <div class="setting-row mt-2">
            <label>Chất lượng video:</label>
            <select v-model="quality" class="premium-select-dl">
              <option value="best">Cao nhất (Auto)</option>
              <option value="4k">4K (Ultra HD)</option>
              <option value="2k">2K (Quad HD)</option>
              <option value="1080p">1080p (Full HD)</option>
              <option value="720p">720p (HD)</option>
            </select>
          </div>
        </div>

        <div class="actions-group">
          <button 
            v-if="!isDownloading"
            class="btn-download-ultra" 
            :disabled="!url || !outputDir"
            @click="startDownload"
          >
            🚀 BẮT ĐẦU TẢI VỀ
          </button>
          <button 
            v-else
            class="btn-stop-dl" 
            @click="stopDownload"
          >
            🛑 DỪNG TẢI NGAY
          </button>
        </div>

        <!-- Pro Tips Panel -->
        <div class="pro-tips-panel">
          <h4 class="tips-title">💡 Mẹo tải YouTube:</h4>
          <ul class="tips-list">
            <li>Nên đăng nhập YouTube trên <strong>Chrome</strong> để tránh bị chặn Bot.</li>
            <li>Chọn <strong>4K/2K</strong> để lấy bản gốc siêu nét nếu video có hỗ trợ.</li>
            <li>Dán link <strong>Playlist</strong> và chọn "Channel/Playlist" để tải cả danh sách.</li>
            <li>Nếu lỗi liên tục, hãy thử xem video trên Chrome 5-10s rồi tải lại.</li>
          </ul>
        </div>
      </div>

      <!-- Progress Panel -->
      <div class="glass-panel progress-panel">
        <div class="panel-header-dl">
          <h3>Trạng thái tải về</h3>
          <span :class="statusClass">{{ statusText }}</span>
        </div>

        <div class="log-viewport custom-scrollbar" ref="logContainer">
          <div v-for="(log, i) in logs" :key="i" class="dl-log" :class="log.type">
            <span class="log-time">{{ log.time }}</span>
            <span class="log-msg">{{ log.msg }}</span>
          </div>
          <div v-if="logs.length === 0" class="log-empty">
            Hãy dán link và bắt đầu tải video...
          </div>
        </div>

        <div v-if="isDownloading" class="dl-progress-box">
          <div class="dl-info">
            <span>{{ progressMsg }}</span>
            <span>{{ progress }}%</span>
          </div>
          <div class="dl-bar-bg">
            <div class="dl-bar-fill" :style="{ width: progress + '%' }"></div>
          </div>
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
const mode = ref('single')
const outputDir = ref('')
const quality = ref('best')
const isDownloading = ref(false)
const progress = ref(0)
const progressMsg = ref('Đang kết nối...')
const logs = ref([])
const logContainer = ref(null)
let unlistenLog = null

const statusText = computed(() => {
  if (isDownloading.value) return 'Đang xử lý...'
  if (logs.value.some(l => l.type === 'success')) return 'Hoàn tất'
  return 'Sẵn sàng'
})

const statusClass = computed(() => {
  if (isDownloading.value) return 'status-badge processing'
  if (logs.value.some(l => l.type === 'success')) return 'status-badge success'
  return 'status-badge'
})

// Tự động scroll xuống cuối khi có log mới
const scrollToBottom = () => {
  if (logContainer.value) {
    setTimeout(() => {
      logContainer.value.scrollTop = logContainer.value.scrollHeight
    }, 50)
  }
}

const addLog = (msg, type = 'info') => {
  logs.value.push({ // đổi sang push để scroll xuống dưới cho tự nhiên
    time: new Date().toLocaleTimeString([], { hour12: false }),
    msg,
    type
  })
  scrollToBottom()
}

const selectDir = async () => {
  const selected = await open({ directory: true })
  if (selected) {
    outputDir.value = selected
    localStorage.setItem('dl_youtube_dir', selected)
  }
}

const stopDownload = async () => {
  try {
    const result = await invoke('stop_download')
    addLog(result, 'warning')
  } catch (err) {
    addLog('Lỗi khi dừng: ' + String(err), 'error')
  }
}

const startDownload = async () => {
  if (!url.value || !outputDir.value) return
  
  isDownloading.value = true
  progress.value = 0
  addLog(`Bắt đầu tải ${mode.value === 'single' ? 'video' : 'danh sách'}...`, 'info')
  
  try {
    const result = await invoke('download_video', {
      url: url.value,
      mode: mode.value,
      quality: quality.value,
      outputDir: outputDir.value
    })
    
    addLog(result, 'success')
    progress.value = 100
  } catch (err) {
    if (String(err).includes('process killed') || String(err).includes('kiểm tra kết quả')) {
       addLog('Đã dừng bởi người dùng.', 'warning')
    } else {
       addLog('Lỗi: ' + String(err), 'error')
    }
  } finally {
    isDownloading.value = false
  }
}

onMounted(async () => {
  const saved = localStorage.getItem('dl_youtube_dir')
  if (saved) outputDir.value = saved

  unlistenLog = await listen('download-log', (event) => {
    const logStr = event.payload
    addLog(logStr, 'info')
    
    // Parse progress if possible (e.g. "[download]  25.0% of...")
    const match = logStr.match(/(\d+\.?\d*)%/)
    if (match) {
      progress.value = parseFloat(match[1])
      progressMsg.value = 'Đang tải...'
    }
  })
})

onUnmounted(() => {
  if (unlistenLog) unlistenLog()
})
</script>

<style scoped>
.youtube-theme {
  --primary: #ef4444;
  --bg: #0f172a;
  --panel: rgba(255, 255, 255, 0.03);
}

.page-container {
  padding: 1.5rem 2.5rem;
  height: 100vh;
  background: var(--bg);
  color: white;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.content-grid-dl {
  display: grid;
  grid-template-columns: 450px 1fr;
  gap: 2rem;
  margin-top: 1.5rem;
  flex: 1;
  min-height: 0;
}

.glass-panel {
  background: var(--panel);
  border: 1px solid rgba(255, 255, 255, 0.05);
  border-radius: 2rem;
  padding: 2rem;
  backdrop-filter: blur(10px);
  display: flex;
  flex-direction: column;
}

.mode-selector {
  display: flex;
  gap: 1rem;
  margin-bottom: 2rem;
  background: rgba(0, 0, 0, 0.3);
  padding: 0.5rem;
  border-radius: 1.25rem;
}

.mode-btn {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.75rem;
  padding: 0.9rem;
  border: none;
  background: transparent;
  color: #94a3b8;
  font-weight: 700;
  border-radius: 1rem;
  cursor: pointer;
  transition: all 0.3s;
}

.mode-btn.active {
  background: var(--primary);
  color: white;
  box-shadow: 0 4px 15px rgba(239, 68, 68, 0.3);
}

.premium-label {
  display: block;
  font-size: 0.75rem;
  font-weight: 800;
  color: #ef4444;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  margin-bottom: 1rem;
}

.glow-input {
  width: 100%;
  background: rgba(15, 23, 42, 0.6);
  border: 1px solid #334155;
  border-radius: 1rem;
  padding: 1.25rem;
  color: white;
  outline: none;
  transition: all 0.3s;
}

.glow-input:focus {
  border-color: #ef4444;
  box-shadow: 0 0 20px rgba(239, 68, 68, 0.1);
}

.settings-stack {
  margin: 2.5rem 0;
}

.setting-row {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.setting-row label {
  font-size: 0.85rem;
  color: #64748b;
}

.folder-picker {
  display: flex;
  gap: 0.75rem;
}

.folder-picker input {
  flex: 1;
  background: rgba(0, 0, 0, 0.2);
  border: 1px solid #1e293b;
  border-radius: 0.75rem;
  padding: 0.75rem;
  color: #475569;
  font-size: 0.8rem;
}

.btn-picker {
  background: #1e293b;
  border: none;
  color: white;
  padding: 0 1.25rem;
  border-radius: 0.75rem;
  cursor: pointer;
}

.premium-select-dl {
  width: 100%;
  background: rgba(0, 0, 0, 0.2);
  border: 1px solid #1e293b;
  border-radius: 0.75rem;
  padding: 0.75rem;
  color: #e2e8f0;
  font-size: 0.85rem;
  outline: none;
}

.mt-2 { margin-top: 1rem; }

.btn-download-ultra {
  width: 100%;
  padding: 1.5rem;
  background: linear-gradient(135deg, #ef4444 0%, #b91c1c 100%);
  border: none;
  border-radius: 1.25rem;
  color: white;
  font-weight: 900;
  font-size: 1rem;
  letter-spacing: 0.1em;
  cursor: pointer;
  transition: all 0.3s;
  box-shadow: 0 10px 30px rgba(239, 68, 68, 0.3);
}

.btn-download-ultra:hover:not(:disabled) {
  transform: translateY(-3px);
  box-shadow: 0 15px 40px rgba(239, 68, 68, 0.4);
}

.btn-download-ultra:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  filter: grayscale(1);
}

.btn-stop-dl {
  width: 100%;
  padding: 1.5rem;
  background: #1e293b;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 1.25rem;
  color: #ef4444;
  font-weight: 900;
  font-size: 1rem;
  letter-spacing: 0.1em;
  cursor: pointer;
  transition: all 0.3s;
}

.btn-stop-dl:hover {
  background: #2d3748;
  border-color: #ef4444;
  transform: translateY(-3px);
}

.progress-panel {
  flex: 1;
  min-height: 0;
}

.panel-header-dl {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 2rem;
}

.status-badge {
  background: #1e293b;
  padding: 4px 12px;
  border-radius: 20px;
  font-size: 0.7rem;
  font-weight: 800;
  text-transform: uppercase;
  color: #64748b;
}

.status-badge.processing {
  background: rgba(59, 130, 246, 0.1);
  color: #60a5fa;
  animation: pulse 2s infinite;
}

.status-badge.success {
  background: rgba(34, 197, 94, 0.1);
  color: #4ade80;
}

.log-viewport {
  flex: 1;
  background: rgba(0, 0, 0, 0.2);
  border-radius: 1.5rem;
  padding: 1.5rem;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.dl-log {
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.8rem;
  padding-left: 1rem;
  border-left: 2px solid #1e293b;
}

.dl-log.success { border-color: #22c55e; color: #4ade80; }
.dl-log.error { border-color: #ef4444; color: #f87171; }
.log-time { opacity: 0.3; margin-right: 1rem; }

.log-empty {
  text-align: center;
  margin-top: 5rem;
  color: #334155;
  font-style: italic;
}

.dl-progress-box {
  margin-top: 2rem;
}

.dl-info {
  display: flex;
  justify-content: space-between;
  font-size: 0.85rem;
  color: #94a3b8;
  margin-bottom: 0.75rem;
}

.dl-bar-bg {
  height: 8px;
  background: #0f172a;
  border-radius: 10px;
  overflow: hidden;
}

.dl-bar-fill {
  height: 100%;
  background: linear-gradient(to right, #ef4444, #f87171);
  transition: width 0.3s;
}

.loader-dl {
  width: 20px;
  height: 20px;
  border: 3px solid rgba(255, 255, 255, 0.3);
  border-top-color: white;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
  margin: 0 auto;
}

@keyframes spin { to { transform: rotate(360deg); } }
@keyframes pulse { 0%, 100% { opacity: 1; } 50% { opacity: 0.6; } }

.pro-tips-panel {
  margin-top: 2rem;
  padding: 1.25rem;
  background: rgba(239, 68, 68, 0.05);
  border: 1px dashed rgba(239, 68, 68, 0.2);
  border-radius: 1rem;
}

.tips-title {
  font-size: 0.85rem;
  color: #ef4444;
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
  color: #ef4444;
}

.custom-scrollbar::-webkit-scrollbar { width: 5px; }
.custom-scrollbar::-webkit-scrollbar-thumb { background: rgba(255, 255, 255, 0.1); border-radius: 10px; }
</style>
