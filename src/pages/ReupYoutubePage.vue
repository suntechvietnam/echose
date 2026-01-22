<template>
  <div class="reup-page page-container">
    <div class="page-header">
      <div class="header-left">
        <h1 class="page-title">YouTube Reup Engine</h1>
        <p class="page-subtitle">Tự động hóa quy trình Reup video YouTube với công nghệ lách bản quyền AI</p>
      </div>
    </div>

    <div class="reup-grid">
      <!-- Input Section -->
      <div class="glass-panel main-panel">
        <div class="input-url-box">
          <label class="premium-label">YouTube Video URL</label>
          <div class="url-input-wrapper">
            <span class="url-icon">🔗</span>
            <input 
              v-model="videoUrl" 
              type="text" 
              placeholder="https://www.youtube.com/watch?v=..." 
              class="premium-input"
            />
            <button class="btn-fetch" @click="fetchInfo">Phân tích</button>
          </div>
        </div>

        <div v-if="videoInfo" class="video-preview-card animate-fade-in">
          <img :src="videoInfo.thumbnail" class="v-thumb" />
          <div class="v-meta">
            <h3 class="v-title">{{ videoInfo.title }}</h3>
            <div class="v-stats">
              <span>⏱️ {{ videoInfo.duration }}</span>
              <span>👁️ {{ videoInfo.views }}</span>
            </div>
          </div>
        </div>

        <div class="processing-options">
          <h4 class="section-title">Cấu hình xử lý Video</h4>
          <div class="options-grid">
            <div class="option-card" v-for="opt in options" :key="opt.id">
              <label class="checkbox-container">
                <input type="checkbox" v-model="opt.enabled">
                <span class="checkmark"></span>
                <div class="opt-info">
                  <span class="opt-name">{{ opt.name }}</span>
                  <p class="opt-desc">{{ opt.desc }}</p>
                </div>
              </label>
            </div>
          </div>
        </div>

        <div class="action-footer">
          <button class="btn-start-reup" :disabled="!videoUrl || isProcessing" @click="startReup">
            <span v-if="!isProcessing">BẮT ĐẦU XỬ LÝ & REUP</span>
            <div v-else class="loader-nano"></div>
          </button>
        </div>
      </div>

      <!-- Log/Status Section -->
      <div class="glass-panel status-panel">
        <h3 class="panel-title-mini">Tiến trình xử lý</h3>
        <div class="logs-container custom-scrollbar">
          <div v-for="(log, index) in logs" :key="index" class="log-entry" :class="log.type">
            <span class="log-time">{{ log.time }}</span>
            <span class="log-msg">{{ log.msg }}</span>
          </div>
          <div v-if="logs.length === 0" class="log-empty">Chưa có hoạt động nào...</div>
        </div>

        <div v-if="isProcessing" class="overall-progress">
          <div class="progress-info">
            <span>Đang hoàn thiện video...</span>
            <span>{{ progress }}%</span>
          </div>
          <div class="progress-bar-bg">
            <div class="progress-bar-fill" :style="{ width: progress + '%' }"></div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue'

const videoUrl = ref('')
const isProcessing = ref(false)
const progress = ref(0)
const videoInfo = ref(null)

const options = ref([
  { id: 'mirror', name: 'Lật gương', desc: 'Lách nhận diện hình ảnh', enabled: true },
  { id: 'pitch', name: 'Nâng Tone Audio', desc: 'Tránh bản quyền âm thanh', enabled: true },
  { id: 'frame', name: 'Thêm khung nền', desc: 'Thay đổi tỷ lệ khung hình', enabled: false },
  { id: 'overlay', name: 'Hiệu ứng phủ', desc: 'Chống ID Content AI', enabled: true },
])

const logs = ref([])

const addLog = (msg, type = 'info') => {
  logs.value.unshift({
    time: new Date().toLocaleTimeString(),
    msg,
    type
  })
}

const fetchInfo = () => {
  if (!videoUrl.value) return
  addLog('Đang lấy thông tin video từ YouTube...', 'info')
  // Mock fetch
  setTimeout(() => {
    videoInfo.value = {
      title: 'Hustle Hard - Motivational Video 2024',
      thumbnail: 'https://picsum.photos/seed/yt/300/180',
      duration: '03:45',
      views: '1.2M'
    }
    addLog('Tìm thấy video: ' + videoInfo.value.title, 'success')
  }, 1000)
}

const startReup = () => {
  isProcessing.value = true
  progress.value = 0
  addLog('Bắt đầu tải video...', 'info')
  
  const interval = setInterval(() => {
    progress.value += 5
    if (progress.value === 30) addLog('Đang áp dụng hiệu ứng lách bản quyền...', 'info')
    if (progress.value === 60) addLog('Đang điều chỉnh dải tần âm thanh...', 'info')
    if (progress.value === 90) addLog('Đang xuất bản thành phẩm...', 'info')
    
    if (progress.value >= 100) {
      clearInterval(interval)
      isProcessing.value = false
      addLog('Đã hoàn tất! Video sẵn sàng tại thư mục REUP.', 'success')
    }
  }, 300)
}
</script>

<style scoped>
.page-container { padding: 2.5rem; background: #0b0f1a; min-height: 100vh; }
.reup-grid { display: grid; grid-template-columns: 1fr 380px; gap: 2rem; margin-top: 2rem; }

.glass-panel {
  background: rgba(255, 255, 255, 0.02);
  border: 1px solid rgba(255, 255, 255, 0.05);
  border-radius: 1.5rem;
  padding: 2rem;
}

.premium-label { display: block; font-size: 0.8rem; font-weight: 700; color: #94a3b8; margin-bottom: 0.75rem; }
.url-input-wrapper {
  display: flex; gap: 0.75rem; background: #161b22; padding: 0.5rem; border-radius: 1rem; border: 1px solid #30363d;
}
.url-icon { padding: 0.5rem; font-size: 1.2rem; }
.premium-input {
  flex: 1; background: transparent; border: none; color: white; outline: none; font-size: 0.9rem;
}
.btn-fetch {
  background: #30363d; border: 1px solid #484f58; color: white; padding: 0.5rem 1.25rem;
  border-radius: 0.75rem; cursor: pointer; font-size: 0.85rem; transition: all 0.2s;
}
.btn-fetch:hover { background: #484f58; }

.video-preview-card {
  margin-top: 1.5rem; display: flex; gap: 1rem; background: rgba(255, 255, 255, 0.03);
  padding: 1rem; border-radius: 1rem; border: 1px solid rgba(255, 255, 255, 0.05);
}
.v-thumb { width: 140px; border-radius: 0.5rem; }
.v-title { font-size: 0.95rem; font-weight: 700; margin-bottom: 0.5rem; }
.v-stats { font-size: 0.75rem; color: #64748b; display: flex; gap: 1rem; }

.processing-options { margin-top: 2.5rem; }
.section-title { font-size: 0.85rem; color: #818cf8; margin-bottom: 1.25rem; }
.options-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; }

.option-card {
  background: rgba(255, 255, 255, 0.02); border: 1px solid rgba(255, 255, 255, 0.05);
  border-radius: 1rem; padding: 1rem;
}

.checkbox-container { display: flex; align-items: flex-start; gap: 0.75rem; cursor: pointer; }
.checkmark { width: 18px; height: 18px; border: 2px solid #30363d; border-radius: 4px; position: relative; flex-shrink: 0; }
.checkbox-container input:checked ~ .checkmark { background: #6366f1; border-color: #6366f1; }
.checkbox-container input { display: none; }
.opt-name { font-size: 0.85rem; font-weight: 700; color: white; display: block; }
.opt-desc { font-size: 0.7rem; color: #475569; margin-top: 0.25rem; line-height: 1.4; }

.action-footer { margin-top: 3rem; }
.btn-start-reup {
  width: 100%; padding: 1.25rem; border-radius: 1rem; border: none;
  background: linear-gradient(135deg, #ef4444 0%, #991b1b 100%);
  color: white; font-weight: 900; letter-spacing: 0.05em; cursor: pointer;
  box-shadow: 0 10px 25px rgba(239, 68, 68, 0.2); transition: all 0.3s;
}
.btn-start-reup:hover:not(:disabled) { transform: translateY(-2px); box-shadow: 0 15px 35px rgba(239, 68, 68, 0.3); }

.status-panel { display: flex; flex-direction: column; height: 600px; }
.panel-title-mini { font-size: 0.8rem; color: #64748b; margin-bottom: 1.5rem; }
.logs-container { flex: 1; overflow-y: auto; display: flex; flex-direction: column; gap: 0.75rem; }
.log-entry { font-size: 0.75rem; border-left: 2px solid #30363d; padding-left: 0.75rem; }
.log-time { color: #475569; margin-right: 0.5rem; }
.log-entry.success { border-color: #22c55e; color: #4ade80; }
.log-entry.error { border-color: #ef4444; color: #f87171; }
.log-empty { text-align: center; color: #334155; margin-top: 4rem; font-style: italic; }

.overall-progress { margin-top: 2rem; }
.progress-info { display: flex; justify-content: space-between; font-size: 0.75rem; color: #94a3b8; margin-bottom: 0.5rem; }
.progress-bar-bg { height: 6px; background: #161b22; border-radius: 10px; overflow: hidden; }
.progress-bar-fill { height: 100%; background: #6366f1; transition: width 0.3s; }

.loader-nano { width: 20px; height: 20px; border: 2px solid rgba(255, 255, 255, 0.3); border-top-color: white; border-radius: 50%; animation: spin 0.8s linear infinite; margin: 0 auto; }
@keyframes spin { to { transform: rotate(360deg); } }
</style>
