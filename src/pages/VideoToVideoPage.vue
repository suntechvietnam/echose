<template>
  <div class="video-merge-page">
    <!-- Hero Header -->
    <div class="page-hero">
      <div class="hero-content">
        <div class="hero-icon">🎬</div>
        <div class="hero-text">
          <h1 class="hero-title">Ghép Video Chuyên Nghiệp</h1>
          <p class="hero-subtitle">Nhanh chóng, chất lượng cao với công nghệ HEVC</p>
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
              Chọn Video
            </h3>
          </div>
          
          <VideoAspectRatioSection v-model="videoAspectRatio" />
          
          <ImportVideoSection 
            :modelValue="videoFiles"
            @update:videoFiles="handleVideoFilesUpdate"
            @videos-selected="handleVideosSelected"
            @videos-cleared="handleVideosCleared"
            @status-message="handleStatusMessage"
            @update:removeOriginalAudio="handleRemoveOriginalAudioUpdate"
          />
        </div>
      </div>

      <!-- Right Column: Settings & Actions -->
      <div class="right-panel">
        <!-- Merge Mode Card -->
        <div class="glass-card mode-card">
          <div class="card-header">
            <h3 class="card-title">
              <span class="title-icon">⚡</span>
              Chế Độ Ghép
            </h3>
          </div>
          
          <div class="mode-selector-inline">
            <label class="mode-radio" :class="{ active: mergeMode === 'fast' }">
              <input type="radio" value="fast" v-model="mergeMode" />
              <span class="radio-content">
                <span class="radio-icon">🚀</span>
                <span class="radio-text">Siêu tốc (cùng tỉ lệ, cùng chất lượng)</span>
              </span>
            </label>

            <label class="mode-radio" :class="{ active: mergeMode === 'convert' }">
              <input type="radio" value="convert" v-model="mergeMode" />
              <span class="radio-content">
                <span class="radio-icon">🔄</span>
                <span class="radio-text">Chuyển đổi (có thể ghép khác tỉ lệ, chất lượng)</span>
              </span>
            </label>
          </div>
        </div>

        <!-- Quality Settings (Convert Mode) -->
        <div v-if="mergeMode === 'convert'" class="glass-card quality-card">
          <div class="card-header">
            <h3 class="card-title">
              <span class="title-icon">🎯</span>
              Chất Lượng Đầu Ra
            </h3>
          </div>
          
          <div class="quality-buttons">
            <button 
              v-for="quality in qualityOptions" 
              :key="quality.value"
              class="quality-btn"
              :class="{ active: videoQuality === quality.value }"
              @click="videoQuality = quality.value"
            >
              <span class="quality-label">{{ quality.label }}</span>
            </button>
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
              placeholder="Chọn thư mục để lưu video..."
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
            @click="createVideoFromVideos" 
            :disabled="isCreating || !videoFiles.length || !outputFolder"
            :class="{ loading: isCreating }"
          >
            <span v-if="!isCreating" class="btn-content">
              <span class="btn-icon">✨</span>
              <span>Ghép Video</span>
            </span>
            <span v-else class="btn-content">
              <span class="spinner"></span>
              <span>Đang xử lý...</span>
            </span>
          </button>

          <button 
            v-if="isCreating"
            class="btn-secondary btn-cancel" 
            @click="stopVideoCreation"
          >
            <span class="btn-icon">⏹️</span>
            Huỷ
          </button>
        </div>

        <!-- Progress & Status -->
        <div v-if="isCreating" class="progress-section">
          <div class="progress-bar-wrapper">
            <div class="progress-bar">
              <div class="progress-fill" :style="{ width: progress + '%' }"></div>
            </div>
            <div class="progress-text">{{ progress }}%</div>
          </div>
        </div>

        <div v-if="statusMessage" :class="['status-message', statusType]">
          <span class="status-icon">{{ getStatusIcon(statusType) }}</span>
          <span class="status-text">{{ statusMessage }}</span>
          <button 
            v-if="statusType === 'success' && videoFolderPath"
            class="btn-open-folder" 
            @click="openVideoFolderFromPath"
            title="Mở thư mục"
          >
            📂
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { useTauri } from '../composables/useTauri'
import ImportVideoSection from '@/components/ImportVideoSection.vue'
import VideoAspectRatioSection from '@/components/VideoAspectRatioSection.vue'

const { callCommand } = useTauri()

// Quality options
const qualityOptions = [
  { value: 'hd', label: '720', resolution: '720p', icon: '📱' },
  { value: 'fullhd', label: '1080', resolution: '1080p', icon: '💻' },
  { value: '2K', label: '2K', resolution: '2048×1080', icon: '🖥️' },
  { value: '4K', label: '4K', resolution: '3840×2160', icon: '📺' }
]

// State
const videoFiles = ref([])
const removeOriginalAudio = ref(false)
const videoQuality = ref('fullhd')
const videoAspectRatio = ref('16:9')
const videoEffectType = ref('diagtl')
const mergeMode = ref('fast')
const outputFolder = ref('')
const isCreating = ref(false)
const statusMessage = ref('')
const statusType = ref('info')
const progress = ref(0)
const currentProcessId = ref(null)
const videoFolderPath = ref(null)
const videoHistory = ref([])

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
      const folderPath = Array.isArray(selected) ? selected[0] : selected
      outputFolder.value = folderPath
    }
  } catch (error) {
    statusMessage.value = 'Lỗi khi chọn thư mục: ' + error
    statusType.value = 'error'
  }
}

const createVideoFromVideos = async () => {
  if (videoFiles.value.length === 0) {
    statusMessage.value = 'Vui lòng chọn ít nhất một video'
    statusType.value = 'error'
    return
  }
  
  if (!outputFolder.value.trim()) {
    statusMessage.value = 'Vui lòng chọn thư mục để lưu video'
    statusType.value = 'error'
    return
  }
  
  isCreating.value = true
  progress.value = 5
  statusMessage.value = 'Đang bắt đầu ghép video...'
  statusType.value = 'info'
  videoFolderPath.value = null
  
  // Tạo process_id để có thể stop
  const timestampForId = Date.now();
  const processId = `video_video_${timestampForId}`;
  currentProcessId.value = processId;
  
  try {
    progress.value = 20
    statusMessage.value = 'Đang ghép video...'
    
    const crfMap = {
      'hd': '23',
      'fullhd': '22',
      '2K': '20',
      '4K': '20'
    }
    const crf = crfMap[videoQuality.value] || '22'
    
    const timestamp = Date.now()
    const outputFileName = `final_video_${timestamp}.mp4`
    const outputFolderPath = outputFolder.value.trim()
    const separator = outputFolderPath.includes('\\') ? '\\' : '/'
    const outputPath = outputFolderPath.endsWith(separator) 
      ? `${outputFolderPath}${outputFileName}`
      : `${outputFolderPath}${separator}${outputFileName}`
    
    const result = await callCommand('create_video_from_video', {
      videoFiles: videoFiles.value,
      audioFiles: [],
      outputPath: outputPath,
      crf: crf,
      isHasAutoCaption: false,
      videoEffectType: videoEffectType.value,
      videoQuality: videoQuality.value,
      videoAspectRatio: videoAspectRatio.value,
      removeOriginalAudio: removeOriginalAudio.value,
      mergeMode: mergeMode.value,
      processId: processId
    })
    
    progress.value = 100
    
    if (result.includes('Lỗi')) {
      throw new Error(result)
    }
    
    const videoFileNameMatch = result.match(/Video đã được tạo thành công!\s*(.+)/i)
    if (videoFileNameMatch) {
      const videoFilePath = videoFileNameMatch[1].trim()
      const videoFileName = videoFilePath.split('/').pop() || videoFilePath.split('\\').pop() || videoFilePath
      statusMessage.value = `Video đã được tạo thành công! ${videoFileName}`
      
      if (outputFolder.value) {
        videoFolderPath.value = outputFolder.value
      }
      
      videoHistory.value.unshift({
        fileName: videoFileName,
        outputPath: videoFilePath,
        videoCount: videoFiles.value.length,
        quality: videoQuality.value,
        createdAt: new Date().toISOString()
      })
      if (videoHistory.value.length > 20) {
        videoHistory.value = videoHistory.value.slice(0, 20)
      }
    } else {
      statusMessage.value = result
    }
    
    statusType.value = 'success'
  } catch (error) {
    if (error.toString().includes('đã bị hủy') || error.toString().includes('cancelled')) {
      statusMessage.value = 'Quá trình ghép video đã bị hủy'
      statusType.value = 'info'
    } else {
      statusMessage.value = 'Lỗi: ' + error
      statusType.value = 'error'
    }
  } finally {
    isCreating.value = false
  }
}

const stopVideoCreation = async () => {
  if (!currentProcessId.value) return
  
  try {
    await callCommand('stop_video_video_creation', {
      processId: currentProcessId.value
    })
    currentProcessId.value = null
    statusMessage.value = 'Đã dừng quá trình ghép video'
    statusType.value = 'info'
    isCreating.value = false
  } catch (error) {
    statusMessage.value = 'Lỗi khi dừng: ' + error
    statusType.value = 'error'
  }
}

const openVideoFolderFromPath = async () => {
  if (!videoFolderPath.value) return
  
  try {
    await callCommand('open_folder', { path: videoFolderPath.value })
  } catch (error) {
    statusMessage.value = 'Lỗi khi mở thư mục: ' + error
    statusType.value = 'error'
  }
}

const handleVideoFilesUpdate = (files) => {
  videoFiles.value = files
}

const handleVideosSelected = (files) => {
  statusType.value = 'success'
}

const handleVideosCleared = () => {
  // Clear any related state if needed
}

const handleStatusMessage = (message, type) => {
  statusMessage.value = message
  statusType.value = type
}

const handleRemoveOriginalAudioUpdate = (value) => {
  removeOriginalAudio.value = value
}
</script>

<style scoped>
.video-merge-page {
  min-height: 100vh;
  background: linear-gradient(135deg, #0f172a 0%, #1e293b 100%);
  padding: 2rem;
}

/* Hero Section */
.page-hero {
  margin-bottom: 2rem;
}

.hero-content {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.hero-icon {
  font-size: 3rem;
}

.hero-text {
  flex: 1;
}

.hero-title {
  font-size: 1.75rem;
  font-weight: 700;
  background: linear-gradient(135deg, #60a5fa 0%, #a78bfa 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  margin-bottom: 0.25rem;
}

.hero-subtitle {
  font-size: 0.95rem;
  color: #94a3b8;
  font-weight: 400;
  margin: 0;
}

/* Content Layout */
.content-wrapper {
  display: grid;
  grid-template-columns: 2fr 1fr;
  gap: 2rem;
  max-width: 1400px;
  margin: 0 auto;
}

@media (max-width: 1024px) {
  .content-wrapper {
    grid-template-columns: 1fr;
  }
}

/* Glass Card */
.glass-card {
  background: rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(10px);
  border-radius: 20px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  padding: 1.5rem;
  margin-bottom: 1.5rem;
  transition: all 0.3s ease;
}

.glass-card:hover {
  background: rgba(255, 255, 255, 0.08);
  border-color: rgba(255, 255, 255, 0.2);
  transform: translateY(-2px);
}

.card-header {
  margin-bottom: 1.5rem;
}

.card-title {
  font-size: 1.25rem;
  font-weight: 600;
  color: #f1f5f9;
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.title-icon {
  font-size: 1.5rem;
}

/* Mode Selector - Inline Radio */
.mode-selector-inline {
  display: flex;
  gap: 1rem;
}

.mode-radio {
  flex: 1;
  position: relative;
  cursor: pointer;
}

.mode-radio input[type="radio"] {
  position: absolute;
  opacity: 0;
}

.radio-content {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  padding: 0.875rem 1.25rem;
  background: rgba(255, 255, 255, 0.03);
  border: 2px solid rgba(255, 255, 255, 0.1);
  border-radius: 10px;
  transition: all 0.3s ease;
}

.mode-radio:hover .radio-content {
  background: rgba(255, 255, 255, 0.08);
  border-color: rgba(96, 165, 250, 0.5);
}

.mode-radio.active .radio-content {
  background: linear-gradient(135deg, rgba(96, 165, 250, 0.2) 0%, rgba(167, 139, 250, 0.2) 100%);
  border-color: #60a5fa;
}

.radio-icon {
  font-size: 1.25rem;
}

.radio-text {
  font-size: 0.95rem;
  font-weight: 600;
  color: #f1f5f9;
}

/* Quality Buttons - Inline */
.quality-buttons {
  display: flex;
  gap: 0.75rem;
}

.quality-btn {
  flex: 1;
  padding: 0.75rem 1rem;
  background: rgba(255, 255, 255, 0.03);
  border: 2px solid rgba(255, 255, 255, 0.1);
  border-radius: 10px;
  color: #f1f5f9;
  font-size: 0.9rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s ease;
}

.quality-btn:hover {
  background: rgba(255, 255, 255, 0.08);
  border-color: rgba(96, 165, 250, 0.5);
  transform: translateY(-2px);
}

.quality-btn.active {
  background: linear-gradient(135deg, rgba(96, 165, 250, 0.2) 0%, rgba(167, 139, 250, 0.2) 100%);
  border-color: #60a5fa;
  color: #60a5fa;
}

/* Folder Selector */
.folder-selector {
  display: flex;
  gap: 0.75rem;
}

.folder-input {
  flex: 1;
  padding: 0.875rem 1rem;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 10px;
  color: #f1f5f9;
  font-size: 0.9rem;
  transition: all 0.3s ease;
}

.folder-input:focus {
  outline: none;
  border-color: #60a5fa;
  background: rgba(255, 255, 255, 0.08);
}

.btn-select-folder {
  padding: 0.875rem 1.5rem;
  background: linear-gradient(135deg, #3b82f6 0%, #8b5cf6 100%);
  border: none;
  border-radius: 10px;
  color: white;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s ease;
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.btn-select-folder:hover {
  transform: translateY(-2px);
  box-shadow: 0 10px 30px rgba(59, 130, 246, 0.3);
}

/* Action Buttons */
.action-section {
  display: flex;
  gap: 1rem;
  margin-top: 1.5rem;
}

.btn-primary, .btn-secondary {
  flex: 1;
  padding: 1rem 2rem;
  border: none;
  border-radius: 12px;
  font-size: 1rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s ease;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
}

.btn-create {
  background: linear-gradient(135deg, #10b981 0%, #059669 100%);
  color: white;
}

.btn-create:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 10px 30px rgba(16, 185, 129, 0.4);
}

.btn-create:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-create.loading {
  background: linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%);
}

.btn-cancel {
  background: rgba(239, 68, 68, 0.2);
  color: #ef4444;
  border: 1px solid rgba(239, 68, 68, 0.3);
}

.btn-cancel:hover {
  background: rgba(239, 68, 68, 0.3);
}

.btn-content {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.spinner {
  width: 16px;
  height: 16px;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-top-color: white;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

/* Progress */
.progress-section {
  margin-top: 1.5rem;
}

.progress-bar-wrapper {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.progress-bar {
  flex: 1;
  height: 8px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 10px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, #10b981 0%, #059669 100%);
  border-radius: 10px;
  transition: width 0.3s ease;
}

.progress-text {
  font-size: 0.875rem;
  font-weight: 600;
  color: #10b981;
  min-width: 45px;
}

/* Status Message */
.status-message {
  margin-top: 1.5rem;
  padding: 1rem 1.25rem;
  border-radius: 10px;
  display: flex;
  align-items: center;
  gap: 0.75rem;
  font-size: 0.9rem;
  animation: slideIn 0.3s ease;
}

@keyframes slideIn {
  from {
    opacity: 0;
    transform: translateY(-10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.status-message.success {
  background: rgba(16, 185, 129, 0.15);
  border: 1px solid rgba(16, 185, 129, 0.3);
  color: #10b981;
}

.status-message.error {
  background: rgba(239, 68, 68, 0.15);
  border: 1px solid rgba(239, 68, 68, 0.3);
  color: #ef4444;
}

.status-message.info {
  background: rgba(59, 130, 246, 0.15);
  border: 1px solid rgba(59, 130, 246, 0.3);
  color: #3b82f6;
}

.status-icon {
  font-size: 1.25rem;
}

.status-text {
  flex: 1;
}

.btn-open-folder {
  padding: 0.5rem 0.75rem;
  background: rgba(255, 255, 255, 0.1);
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 1.25rem;
  transition: all 0.2s ease;
}

.btn-open-folder:hover {
  background: rgba(255, 255, 255, 0.2);
  transform: scale(1.1);
}
</style>
