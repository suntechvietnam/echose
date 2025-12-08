<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title">📸 Tách ảnh từ video</h2>
      <p class="page-subtitle">
        Chọn một video bất kỳ, kéo timeline để chọn thời điểm và bấm <strong>Extract</strong> để lưu ảnh hiện tại.
      </p>
    </div>

    <div class="extract-layout">
      <!-- Cột trái: chọn file + player -->
      <div class="left-column">
        <div class="file-group">
          <label class="form-label">Video nguồn</label>
          <div class="file-selector">
            <button class="btn-select-file" @click="selectVideo">
              🎞️ Chọn video
            </button>
            <span v-if="videoPath" class="file-name">
              {{ getFileName(videoPath) }}
            </span>
          </div>
        </div>

        <div v-if="videoUrl" class="video-player-wrapper">
          <video
            ref="videoRef"
            class="video-player"
            :src="videoUrl"
            controls
            @loadedmetadata="onLoadedMetadata"
            @timeupdate="onTimeUpdate"
          ></video>

          <div class="timeline-wrapper" v-if="duration > 0">
            <input
              type="range"
              min="0"
              :max="duration"
              step="0.04"
              v-model.number="sliderTime"
              @input="onSliderChange"
            />
            <div class="timeline-info">
              <span>Thời điểm: {{ formatTime(currentTime) }} / {{ formatTime(duration) }}</span>
            </div>
          </div>
        </div>

        <div v-else class="empty-video-hint">
          Chưa có video. Hãy chọn một file video để bắt đầu.
        </div>
      </div>

      <!-- Cột phải: canvas + nút extract + preview -->
      <div class="right-column">
        <div class="file-group">
          <label class="form-label">Kết quả ảnh</label>
          <div class="actions-row">
            <button
              class="btn-extract"
              :disabled="!videoUrl || isExtracting"
              @click="handleExtract"
            >
              {{ isExtracting ? 'Đang xử lý...' : '📸 Extract current frame' }}
            </button>
            <button
              v-if="capturedImageUrl"
              class="btn-download"
              @click="downloadImage"
            >
              💾 Tải ảnh
            </button>
          </div>
        </div>

        <div class="canvas-preview-wrapper">
          <canvas ref="canvasRef" class="capture-canvas"></canvas>

          <div v-if="capturedImageUrl" class="image-preview">
            <img :src="capturedImageUrl" alt="Captured frame" />
          </div>
          <div v-else class="image-preview empty">
            Ảnh sẽ hiển thị ở đây sau khi bạn bấm <strong>Extract</strong>.
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import { open, save } from '@tauri-apps/plugin-dialog'
import { writeFile } from '@tauri-apps/plugin-fs'
import { convertFileSrc } from '@tauri-apps/api/core'

const videoRef = ref(null)
const canvasRef = ref(null)

const videoPath = ref('')
const videoUrl = ref('')
const duration = ref(0)
const currentTime = ref(0)
const sliderTime = ref(0)

const isExtracting = ref(false)
const capturedImageUrl = ref('')
const capturedBlob = ref(null)

const getFileName = (filePath) => {
  return filePath.split('/').pop() || filePath.split('\\').pop() || filePath
}

const selectVideo = async () => {
  try {
    const selected = await open({
      multiple: false,
      filters: [
        {
          name: 'Video',
          extensions: ['mp4', 'mov', 'mkv', 'avi', 'webm']
        }
      ]
    })

    if (!selected) return

    const file = Array.isArray(selected) ? selected[0] : selected
    videoPath.value = file
    videoUrl.value = convertFileSrc(file)
    capturedImageUrl.value = ''
    duration.value = 0
    currentTime.value = 0
    sliderTime.value = 0
  } catch (error) {
    console.error('Lỗi khi chọn video:', error)
  }
}

const onLoadedMetadata = () => {
  if (videoRef.value) {
    duration.value = videoRef.value.duration || 0
  }
}

const onTimeUpdate = () => {
  if (videoRef.value) {
    currentTime.value = videoRef.value.currentTime
    sliderTime.value = currentTime.value
  }
}

const onSliderChange = () => {
  if (!videoRef.value) return
  try {
    videoRef.value.currentTime = sliderTime.value
  } catch (e) {
    console.warn('Không thể seek video:', e)
  }
}

const formatTime = (seconds) => {
  if (!seconds || isNaN(seconds)) return '00:00'
  const total = Math.floor(seconds)
  const mins = Math.floor(total / 60)
  const secs = total % 60
  const mm = String(mins).padStart(2, '0')
  const ss = String(secs).padStart(2, '0')
  return `${mm}:${ss}`
}

const handleExtract = async () => {
  if (!videoRef.value || !canvasRef.value) return
  if (!videoUrl.value) return

  isExtracting.value = true
  try {
    const video = videoRef.value
    const canvas = canvasRef.value
    const ctx = canvas.getContext('2d')
    if (!ctx) {
      console.error('Không thể lấy canvas context')
      return
    }

    // Đảm bảo video đã có kích thước
    const vw = video.videoWidth || video.clientWidth
    const vh = video.videoHeight || video.clientHeight
    if (!vw || !vh) {
      console.error('Video chưa sẵn sàng để capture')
      return
    }

    canvas.width = vw
    canvas.height = vh

    // Vẽ frame hiện tại của video lên canvas
    ctx.drawImage(video, 0, 0, vw, vh)

    // Xuất ảnh từ canvas
    canvas.toBlob((blob) => {
      if (!blob) {
        console.error('Không thể tạo blob từ canvas')
        return
      }
      // Lưu blob để có thể ghi file bằng Tauri
      capturedBlob.value = blob

      const url = URL.createObjectURL(blob)
      // Cleanup url cũ nếu có
      if (capturedImageUrl.value) {
        URL.revokeObjectURL(capturedImageUrl.value)
      }
      capturedImageUrl.value = url
    }, 'image/png')
  } catch (error) {
    console.error('Lỗi khi extract frame:', error)
  } finally {
    isExtracting.value = false
  }
}

const downloadImage = async () => {
  if (!capturedBlob.value) return
  try {
    const filePath = await save({
      title: 'Lưu ảnh',
      defaultPath: `frame_${Date.now()}.png`,
      filters: [
        { name: 'Image', extensions: ['png'] }
      ]
    })

    if (!filePath) return

    const arrayBuffer = await capturedBlob.value.arrayBuffer()
    const bytes = new Uint8Array(arrayBuffer)
    await writeFile(filePath, bytes)
  } catch (error) {
    console.error('Lỗi khi lưu ảnh:', error)
  }
}
</script>

<style scoped>
.page-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 1.5rem;
  overflow: hidden;
}

.page-header {
  margin-bottom: 1rem;
}

.page-title {
  font-size: 1.4rem;
  font-weight: 600;
  margin: 0 0 0.25rem;
}

.page-subtitle {
  margin: 0;
  color: #64748b;
  font-size: 0.9rem;
}

.extract-layout {
  display: grid;
  grid-template-columns: 1.2fr 1fr;
  gap: 1.5rem;
  flex: 1;
  min-height: 0;
}

.left-column,
.right-column {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.file-group {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.form-label {
  font-weight: 600;
  font-size: 0.9rem;
  color: #0f172a;
}

.file-selector {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.btn-select-file {
  padding: 8px 16px;
  background: #6366f1;
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.2s ease;
}

.btn-select-file:hover {
  background: #4f46e5;
}

.file-name {
  font-size: 0.85rem;
  color: #475569;
  max-width: 260px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.video-player-wrapper {
  background: #0f172a;
  border-radius: 12px;
  padding: 0.75rem;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.video-player {
  width: 100%;
  max-height: 320px;
  border-radius: 8px;
  background: black;
}

.timeline-wrapper {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.timeline-wrapper input[type="range"] {
  width: 100%;
}

.timeline-info {
  font-size: 0.8rem;
  color: #e5e7eb;
  display: flex;
  justify-content: space-between;
}

.empty-video-hint {
  flex: 1;
  border: 1px dashed #cbd5f5;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 1rem;
  font-size: 0.9rem;
  color: #64748b;
  background: #f8fafc;
}

.actions-row {
  display: flex;
  gap: 0.5rem;
  align-items: center;
}

.btn-extract {
  padding: 8px 16px;
  background: #0ea5e9;
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.2s ease;
}

.btn-extract:disabled {
  opacity: 0.6;
  cursor: default;
}

.btn-extract:not(:disabled):hover {
  background: #0284c7;
}

.btn-download {
  padding: 8px 16px;
  background: #22c55e;
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.2s ease;
}

.btn-download:hover {
  background: #16a34a;
}

.canvas-preview-wrapper {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  min-height: 0;
}

.capture-canvas {
  width: 100%;
  max-height: 240px;
  border-radius: 8px;
  background: #020617;
}

.image-preview {
  flex: 1;
  border-radius: 10px;
  border: 1px solid #e2e8f0;
  background: white;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0.5rem;
}

.image-preview img {
  max-width: 100%;
  max-height: 100%;
  border-radius: 8px;
  object-fit: contain;
}

.image-preview.empty {
  font-size: 0.9rem;
  color: #64748b;
  text-align: center;
}
</style>


