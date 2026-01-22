<template>
  <div class="ai-page ai-image-theme page-container">
    <div class="page-header">
      <div class="header-left">
        <h1 class="page-title">AI Image Studio</h1>
        <p class="page-subtitle">Biến ý tưởng thành hình ảnh nghệ thuật với sức mạnh AI (Flux/SDXL)</p>
      </div>
    </div>

    <div class="content-grid-ai">
      <!-- Controls -->
      <div class="glass-panel control-panel">
        <div class="prompt-section">
          <label class="premium-label">Mô tả hình ảnh (English is better)</label>
          <textarea 
            v-model="prompt" 
            placeholder="Ví dụ: A futuristic city with cyberpunk lights, ultra realistic, 8k..."
            class="glow-textarea"
          ></textarea>
        </div>

        <div class="ai-settings">
          <div class="setting-row">
            <label>Tỷ lệ khung hình:</label>
            <div class="ratio-picker">
              <button 
                v-for="r in ratios" 
                :key="r.val"
                :class="{ active: ratio === r.val }"
                @click="ratio = r.val"
              >
                {{ r.label }}
              </button>
            </div>
          </div>

          <div class="setting-row">
            <label>Model AI:</label>
            <select v-model="model" class="premium-select">
              <option value="flux">Flux.1 (Chân thực nhất)</option>
              <option value="turbo">Stable Diffusion Turbo (Nhanh nhất)</option>
            </select>
          </div>
        </div>

        <button 
          class="btn-generate" 
          :disabled="!prompt || isGenerating"
          @click="generateImage"
        >
          <span v-if="!isGenerating">✨ TẠO ẢNH MIỄN PHÍ</span>
          <div v-else class="loader-ai"></div>
        </button>

        <div class="pro-tips-panel">
          <h4 class="tips-title">💡 Mẹo tạo ảnh đẹp:</h4>
          <ul class="tips-list">
            <li>Nên dùng tiếng Anh để AI hiểu chính xác nhất.</li>
            <li>Thêm các từ khóa: <em>masterpiece, cinematic, hyper-realistic, highly detailed</em>.</li>
            <li>Flux model cực giỏi trong việc vẽ chữ và bàn tay.</li>
          </ul>
        </div>
      </div>

      <!-- Result View -->
      <div class="glass-panel result-panel">
        <div v-if="!generatedUrl && !isGenerating" class="empty-state">
          <div class="empty-icon">🎨</div>
          <p>Nhập mô tả và nhấn nút tạo để bắt đầu</p>
        </div>

        <div v-if="isGenerating" class="generating-state">
          <div class="pulse-circle"></div>
          <p>AI đang "vẽ" bức tranh của bạn...</p>
        </div>

        <div v-if="generatedUrl" class="image-preview-container">
          <img :src="generatedUrl" alt="AI Generated" class="main-image" @load="isGenerating = false" />
          <div class="image-actions">
            <button class="btn-action" @click="downloadImage">📥 Tải về máy</button>
            <button class="btn-action secondary" @click="copyLink">🔗 Copy Link</button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue'

const prompt = ref('')
const ratio = ref('16:9')
const model = ref('flux')
const isGenerating = ref(false)
const generatedUrl = ref(null)

const ratios = [
  { label: '1:1', val: '1:1' },
  { label: '9:16', val: '9:16' },
  { label: '16:9', val: '16:9' }
]

const generateImage = () => {
  if (!prompt.value) return
  
  isGenerating.value = true
  
  // Xử lý kích thước dựa trên tỷ lệ
  let width = 1024
  let height = 1024
  if (ratio.value === '16:9') { width = 1280; height = 720 }
  if (ratio.value === '9:16') { width = 720; height = 1280 }

  // Sử dụng Pollinations AI (Hoàn toàn miễn phí, không cần Key)
  const seed = Math.floor(Math.random() * 1000000)
  const encodedPrompt = encodeURIComponent(prompt.value)
  
  // URL format: https://image.pollinations.ai/prompt/{prompt}?width={w}&height={h}&model={model}&seed={seed}
  generatedUrl.value = `https://pollinations.ai/p/${encodedPrompt}?width=${width}&height=${height}&seed=${seed}&model=${model.value}&nologo=true`
  
  // Lưu ý: Image sẽ tự load, isGenerating sẽ tắt ở sự kiện @load của thẻ img
}

const downloadImage = async () => {
  if (!generatedUrl.value) return
  const response = await fetch(generatedUrl.value)
  const blob = await response.blob()
  const url = window.URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `ai-image-${Date.now()}.png`
  a.click()
}

const copyLink = () => {
  navigator.clipboard.writeText(generatedUrl.value)
  alert('Đã copy link ảnh!')
}
</script>

<style scoped>
.ai-image-theme {
  --primary: #8b5cf6;
  --bg: #030712;
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
  box-shadow: 0 0 20px rgba(139, 92, 246, 0.2);
  outline: none;
}

.ai-settings { margin: 2rem 0; }
.setting-row { margin-bottom: 1.5rem; }
.setting-row label { display: block; font-size: 0.8rem; color: #94a3b8; margin-bottom: 0.75rem; }

.ratio-picker { display: flex; gap: 0.5rem; }
.ratio-picker button {
  flex: 1; padding: 0.6rem; background: #1f2937; border: 1px solid #374151;
  border-radius: 0.75rem; color: #94a3b8; cursor: pointer; transition: 0.3s;
}
.ratio-picker button.active { background: var(--primary); color: white; border-color: var(--primary); }

.premium-select {
  width: 100%; padding: 0.75rem; background: #1f2937; border: 1px solid #374151;
  border-radius: 0.75rem; color: white; outline: none;
}

.btn-generate {
  width: 100%; padding: 1.25rem; background: linear-gradient(135deg, #8b5cf6 0%, #6d28d9 100%);
  border: none; border-radius: 1rem; color: white; font-weight: 800; cursor: pointer;
  transition: 0.3s; box-shadow: 0 10px 25px rgba(139, 92, 246, 0.3);
}

.btn-generate:hover:not(:disabled) { transform: translateY(-3px); box-shadow: 0 15px 35px rgba(139, 92, 246, 0.4); }

.result-panel {
  flex: 1; display: flex; align-items: center; justify-content: center; position: relative;
  background: rgba(0, 0, 0, 0.4); overflow: hidden;
}

.main-image {
  max-width: 100%; max-height: 80vh; border-radius: 1rem;
  box-shadow: 0 20px 50px rgba(0,0,0,0.5);
}

.image-preview-container { text-align: center; }
.image-actions { display: flex; gap: 1rem; justify-content: center; margin-top: 1.5rem; }

.btn-action {
  padding: 0.8rem 1.5rem; background: var(--primary); border: none; border-radius: 0.75rem;
  color: white; font-weight: 700; cursor: pointer;
}
.btn-action.secondary { background: #374151; }

.empty-state { text-align: center; color: #4b5563; }
.empty-icon { font-size: 4rem; margin-bottom: 1rem; opacity: 0.3; }

.generating-state { text-align: center; }
.pulse-circle {
  width: 60px; height: 60px; background: var(--primary); border-radius: 50%;
  margin: 0 auto 1.5rem; animation: pulse 1.5s infinite;
}

@keyframes pulse { 0% { transform: scale(0.95); opacity: 0.7; } 50% { transform: scale(1.1); opacity: 0.3; } 100% { transform: scale(0.95); opacity: 0.7; } }

.loader-ai {
  width: 24px; height: 24px; border: 3px solid rgba(255,255,255,0.3);
  border-top-color: white; border-radius: 50%; animation: spin 0.8s linear infinite; margin: 0 auto;
}
@keyframes spin { to { transform: rotate(360deg); } }

.pro-tips-panel {
  margin-top: 1.5rem; padding: 1rem; background: rgba(139, 92, 246, 0.05);
  border: 1px dashed rgba(139, 92, 246, 0.2); border-radius: 1rem;
}
.tips-title { font-size: 0.8rem; color: var(--primary); margin-bottom: 0.5rem; }
.tips-list { list-style: none; padding: 0; font-size: 0.7rem; color: #94a3b8; line-height: 1.5; }
.tips-list li::before { content: "• "; color: var(--primary); }
</style>
