<template>
  <div class="ai-page ai-video-theme page-container">
    <div class="page-header">
      <div class="header-left">
        <h1 class="page-title">AI Video Animation</h1>
        <p class="page-subtitle">Tạo video Cinematic từ ý tưởng (Hoàn toàn miễn phí)</p>
      </div>
    </div>

    <div class="content-grid-ai">
      <!-- Controls -->
      <div class="glass-panel control-panel">
        <div class="prompt-section">
          <label class="premium-label">Mô tả cảnh quay</label>
          <textarea 
            v-model="prompt" 
            placeholder="Ví dụ: A majestic waterfall in a deep forest, sunlight through trees..."
            class="glow-textarea"
          ></textarea>
        </div>

        <div class="ai-settings">
          <div class="setting-row">
            <label>Hiệu ứng Motion:</label>
            <select v-model="effect" class="premium-select">
              <option value="zoom-in">Cinematic Zoom In</option>
              <option value="pan-right">Slow Pan Right</option>
              <option value="ken-burns">Classic Ken Burns</option>
            </select>
          </div>

          <div class="setting-row">
            <label>Thời lượng:</label>
            <select v-model="duration" class="premium-select">
              <option value="3">3 Giây (Nhanh)</option>
              <option value="5">5 Giây (Tiêu chuẩn)</option>
            </select>
          </div>
        </div>

        <button 
          class="btn-generate-video" 
          :disabled="!prompt || isGenerating"
          @click="generateVideo"
        >
          <span v-if="!isGenerating">🎬 TẠO VIDEO CINEMATIC</span>
          <div v-else class="loader-ai"></div>
        </button>

        <div class="pro-tips-panel">
          <h4 class="tips-title">💡 Giải pháp AI Video Miễn phí:</h4>
          <p class="tips-desc">Hệ thống sẽ dùng AI tạo ảnh gốc từ mô tả của bạn, sau đó dùng thuật toán Motion để biến thành Video chuẩn Cinematic.</p>
        </div>
      </div>

      <!-- Result View -->
      <div class="glass-panel result-panel">
        <div v-if="!videoUrl && !isGenerating" class="empty-state">
          <div class="empty-icon">🎬</div>
          <p>Nhập mô tả và bắt đầu sáng tạo video</p>
        </div>

        <div v-if="isGenerating" class="generating-state">
          <div class="ai-visualizer">
            <div class="ai-bar" v-for="i in 5" :key="i"></div>
          </div>
          <p>{{ statusMsg }}</p>
        </div>

        <div v-if="videoUrl" class="video-preview-container">
          <video 
            :src="videoUrl" 
            controls 
            autoplay 
            loop 
            class="main-video"
          ></video>
          <div class="image-actions">
            <button class="btn-action" @click="downloadVideo">📥 Tải Video về máy</button>
            <button class="btn-action secondary" @click="videoUrl = null">🗑️ Xóa</button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import { invoke, convertFileSrc } from '@tauri-apps/api/core'

const prompt = ref('')
const effect = ref('zoom-in')
const duration = ref('5')
const isGenerating = ref(false)
const videoUrl = ref(null)
const statusMsg = ref('Đang khởi tạo AI...')

const generateVideo = async () => {
  if (!prompt.value) return
  
  isGenerating.value = true
  statusMsg.value = 'AI đang vẽ cảnh quay...'
  
  try {
    // Bước 1: Tạo ảnh từ Prompt (Sử dụng logic giống trang AI Image)
    const seed = Math.floor(Math.random() * 1000000)
    const imageUrl = `https://pollinations.ai/p/${encodeURIComponent(prompt.value)}?width=1280&height=720&seed=${seed}&model=flux&nologo=true`
    
    // Đợi 2s để giả lập AI vẽ xong ảnh
    await new Promise(r => setTimeout(r, 2000))
    statusMsg.value = 'Đang tạo hiệu ứng Motion (FFmpeg)...'

    // Bước 2: Gọi Backend để biến Ảnh thành Video (Tôi sẽ triển khai hàm này ở rust phía dưới)
    // Giả sử chúng ta truyền link ảnh vào backend để nó xử lý
    const resultVideoPath = await invoke('create_video_from_ai_image', {
      imageUrl,
      effect: effect.value,
      duration: parseInt(duration.value)
    })
    
    // Chuyển đường dẫn file máy khách thành URL có thể xem được
    videoUrl.value = convertFileSrc(resultVideoPath)
    
  } catch (err) {
    alert('Lỗi tạo video: ' + String(err))
  } finally {
    isGenerating.value = false
  }
}

const downloadVideo = () => {
  // Logic tải file từ path
  alert('Video đã được lưu vào thư mục Temp của ứng dụng. Đường dẫn: ' + videoUrl.value)
}
</script>

<style scoped>
.ai-video-theme {
  --primary: #ec4899;
  --bg: #09090b;
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

.content-grid-ai {
  display: grid;
  grid-template-columns: 400px 1fr;
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
  backdrop-filter: blur(20px);
}

.glow-textarea {
  width: 100%;
  height: 120px;
  background: rgba(0, 0, 0, 0.3);
  border: 1px solid #334155;
  border-radius: 1rem;
  padding: 1rem;
  color: white;
  resize: none;
  font-family: inherit;
  margin-top: 0.5rem;
}

.glow-textarea:focus {
  border-color: var(--primary);
  box-shadow: 0 0 20px rgba(236, 72, 153, 0.2);
  outline: none;
}

.ai-settings { margin: 2rem 0; }
.setting-row { margin-bottom: 1.5rem; }
.setting-row label { display: block; font-size: 0.8rem; color: #94a3b8; margin-bottom: 0.75rem; }

.premium-select {
  width: 100%; padding: 0.75rem; background: #1f2937; border: 1px solid #374151;
  border-radius: 0.75rem; color: white; outline: none;
}

.btn-generate-video {
  width: 100%; padding: 1.25rem; background: linear-gradient(135deg, #ec4899 0%, #be185d 100%);
  border: none; border-radius: 1rem; color: white; font-weight: 800; cursor: pointer;
  transition: 0.3s; box-shadow: 0 10px 25px rgba(236, 72, 153, 0.3);
}

.result-panel {
  flex: 1; display: flex; align-items: center; justify-content: center;
  background: rgba(0, 0, 0, 0.6); overflow: hidden;
}

.main-video {
  max-width: 100%; max-height: 75vh; border-radius: 1rem;
  box-shadow: 0 25px 50px rgba(0,0,0,0.6);
}

.image-actions { display: flex; gap: 1rem; justify-content: center; margin-top: 1.5rem; }

.btn-action {
  padding: 0.8rem 1.5rem; background: var(--primary); border: none; border-radius: 0.75rem;
  color: white; font-weight: 700; cursor: pointer;
}
.btn-action.secondary { background: #374151; }

.empty-state { text-align: center; color: #4b5563; }
.empty-icon { font-size: 4rem; margin-bottom: 1rem; opacity: 0.3; }

.generating-state { text-align: center; }
.ai-visualizer {
  display: flex; justify-content: center; gap: 8px; margin-bottom: 2rem; height: 40px;
}
.ai-bar {
  width: 6px; height: 100%; background: var(--primary); border-radius: 3px;
  animation: ai-wave 1s infinite ease-in-out;
}
.ai-bar:nth-child(2) { animation-delay: 0.1s; }
.ai-bar:nth-child(3) { animation-delay: 0.2s; }
.ai-bar:nth-child(4) { animation-delay: 0.3s; }
.ai-bar:nth-child(5) { animation-delay: 0.4s; }

@keyframes ai-wave {
  0%, 100% { height: 20%; opacity: 0.3; }
  50% { height: 100%; opacity: 1; }
}

.pro-tips-panel {
  margin-top: 1.5rem; padding: 1.25rem; background: rgba(236, 72, 153, 0.05);
  border: 1px dashed rgba(236, 72, 153, 0.2); border-radius: 1rem;
}
.tips-title { font-size: 0.85rem; color: var(--primary); margin-bottom: 0.5rem; font-weight: 800; }
.tips-desc { font-size: 0.75rem; color: #94a3b8; line-height: 1.5; }

.loader-ai {
  width: 24px; height: 24px; border: 3px solid rgba(255,255,255,0.3);
  border-top-color: white; border-radius: 50%; animation: spin 0.8s linear infinite; margin: 0 auto;
}
@keyframes spin { to { transform: rotate(360deg); } }
</style>
