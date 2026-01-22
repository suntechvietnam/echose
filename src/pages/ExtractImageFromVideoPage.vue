<template>
  <div class="video-to-image-page">
    <!-- Hero Header -->
    <div class="page-hero">
      <div class="hero-content">
        <div class="hero-icon">📸</div>
        <div class="hero-text">
          <h1 class="hero-title">Tách Ảnh Từ Video</h1>
          <p class="hero-subtitle">Trích xuất khung hình chất lượng cao: thủ công hoặc tự động</p>
        </div>
      </div>
    </div>

    <!-- Main Content -->
    <div class="content-wrapper">
      <!-- Left Column: Media Selection & Mode -->
      <div class="left-panel">
        <div class="glass-card">
          <div class="card-header">
            <h3 class="card-title">
              <span class="title-icon">📹</span>
              Chọn Video Nguồn
            </h3>
          </div>
          
          <div class="file-selector-box">
            <button class="btn-select-main" @click="selectVideo">
              <span class="btn-icon">🎞️</span>
              {{ videoPath ? 'Đổi Video khác' : 'Chọn Video' }}
            </button>
            <div v-if="videoPath" class="selected-file-info">
              <span class="file-label">Đang chọn:</span>
              <span class="file-name">{{ getFileName(videoPath) }}</span>
            </div>
          </div>

          <div class="mode-tabs">
            <button 
              class="tab-btn" 
              :class="{ active: extractMode === 'manual' }"
              @click="extractMode = 'manual'"
            >
              <span class="tab-icon">🎯</span>
              Thủ công
            </button>
            <button 
              class="tab-btn" 
              :class="{ active: extractMode === 'auto' }"
              @click="extractMode = 'auto'"
            >
              <span class="tab-icon">⚡</span>
              Tự động
            </button>
          </div>
        </div>

        <!-- Mode: Manual Player -->
        <div v-if="extractMode === 'manual'" class="glass-card player-card">
          <div v-if="videoUrl" class="player-container">
            <video
              ref="videoRef"
              class="video-preview"
              :src="videoUrl"
              controls
              @loadedmetadata="onLoadedMetadata"
              @timeupdate="onTimeUpdate"
            ></video>

            <div class="custom-timeline" v-if="duration > 0">
              <input
                type="range"
                min="0"
                :max="duration"
                step="0.04"
                v-model.number="sliderTime"
                @input="onSliderChange"
                class="timeline-slider"
              />
              <div class="timeline-stats">
                <span class="current-time">{{ formatTime(currentTime) }}</span>
                <span class="total-time">{{ formatTime(duration) }}</span>
              </div>
            </div>
          </div>
          <div v-else class="empty-hint">
            <div class="hint-icon">🎥</div>
            <p>Vui lòng chọn video để bắt đầu chọn khung hình</p>
          </div>
        </div>

        <!-- Mode: Auto Settings -->
        <div v-if="extractMode === 'auto'" class="glass-card settings-card">
          <div class="card-header">
            <h3 class="card-title">
              <span class="title-icon">⚙️</span>
              Cấu hình Trích xuất tự động
            </h3>
          </div>
          
          <div class="setting-item">
            <label class="setting-label">Số giây mỗi ảnh:</label>
            <div class="input-with-unit">
              <input 
                type="number" 
                v-model.number="autoInterval" 
                min="0.1" 
                step="0.1"
                class="number-input"
              />
              <span class="unit">giây</span>
            </div>
            <p class="setting-hint">Ví dụ: Nhập 5 để cứ 5 giây lưu 1 ảnh.</p>
          </div>

          <div class="setting-item">
            <label class="setting-label">Thư mục lưu ảnh:</label>
            <div class="folder-selector">
              <input 
                type="text" 
                v-model="outputFolder"
                class="folder-input" 
                placeholder="Chọn thư mục..."
                readonly
              />
              <button class="btn-select-folder" @click="selectOutputFolder">
                <span class="btn-icon">📁</span>
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- Right Column: Results & Actions -->
      <div class="right-panel">
        <div class="glass-card results-card" :class="{ 'manual-mode': extractMode === 'manual' }">
          <div class="card-header">
            <h3 class="card-title">
              <span class="title-icon">🖼️</span>
              Kết quả
            </h3>
          </div>

          <!-- Manual Results Preview -->
          <div v-if="extractMode === 'manual'" class="manual-preview-box">
            <div class="canvas-wrapper">
              <canvas ref="canvasRef" style="display: none;"></canvas>
              <div v-if="capturedImageUrl" class="preview-img-container">
                <img :src="capturedImageUrl" alt="Captured" />
              </div>
              <div v-else class="preview-empty">
                <span>Khung hình đã chọn sẽ xuất hiện tại đây</span>
              </div>
            </div>

            <div class="manual-actions">
              <button 
                class="btn-action btn-extract-manual"
                :disabled="!videoUrl || isProcessing"
                @click="handleManualExtract"
              >
                <span class="btn-icon">📸</span>
                {{ isProcessing ? 'Đang trích xuất...' : 'Trích xuất Frame hiện tại' }}
              </button>
              <button 
                v-if="capturedImageUrl"
                class="btn-action btn-save-manual"
                @click="downloadManualImage"
              >
                <span class="btn-icon">💾</span>
                Lưu ảnh này
              </button>
            </div>
          </div>

          <!-- Auto Results Status -->
          <div v-if="extractMode === 'auto'" class="auto-status-box">
            <div class="status-content">
              <div v-if="!isProcessing && !statusMessage" class="ready-hint">
                <div class="ready-icon">🚀</div>
                <p>Sẵn sàng trích xuất hàng loạt ảnh theo thời gian.</p>
              </div>

              <div v-if="isProcessing" class="processing-box">
                <div class="spinner-large"></div>
                <p class="process-text">Đang xử lý video...</p>
                <p class="process-sub">Hệ thống đang trích xuất ảnh theo chu kỳ {{ autoInterval }}s</p>
                <button class="btn-stop" @click="stopAutoExtraction">
                  <span class="btn-icon">⏹️</span>
                  Dừng lại
                </button>
              </div>

              <div v-if="statusMessage" :class="['status-banner', statusType]">
                <span class="status-icon">{{ statusType === 'success' ? '✅' : '❌' }}</span>
                <span class="status-text">{{ statusMessage }}</span>
              </div>
            </div>

            <div class="auto-actions">
              <button 
                class="btn-primary-action"
                v-if="!isProcessing"
                :disabled="!videoPath || !outputFolder || autoInterval <= 0"
                @click="startAutoExtraction"
              >
                <span class="btn-icon">⚡</span>
                Bắt đầu Trích xuất Tự động
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onUnmounted } from 'vue'
import { open, save } from '@tauri-apps/plugin-dialog'
import { writeFile } from '@tauri-apps/plugin-fs'
import { convertFileSrc } from '@tauri-apps/api/core'
import { useTauri } from '../composables/useTauri'

const { callCommand } = useTauri()

// Refs
const videoRef = ref(null)
const canvasRef = ref(null)

// State
const extractMode = ref('manual') // 'manual' | 'auto'
const videoPath = ref('')
const videoUrl = ref('')
const duration = ref(0)
const currentTime = ref(0)
const sliderTime = ref(0)

const isProcessing = ref(false)
const capturedImageUrl = ref('')
const capturedBlob = ref(null)

// Auto mode state
const autoInterval = ref(5)
const outputFolder = ref('')
const statusMessage = ref('')
const statusType = ref('info')
const currentProcessId = ref(null)

const getFileName = (filePath) => {
  return filePath.split(/[\\/]/).pop() || filePath
}

const selectVideo = async () => {
  try {
    const selected = await open({
      multiple: false,
      filters: [{ name: 'Video', extensions: ['mp4', 'mov', 'mkv', 'avi', 'webm'] }]
    })

    if (!selected) return

    videoPath.value = Array.isArray(selected) ? selected[0] : selected
    videoUrl.value = convertFileSrc(videoPath.value)
    
    // Reset manual state
    capturedImageUrl.value = ''
    capturedBlob.value = null
    duration.value = 0
    currentTime.value = 0
    sliderTime.value = 0
    statusMessage.value = ''
  } catch (error) {
    console.error('Lỗi khi chọn video:', error)
  }
}

const selectOutputFolder = async () => {
  try {
    const selected = await open({
      directory: true,
      multiple: false
    })
    if (selected) {
      outputFolder.value = Array.isArray(selected) ? selected[0] : selected
    }
  } catch (error) {
    console.error('Lỗi khi chọn thư mục:', error)
  }
}

// Manual Mode Logic
const onLoadedMetadata = () => {
  if (videoRef.value) duration.value = videoRef.value.duration || 0
}

const onTimeUpdate = () => {
  if (videoRef.value) {
    currentTime.value = videoRef.value.currentTime
    sliderTime.value = currentTime.value
  }
}

const onSliderChange = () => {
  if (videoRef.value) videoRef.value.currentTime = sliderTime.value
}

const formatTime = (seconds) => {
  if (!seconds || isNaN(seconds)) return '00:00'
  const total = Math.floor(seconds)
  const mins = Math.floor(total / 60)
  const secs = total % 60
  return `${String(mins).padStart(2, '0')}:${String(secs).padStart(2, '0')}`
}

const handleManualExtract = async () => {
  if (!videoRef.value || !canvasRef.value) return
  isProcessing.value = true
  
  try {
    const video = videoRef.value
    const canvas = canvasRef.value
    const ctx = canvas.getContext('2d')
    
    // Luôn lấy độ phân giải gốc của video
    const vw = video.videoWidth
    const vh = video.videoHeight
    
    if (!vw || !vh) {
      alert('Video chưa tải xong độ phân giải gốc, vui lòng đợi giây lát')
      isProcessing.value = false
      return
    }
    
    canvas.width = vw
    canvas.height = vh
    ctx.drawImage(video, 0, 0, vw, vh)

    canvas.toBlob((blob) => {
      if (!blob) return
      capturedBlob.value = blob
      if (capturedImageUrl.value) URL.revokeObjectURL(capturedImageUrl.value)
      capturedImageUrl.value = URL.createObjectURL(blob)
      isProcessing.value = false
    }, 'image/png')
  } catch (error) {
    console.error('Extract error:', error)
    isProcessing.value = false
  }
}

const downloadManualImage = async () => {
  if (!capturedBlob.value) return
  try {
    const filePath = await save({
      title: 'Lưu ảnh',
      defaultPath: `frame_${Date.now()}.png`,
      filters: [{ name: 'Image', extensions: ['png'] }]
    })
    if (!filePath) return
    const bytes = new Uint8Array(await capturedBlob.value.arrayBuffer())
    await writeFile(filePath, bytes)
  } catch (error) {
    console.error('Save error:', error)
  }
}

// Auto Mode Logic
const startAutoExtraction = async () => {
  if (!videoPath.value || !outputFolder.value) return
  
  isProcessing.value = true
  statusMessage.value = ''
  
  const pid = `extract_auto_${Date.now()}`
  currentProcessId.value = pid
  
  try {
    await callCommand('extract_images_from_video_periodic', {
      videoPath: videoPath.value,
      outputDir: outputFolder.value,
      interval: autoInterval.value,
      processId: pid
    })
    
    statusMessage.value = 'Đã trích xuất xong toàn bộ ảnh!'
    statusType.value = 'success'
  } catch (error) {
    statusMessage.value = 'Lỗi: ' + error
    statusType.value = 'error'
  } finally {
    isProcessing.value = false
    currentProcessId.value = null
  }
}

const stopAutoExtraction = async () => {
  if (!currentProcessId.value) return
  try {
    await callCommand('stop_image_extraction', { processId: currentProcessId.value })
    statusMessage.value = 'Đã dừng trích xuất'
    statusType.value = 'info'
    isProcessing.value = false
    currentProcessId.value = null
  } catch (error) {
    console.error('Stop error:', error)
  }
}

onUnmounted(() => {
  if (capturedImageUrl.value) URL.revokeObjectURL(capturedImageUrl.value)
})
</script>

<style scoped>
.video-to-image-page {
  padding: 24px;
  min-height: 100vh;
  background: #0f172a;
  color: white;
}

.page-hero {
  margin-bottom: 32px;
  padding: 40px;
  background: linear-gradient(135deg, #1e293b 0%, #0f172a 100%);
  border-radius: 24px;
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.hero-content {
  display: flex;
  align-items: center;
  gap: 24px;
}

.hero-icon {
  font-size: 48px;
  background: rgba(14, 165, 233, 0.2);
  width: 80px;
  height: 80px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 20px;
}

.hero-title {
  font-size: 32px;
  font-weight: 700;
  margin-bottom: 8px;
  background: linear-gradient(to right, #fff, #94a3b8);
  background-clip: text;
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.hero-subtitle {
  color: #94a3b8;
  font-size: 16px;
}

.content-wrapper {
  display: grid;
  grid-template-columns: 1.2fr 1fr;
  gap: 24px;
}

.glass-card {
  background: rgba(30, 41, 59, 0.7);
  backdrop-filter: blur(12px);
  border-radius: 20px;
  padding: 24px;
  border: 1px solid rgba(255, 255, 255, 0.05);
  margin-bottom: 24px;
}

.card-header {
  margin-bottom: 20px;
}

.card-title {
  display: flex;
  align-items: center;
  gap: 12px;
  font-size: 18px;
  font-weight: 600;
}

.file-selector-box {
  background: rgba(15, 23, 42, 0.4);
  padding: 20px;
  border-radius: 16px;
  margin-bottom: 20px;
  border: 1px dashed rgba(255, 255, 255, 0.1);
}

.btn-select-main {
  width: 100%;
  padding: 14px;
  background: #3b82f6;
  border: none;
  border-radius: 12px;
  color: white;
  font-weight: 600;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  transition: all 0.2s;
}

.btn-select-main:hover {
  background: #2563eb;
  transform: translateY(-1px);
}

.selected-file-info {
  margin-top: 12px;
  font-size: 13px;
  display: flex;
  gap: 8px;
  color: #94a3b8;
}

.file-name {
  color: #e2e8f0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.mode-tabs {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
  background: rgba(15, 23, 42, 0.5);
  padding: 6px;
  border-radius: 12px;
}

.tab-btn {
  padding: 10px;
  border: none;
  border-radius: 8px;
  background: transparent;
  color: #94a3b8;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  font-weight: 500;
  transition: all 0.2s;
}

.tab-btn.active {
  background: #1e293b;
  color: white;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
}

/* Manual Player */
.player-container {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.video-preview {
  width: 100%;
  border-radius: 12px;
  background: black;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
}

.custom-timeline {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.timeline-slider {
  width: 100%;
  accent-color: #0ea5e9;
  cursor: pointer;
}

.timeline-stats {
  display: flex;
  justify-content: space-between;
  font-size: 12px;
  color: #94a3b8;
  font-family: monospace;
}

.empty-hint {
  height: 200px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: #64748b;
  text-align: center;
}

.hint-icon {
  font-size: 40px;
  margin-bottom: 12px;
  opacity: 0.5;
}

/* Auto Settings */
.setting-item {
  margin-bottom: 24px;
}

.setting-label {
  display: block;
  margin-bottom: 8px;
  font-size: 14px;
  color: #94a3b8;
}

.input-with-unit {
  display: flex;
  align-items: center;
  gap: 12px;
}

.number-input {
  flex: 1;
  background: rgba(15, 23, 42, 0.5);
  border: 1px solid rgba(255, 255, 255, 0.1);
  padding: 12px;
  border-radius: 8px;
  color: white;
  font-size: 14px;
}

.unit {
  color: #94a3b8;
}

.setting-hint {
  font-size: 12px;
  color: #64748b;
  margin-top: 6px;
}

.folder-selector {
  display: flex;
  gap: 8px;
}

.folder-input {
  flex: 1;
  background: rgba(15, 23, 42, 0.5);
  border: 1px solid rgba(255, 255, 255, 0.1);
  padding: 12px;
  border-radius: 8px;
  color: #94a3b8;
  font-size: 13px;
}

.btn-select-folder {
  padding: 0 16px;
  background: #334155;
  border: none;
  border-radius: 8px;
  color: white;
  cursor: pointer;
}

/* Results Section */
.results-card {
  height: calc(100% - 24px);
  display: flex;
  flex-direction: column;
}

.results-card.manual-mode {
  min-height: 500px;
}

.canvas-wrapper {
  background: rgba(15, 23, 42, 0.5);
  border-radius: 12px;
  height: 300px;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  margin-bottom: 20px;
  border: 1px solid rgba(255, 255, 255, 0.05);
}

.preview-img-container img {
  max-width: 100%;
  max-height: 300px;
  object-fit: contain;
}

.preview-empty {
  color: #475569;
  font-size: 14px;
  text-align: center;
}

.manual-actions {
  display: grid;
  gap: 12px;
}

.btn-action {
  padding: 14px;
  border: none;
  border-radius: 12px;
  font-weight: 600;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  transition: all 0.2s;
}

.btn-extract-manual {
  background: #0ea5e9;
  color: white;
}

.btn-save-manual {
  background: #10b981;
  color: white;
}

/* Auto Results */
.auto-status-box {
  flex: 1;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
}

.ready-hint {
  text-align: center;
  padding: 40px 0;
  color: #64748b;
}

.ready-icon {
  font-size: 48px;
  margin-bottom: 16px;
  opacity: 0.3;
}

.processing-box {
  text-align: center;
  padding: 40px 20px;
  background: rgba(255, 255, 255, 0.02);
  border-radius: 16px;
}

.spinner-large {
  width: 50px;
  height: 50px;
  border: 4px solid rgba(14, 165, 233, 0.2);
  border-top-color: #0ea5e9;
  border-radius: 50%;
  margin: 0 auto 24px;
  animation: spin 1s linear infinite;
}

.process-text {
  font-weight: 600;
  font-size: 18px;
  margin-bottom: 8px;
}

.process-sub {
  color: #64748b;
  font-size: 14px;
  margin-bottom: 24px;
}

.btn-stop {
  background: rgba(244, 63, 94, 0.1);
  color: #fb7185;
  border: 1px solid rgba(244, 63, 94, 0.2);
  padding: 10px 24px;
  border-radius: 12px;
  cursor: pointer;
  font-weight: 500;
}

.status-banner {
  margin-top: 20px;
  padding: 16px;
  border-radius: 12px;
  display: flex;
  gap: 12px;
  font-size: 14px;
}

.status-banner.success {
  background: rgba(16, 185, 129, 0.1);
  color: #34d399;
}

.status-banner.error {
  background: rgba(239, 68, 68, 0.1);
  color: #f87171;
}

.btn-primary-action {
  width: 100%;
  padding: 18px;
  background: linear-gradient(135deg, #0ea5e9 0%, #0284c7 100%);
  border: none;
  border-radius: 16px;
  color: white;
  font-weight: 700;
  font-size: 16px;
  cursor: pointer;
  box-shadow: 0 8px 24px rgba(14, 165, 233, 0.2);
  transition: all 0.3s;
}

.btn-primary-action:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 12px 32px rgba(14, 165, 233, 0.4);
}

.btn-primary-action:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  box-shadow: none;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>
