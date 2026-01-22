<template>
  <div class="download-page tiktok-theme page-container">
    <div class="page-header">
      <div class="header-left">
        <h1 class="page-title">TikTok Downloader Pro</h1>
        <p class="page-subtitle">Tải video TikTok không logo: Từng video hoặc toàn bộ profile</p>
      </div>
    </div>

    <div class="content-grid-dl">
      <!-- Input Panel -->
      <div class="glass-panel input-panel">
        <div class="mode-selector tt-mode">
          <button 
            class="mode-btn" 
            :class="{ active: mode === 'single' }"
            @click="mode = 'single'"
          >
            <span class="m-icon">📱</span>
            <span>Video Đơn</span>
          </button>
          <button 
            class="mode-btn" 
            :class="{ active: mode === 'list' }"
            @click="mode = 'list'"
          >
            <span class="m-icon">👤</span>
            <span>Toàn bộ Profile</span>
          </button>
        </div>

        <div class="url-input-section">
          <label class="premium-label tt-label">{{ mode === 'single' ? 'TikTok Video URL' : 'TikTok Profile URL' }}</label>
          <div class="premium-input-group">
            <input 
              v-model="url" 
              type="text" 
              :placeholder="mode === 'single' ? 'Dán link video TikTok...' : 'Dán link trang cá nhân TikTok (@user)...'"
              class="glow-input tt-input"
            />
          </div>
        </div>

        <div class="settings-stack">
          <div class="setting-row">
            <label>Thư mục lưu:</label>
            <div class="folder-picker">
              <input type="text" v-model="outputDir" readonly />
              <button @click="selectDir" class="btn-picker tt-picker">Chọn</button>
            </div>
          </div>
          
          <div class="setting-row" style="margin-top: 1rem;">
            <label>Chất lượng:</label>
            <select v-model="quality" class="premium-select-dl tt-input">
              <option value="best">Cao nhất (Tự động 4K/HD)</option>
              <option value="4k">4K (Ultra HD)</option>
              <option value="2k">2K (Quad HD)</option>
              <option value="1080p">1080p (Full HD)</option>
              <option value="720p">720p (HD)</option>
            </select>
          </div>
          
          <div class="tt-option-row">
            <label class="tt-switch">
              <input type="checkbox" v-model="noWatermark">
              <span class="tt-slider"></span>
            </label>
            <span class="tt-opt-label">Xóa Watermark (Mặc định)</span>
          </div>
        </div>

        <div class="actions-group">
          <button 
            v-if="!isDownloading"
            class="btn-download-ultra tt-btn" 
            :disabled="!url || !outputDir"
            @click="startDownload"
          >
            🚀 BẮT ĐẦU TẢI TIKTOK
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
          <h4 class="tips-title">💡 Mẹo tải TikTok:</h4>
          <ul class="tips-list">
            <li>Dùng <strong>Chrome</strong> xem video sẽ giúp tool tải ổn định hơn.</li>
            <li><strong>Chất lượng:</strong> Chọn "Cao nhất" để app tự lấy bản gốc 1080p/2K.</li>
            <li>Nếu tải Profile quá lâu, hãy dùng nút <strong>Dừng</strong> để kiểm tra các file đã tải.</li>
            <li>Lỗi <strong>429</strong>: Hãy tạm dừng 1-2 phút rồi tải lại để Reset IP.</li>
          </ul>
        </div>
      </div>

      <!-- Progress Panel -->
      <div class="glass-panel progress-panel tt-logs-list">
        <div class="panel-header-dl">
          <h3>Tiến trình TikTok</h3>
          <span :class="statusClass">{{ statusText }}</span>
        </div>

        <div class="log-viewport tt-log-view custom-scrollbar" ref="logContainer">
          <div v-for="(log, i) in logs" :key="i" class="dl-log tt-log-item" :class="log.type">
            <span class="log-time">{{ log.time }}</span>
            <span class="log-msg">{{ log.msg }}</span>
          </div>
          <div v-if="logs.length === 0" class="log-empty">
            Dán link TikTok để bắt đầu...
          </div>
        </div>

        <div v-if="isDownloading" class="dl-progress-box">
          <div class="dl-info">
            <span>{{ progressMsg }}</span>
            <span>{{ progress }}%</span>
          </div>
          <div class="dl-bar-bg tt-bar-bg">
            <div class="dl-bar-fill tt-bar-fill" :style="{ width: progress + '%' }"></div>
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
const noWatermark = ref(true)
const isDownloading = ref(false)
const progress = ref(0)
const progressMsg = ref('Đang quét video...')
const logs = ref([])
const logContainer = ref(null)
let unlistenLog = null

// Tự động scroll xuống cuối khi có log mới
const scrollToBottom = () => {
  if (logContainer.value) {
    setTimeout(() => {
      logContainer.value.scrollTop = logContainer.value.scrollHeight
    }, 50)
  }
}

const statusText = computed(() => {
  if (isDownloading.value) return 'Đang quét...'
  if (logs.value.some(l => l.type === 'success')) return 'Hoàn tất'
  return 'Sẵn sàng'
})

const statusClass = computed(() => {
  if (isDownloading.value) return 'status-badge processing tt-active'
  if (logs.value.some(l => l.type === 'success')) return 'status-badge status-tt-success'
  return 'status-badge'
})

const addLog = (msg, type = 'info') => {
  logs.value.push({
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
    localStorage.setItem('dl_tiktok_dir', selected)
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
  addLog(`Bắt đầu tải TikTok (${mode.value})...`, 'info')
  
  try {
    const result = await invoke('download_video', {
      url: url.value,
      mode: mode.value,
      quality: quality.value,
      noWatermark: noWatermark.value,
      outputDir: outputDir.value
    })
    
    addLog(result, 'success')
    progress.value = 100
  } catch (err) {
    if (String(err).includes('process killed') || String(err).includes('kiểm tra kết quả')) {
       addLog('Đã dừng bởi người dùng.', 'warning')
    } else {
       addLog('Lỗi TikTok: ' + String(err), 'error')
    }
  } finally {
    isDownloading.value = false
  }
}

onMounted(async () => {
  const saved = localStorage.getItem('dl_tiktok_dir')
  if (saved) outputDir.value = saved

  unlistenLog = await listen('download-log', (event) => {
    const logStr = event.payload
    addLog(logStr, 'info')
    
    const match = logStr.match(/(\d+\.?\d*)%/)
    if (match) {
      progress.value = parseFloat(match[1])
      progressMsg.value = 'Đang tải TikTok...'
    }
  })
})

onUnmounted(() => {
  if (unlistenLog) unlistenLog()
})
</script>

<style scoped>
.tiktok-theme {
  --primary: #fe2c55;
  --secondary: #25f4ee;
  --bg: #010101;
  --panel: rgba(255, 255, 255, 0.05);
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
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 2rem;
  padding: 2rem;
  backdrop-filter: blur(20px);
  display: flex;
  flex-direction: column;
}

.progress-panel {
  flex: 1;
  min-height: 0;
  height: auto;
}

/* Base UI Elements */
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
  box-shadow: 0 4px 20px rgba(254, 44, 85, 0.4);
}

.btn-download-ultra:hover:not(:disabled) {
  transform: translateY(-3px);
  box-shadow: 0 15px 40px rgba(254, 44, 85, 0.4);
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

.premium-label {
  display: block;
  font-size: 0.75rem;
  font-weight: 800;
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

.settings-stack { margin: 2rem 0; }
.setting-row { display: flex; flex-direction: column; gap: 0.75rem; }
.setting-row label { font-size: 0.85rem; color: #64748b; }

.folder-picker { display: flex; gap: 0.75rem; }
.folder-picker input {
  flex: 1;
  background: rgba(0, 0, 0, 0.2);
  border: 1px solid #1e293b;
  border-radius: 0.75rem;
  padding: 0.75rem;
  color: #94a3b8;
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

.btn-download-ultra {
  width: 100%;
  padding: 1.5rem;
  border: none;
  border-radius: 1.25rem;
  color: white;
  font-weight: 900;
  font-size: 1rem;
  letter-spacing: 0.1em;
  cursor: pointer;
  transition: all 0.3s;
}

.premium-select-dl {
  width: 100%;
  background: rgba(0, 0, 0, 0.4) !important;
  border: 1px solid #333 !important;
  border-radius: 0.75rem;
  padding: 0.8rem;
  color: white;
  outline: none;
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

.log-time { opacity: 0.3; margin-right: 1rem; }
.log-empty { text-align: center; margin-top: 5rem; color: #334155; font-style: italic; }

.dl-bar-bg { height: 8px; border-radius: 10px; overflow: hidden; background: #222; margin-top: 1rem; }
.dl-bar-fill { height: 100%; transition: width 0.3s; }

/* TikTok Specific Overrides */
.tt-mode { background: #121212 !important; border: 1px solid #333 !important; }
.tt-label { color: var(--primary); }
.tt-input { background: #121212 !important; border: 1px solid #333 !important; }
.tt-input:focus { border-color: var(--primary) !important; box-shadow: 0 0 20px rgba(254, 44, 85, 0.2) !important; }
.tt-picker { background: #333 !important; }
.tt-btn { background: linear-gradient(135deg, var(--primary) 0%, var(--secondary) 100%) !important; box-shadow: 0 10px 30px rgba(254, 44, 85, 0.3) !important; }

.tt-active { color: var(--primary) !important; }
.status-tt-success { color: var(--secondary) !important; }

.tt-log-view { background: #121212 !important; }
.tt-log-item { border-left-color: var(--primary) !important; }

.tt-option-row { display: flex; align-items: center; gap: 1rem; margin-top: 1.5rem; }
.tt-switch { position: relative; width: 44px; height: 22px; }
.tt-switch input { opacity: 0; }
.tt-slider {
  position: absolute; cursor: pointer; top: 0; left: 0; right: 0; bottom: 0;
  background-color: #333; transition: .4s; border-radius: 34px;
}
.tt-slider:before {
  position: absolute; content: ""; height: 16px; width: 16px; left: 3px; bottom: 3px;
  background-color: white; transition: .4s; border-radius: 50%;
}
input:checked + .tt-slider { background-color: var(--primary); }
input:checked + .tt-slider:before { transform: translateX(22px); }
.tt-opt-label { font-size: 0.8rem; color: #94a3b8; }

.tt-bar-bg { background: #333 !important; }
.tt-bar-fill { background: linear-gradient(to right, var(--primary), var(--secondary)) !important; }

.loader-dl {
  width: 20px; height: 20px; border: 3px solid rgba(255, 255, 255, 0.2);
  border-top-color: var(--primary); border-radius: 50%; animation: spin 0.8s linear infinite;
}

@keyframes spin { to { transform: rotate(360deg); } }
@keyframes pulse { 0%, 100% { opacity: 1; } 50% { opacity: 0.6; } }

.pro-tips-panel {
  margin-top: 2rem;
  padding: 1.25rem;
  background: rgba(254, 44, 85, 0.05);
  border: 1px dashed rgba(254, 44, 85, 0.2);
  border-radius: 1rem;
}

.tips-title {
  font-size: 0.85rem;
  color: #fe2c55;
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
  color: #fe2c55;
}

.custom-scrollbar::-webkit-scrollbar { width: 5px; }
.custom-scrollbar::-webkit-scrollbar-thumb { background: rgba(255, 255, 255, 0.1); border-radius: 10px; }
</style>
