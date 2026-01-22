<template>
  <div class="extract-audio-page">
    <!-- Hero Header -->
    <div class="page-hero">
      <div class="hero-content">
        <div class="hero-icon">🎙️</div>
        <div class="hero-text">
          <h1 class="hero-title">Tách Audio Khỏi Video</h1>
          <p class="hero-subtitle">Trích xuất âm thanh chất lượng cao từ mọi tập tin video</p>
        </div>
      </div>
    </div>

    <!-- Main Content -->
    <div class="content-wrapper">
      <!-- Left Column: Video Selection -->
      <div class="left-panel">
        <div class="glass-card">
          <div class="card-header">
            <h3 class="card-title">
              <span class="title-icon">📹</span>
              Chọn Video Đầu Vào
            </h3>
          </div>
          
          <ImportVideoSection 
            :modelValue="videoFiles"
            :maxFiles="1"
            @update:videoFiles="handleVideoFilesUpdate"
            @status-message="handleStatusMessage"
          />
        </div>
      </div>

      <!-- Right Column: Settings & Actions -->
      <div class="right-panel">
        <!-- Export Settings Card -->
        <div class="glass-card settings-card">
          <div class="card-header">
            <h3 class="card-title">
              <span class="title-icon">⚙️</span>
              Cấu hình Xuất file
            </h3>
          </div>
          
          <div class="settings-options">
            <div class="format-selector">
              <span class="label">Định dạng:</span>
              <div class="radio-group">
                <label class="radio-item" :class="{ active: audioFormat === 'mp3' }">
                  <input type="radio" value="mp3" v-model="audioFormat" />
                  <span>MP3 (Phổ biến)</span>
                </label>
                <label class="radio-item" :class="{ active: audioFormat === 'm4a' }">
                  <input type="radio" value="m4a" v-model="audioFormat" />
                  <span>M4A/AAC (Chất lượng)</span>
                </label>
              </div>
            </div>
          </div>
        </div>

        <!-- Output Folder Card -->
        <div class="glass-card folder-card">
          <div class="card-header">
            <h3 class="card-title">
              <span class="title-icon">💾</span>
              Thư Mục Lưu
            </h3>
          </div>
          
          <div class="folder-selector">
            <input 
              type="text" 
              v-model="outputFolder"
              class="folder-input" 
              placeholder="Chọn thư mục để lưu audio..."
              readonly
            />
            <button class="btn-select-folder" @click="selectOutputFolder">
              <span class="btn-icon">📁</span>
              Chọn
            </button>
          </div>
        </div>

        <!-- Action Buttons -->
        <div class="action-section">
          <button 
            class="btn-primary btn-create" 
            @click="extractAudio" 
            :disabled="isCreating || !videoFiles.length || !outputFolder"
            :class="{ loading: isCreating }"
          >
            <span v-if="!isCreating" class="btn-content">
              <span class="btn-icon">⚡</span>
              <span>Bắt đầu Trích xuất</span>
            </span>
            <span v-else class="btn-content">
              <span class="spinner"></span>
              <span>Đang trích xuất...</span>
            </span>
          </button>

          <button 
            v-if="isCreating"
            class="btn-secondary btn-cancel" 
            @click="stopExtraction"
          >
            <span class="btn-icon">⏹️</span>
            Huỷ
          </button>
        </div>

        <!-- Progress & Status -->
        <div v-if="isCreating" class="progress-section">
          <div class="progress-bar-wrapper">
            <div class="progress-bar">
              <div class="progress-fill" style="width: 50%"></div>
            </div>
            <div class="progress-text">Đang xử lý...</div>
          </div>
        </div>

        <div v-if="statusMessage" :class="['status-message', statusType]">
          <span class="status-icon">{{ getStatusIcon(statusType) }}</span>
          <span class="status-text">{{ statusMessage }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { useTauri } from '../composables/useTauri'
import ImportVideoSection from '@/components/ImportVideoSection.vue'

const { callCommand } = useTauri()

// State
const videoFiles = ref([])
const audioFormat = ref('mp3')
const outputFolder = ref('')
const isCreating = ref(false)
const statusMessage = ref('')
const statusType = ref('info')
const currentProcessId = ref(null)

const handleVideoFilesUpdate = (files) => {
  videoFiles.value = files
}

const handleStatusMessage = ({ message, type }) => {
  statusMessage.value = message
  statusType.value = type
}

const getStatusIcon = (type) => {
  const icons = {
    success: '✅',
    error: '❌',
    info: 'ℹ️',
    warning: '⚠️'
  }
  return icons[type] || 'ℹ️'
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
    statusMessage.value = 'Lỗi khi chọn thư mục: ' + error
    statusType.value = 'error'
  }
}

const extractAudio = async () => {
  if (videoFiles.value.length === 0) {
    statusMessage.value = 'Vui lòng chọn video'
    statusType.value = 'error'
    return
  }
  
  if (!outputFolder.value) {
    statusMessage.value = 'Vui lòng chọn thư mục lưu'
    statusType.value = 'error'
    return
  }

  isCreating.value = true
  statusMessage.value = 'Đang trích xuất audio...'
  statusType.value = 'info'
  
  const timestamp = Date.now()
  const processId = `extract_audio_${timestamp}`
  currentProcessId.value = processId
  
  try {
    const inputPath = videoFiles.value[0]
    const inputFileName = inputPath.split(/[\\/]/).pop().split('.').slice(0, -1).join('.')
    const outputFileName = `${inputFileName}.${audioFormat.value}`
    
    const separator = outputFolder.value.includes('\\') ? '\\' : '/'
    const outputPath = outputFolder.value.endsWith(separator) 
      ? `${outputFolder.value}${outputFileName}`
      : `${outputFolder.value}${separator}${outputFileName}`

    await callCommand('extract_audio_from_video', {
      videoPath: inputPath,
      outputPath: outputPath,
      format: audioFormat.value,
      processId: processId
    })
    
    statusMessage.value = `Trích xuất thành công! File lưu tại: ${outputFileName}`
    statusType.value = 'success'
  } catch (error) {
    statusMessage.value = 'Lỗi: ' + error
    statusType.value = 'error'
  } finally {
    isCreating.value = false
    currentProcessId.value = null
  }
}

const stopExtraction = async () => {
  if (!currentProcessId.value) return
  
  try {
    await callCommand('stop_video_video_creation', {
      processId: currentProcessId.value
    })
    statusMessage.value = 'Đã dừng trích xuất'
    statusType.value = 'info'
    isCreating.value = false
    currentProcessId.value = null
  } catch (error) {
    statusMessage.value = 'Lỗi khi dừng: ' + error
    statusType.value = 'error'
  }
}
</script>

<style scoped>
.extract-audio-page {
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
  background: rgba(16, 185, 129, 0.2);
  width: 80px;
  height: 80px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 20px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
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
  grid-template-columns: 2fr 1fr;
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

.format-selector {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.radio-group {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.radio-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  background: rgba(15, 23, 42, 0.4);
  border: 1px solid rgba(255, 255, 255, 0.05);
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.2s;
}

.radio-item:hover {
  background: rgba(255, 255, 255, 0.05);
}

.radio-item.active {
  background: rgba(16, 185, 129, 0.1);
  border-color: rgba(16, 185, 129, 0.3);
}

.radio-item input {
  display: none;
}

.folder-selector {
  display: flex;
  gap: 12px;
}

.folder-input {
  flex: 1;
  background: rgba(15, 23, 42, 0.5);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 12px;
  padding: 12px 16px;
  color: #94a3b8;
  font-size: 14px;
}

.btn-select-folder {
  background: #334155;
  border: none;
  border-radius: 12px;
  padding: 0 20px;
  color: white;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 8px;
  transition: all 0.2s;
}

.btn-primary {
  background: linear-gradient(135deg, #10b981 0%, #059669 100%);
  border: none;
  border-radius: 16px;
  padding: 18px;
  color: white;
  font-weight: 600;
  font-size: 16px;
  cursor: pointer;
  transition: all 0.3s;
}

.btn-primary:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 8px 20px rgba(16, 185, 129, 0.3);
}

.btn-secondary {
  background: rgba(244, 63, 94, 0.1);
  border: 1px solid rgba(244, 63, 94, 0.2);
  border-radius: 16px;
  padding: 14px;
  color: #fb7185;
  font-weight: 600;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
}

.status-message {
  padding: 16px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  gap: 12px;
  font-size: 14px;
}

.status-message.success { background: rgba(16, 185, 129, 0.1); color: #34d399; }
.status-message.error { background: rgba(239, 68, 68, 0.1); color: #f87171; }
.status-message.info { background: rgba(59, 130, 246, 0.1); color: #60a5fa; }

.spinner {
  width: 20px;
  height: 20px;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-top-color: white;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin { to { transform: rotate(360deg); } }
</style>
