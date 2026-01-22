<template>
  <div class="reup-page tiktok-theme page-container">
    <div class="page-header">
      <div class="header-left">
        <h1 class="page-title">TikTok Reup Master</h1>
        <p class="page-subtitle">Quét video xu hướng & Reup tự động với hiệu ứng Viral</p>
      </div>
    </div>

    <div class="reup-grid">
      <!-- Input Section -->
      <div class="glass-panel main-panel">
        <div class="input-url-box">
          <label class="premium-label">TikTok Video / Profile URL</label>
          <div class="url-input-wrapper">
            <span class="url-icon">🎵</span>
            <input 
              v-model="videoUrl" 
              type="text" 
              placeholder="https://www.tiktok.com/@user/video/..." 
              class="premium-input"
            />
            <button class="btn-fetch" @click="fetchInfo">Quét Video</button>
          </div>
        </div>

        <div v-if="videoInfo" class="video-preview-card tiktok-card animate-fade-in">
          <div class="v-preview-wrap">
             <img :src="videoInfo.thumbnail" class="v-thumb-vertical" />
             <div class="v-play-overlay">▶</div>
          </div>
          <div class="v-meta">
            <h3 class="v-title">{{ videoInfo.title }}</h3>
            <div class="v-author">@{{ videoInfo.author }}</div>
            <div class="v-stats">
              <span>❤️ {{ videoInfo.likes }}</span>
              <span>💬 {{ videoInfo.comments }}</span>
            </div>
          </div>
        </div>

        <div class="processing-options">
          <h4 class="section-title">Viral Reup Settings</h4>
          <div class="options-grid">
            <div class="option-card tt-opt" v-for="opt in options" :key="opt.id">
              <label class="checkbox-container">
                <input type="checkbox" v-model="opt.enabled">
                <span class="checkmark tiktok-check"></span>
                <div class="opt-info">
                  <span class="opt-name">{{ opt.name }}</span>
                  <p class="opt-desc">{{ opt.desc }}</p>
                </div>
              </label>
            </div>
          </div>
        </div>

        <div class="action-footer">
          <button class="btn-start-reup tiktok-action" :disabled="!videoUrl || isProcessing" @click="startReup">
            <span v-if="!isProcessing">BẮT ĐẦU REUP VIRAL</span>
            <div v-else class="loader-nano"></div>
          </button>
        </div>
      </div>

      <!-- Log/Status Section -->
      <div class="glass-panel status-panel">
        <h3 class="panel-title-mini">Tiến độ TikTok</h3>
        <div class="logs-container custom-scrollbar">
          <div v-for="(log, index) in logs" :key="index" class="log-entry tt-log" :class="log.type">
            <span class="log-time">{{ log.time }}</span>
            <span class="log-msg">{{ log.msg }}</span>
          </div>
          <div v-if="logs.length === 0" class="log-empty">Hệ thống đang sẵn sàng...</div>
        </div>

        <div v-if="isProcessing" class="overall-progress">
          <div class="progress-info">
            <span>TikTok Processing...</span>
            <span>{{ progress }}%</span>
          </div>
          <div class="progress-bar-bg">
            <div class="progress-bar-fill tt-fill" :style="{ width: progress + '%' }"></div>
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
  { id: 'remove_wm', name: 'Xóa Watermark', desc: 'Tải video gốc không logo', enabled: true },
  { id: 'speed', name: 'Thay đổi tốc độ', desc: 'Lách bản quyền thuật toán', enabled: true },
  { id: 'zoom', name: 'Deep Zoom', desc: 'Tối ưu AI quét hình ảnh', enabled: true },
  { id: 'music', name: 'Remix Audio', desc: 'Thay nhạc nền bản quyền', enabled: false },
])

const logs = ref([])
const addLog = (msg, type = 'info') => {
  logs.value.unshift({ time: new Date().toLocaleTimeString(), msg, type })
}

const fetchInfo = () => {
  if (!videoUrl.value) return
  addLog('Đang kết nối tới TikTok Server...', 'info')
  setTimeout(() => {
    videoInfo.value = {
      title: 'Trending Dance Challenge #2024',
      author: 'top_viral_creator',
      thumbnail: 'https://picsum.photos/seed/tt/200/350',
      likes: '850K',
      comments: '12K'
    }
    addLog('Đã quét thành công video của @' + videoInfo.value.author, 'success')
  }, 1000)
}

const startReup = () => {
  isProcessing.value = true
  progress.value = 0
  addLog('Bắt đầu quy trình Reup TikTok...', 'info')
  
  const interval = setInterval(() => {
    progress.value += 10
    if (progress.value === 20) addLog('Đang xóa TikTok Watermark...', 'info')
    if (progress.value === 50) addLog('Đang tối ưu hóa metadata...', 'info')
    if (progress.value === 80) addLog('Đang thêm hash lách bản quyền...', 'info')
    
    if (progress.value >= 100) {
      clearInterval(interval)
      isProcessing.value = false
      addLog('Hoàn tất! Video đã sẵn sàng tải lên.', 'success')
    }
  }, 200)
}
</script>

<style scoped>
.page-container { padding: 2.5rem; background: #010101; min-height: 100vh; color: white; }
.reup-grid { display: grid; grid-template-columns: 1fr 380px; gap: 2rem; margin-top: 2rem; }

.glass-panel {
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 1.5rem;
  padding: 2.5rem;
}

.premium-label { color: #fe2c55; font-size: 0.8rem; font-weight: 800; text-transform: uppercase; margin-bottom: 0.75rem; display: block; }
.url-input-wrapper {
  display: flex; gap: 0.75rem; background: #121212; padding: 0.75rem; border-radius: 1rem; border: 1px solid #333;
}
.url-icon { font-size: 1.2rem; }
.premium-input { flex: 1; background: transparent; border: none; color: white; outline: none; }
.btn-fetch { background: #fe2c55; color: white; border: none; padding: 0.5rem 1.5rem; border-radius: 0.75rem; font-weight: 800; cursor: pointer; transition: 0.3s; }
.btn-fetch:hover { background: #ff3b5c; transform: scale(1.05); }

.video-preview-card.tiktok-card {
  margin-top: 2rem; gap: 1.5rem; background: rgba(0, 242, 234, 0.05); border: 1px solid rgba(0, 242, 234, 0.2);
}
.v-preview-wrap { position: relative; width: 100px; height: 160px; }
.v-thumb-vertical { width: 100%; height: 100%; object-fit: cover; border-radius: 0.75rem; }
.v-play-overlay { position: absolute; top: 50%; left: 50%; transform: translate(-50%, -50%); font-size: 2rem; opacity: 0.7; }
.v-title { font-size: 1rem; margin-bottom: 0.5rem; color: #fff; }
.v-author { font-weight: 800; color: #69c9d0; margin-bottom: 0.5rem; }
.v-stats { display: flex; gap: 1rem; font-size: 0.8rem; color: #94a3b8; }

.settings-title { font-size: 0.85rem; color: #69c9d0; margin-bottom: 1.5rem; }
.tt-opt { border-left: 3px solid #fe2c55; }
.tiktok-check { border-color: #333; }
.checkbox-container input:checked ~ .tiktok-check { background: #fe2c55; border-color: #fe2c55; }

.btn-start-reup.tiktok-action {
  background: linear-gradient(135deg, #fe2c55 0%, #25f4ee 100%);
  border-radius: 3rem; box-shadow: 0 10px 30px rgba(254, 44, 85, 0.3);
}

.status-panel { background: rgba(18, 18, 18, 0.6); }
.tt-log { border-color: #fe2c55; }
.tt-fill { background: linear-gradient(to right, #fe2c55, #25f4ee); }

.loader-nano { width: 22px; height: 22px; border: 3px solid rgba(255, 255, 255, 0.2); border-top-color: #fe2c55; border-radius: 50%; animation: spin 0.8s linear infinite; margin: 0 auto; }
</style>
