<template>
  <div class="page-content">
    <div class="page-header">
      <h2>Ghép video</h2>
    </div>
    <div class="page-body image-to-video-container">
      <!-- Layout 2 cột -->
      <div class="image-to-video-layout">
        <!-- Cột trái: Chọn và hiển thị danh sách video -->
        <div class="image-to-video-left-column">
          <VideoAspectRatioSection
            v-model="videoAspectRatio"
          />

          <ImportVideoSection 
            :modelValue="videoFiles"
            @update:videoFiles="handleVideoFilesUpdate"
            @videos-selected="handleVideosSelected"
            @videos-cleared="handleVideosCleared"
            @status-message="handleStatusMessage"
            @update:removeOriginalAudio="handleRemoveOriginalAudioUpdate"
          />

          <!-- Audio Selection Section -->
          <ImportAudioSection 
            :modelValue="audioFiles"
            @update:audioFiles="handleAudioFilesUpdate"
            @audio-selected="handleAudioSelected"
            @audio-cleared="handleAudioCleared"
          />
        </div>

        <!-- Cột phải: Options và controls -->
        <div class="image-to-video-right-column">
          <!-- Ở trang ghép video không hiển thị cấu hình "Thời gian mỗi ảnh" và "Hiệu ứng ảnh" -->

          <div class="file-group">
            <div class="form-group">
              <label class="form-label">📺 Chất lượng video</label>
              <select v-model="videoQuality" class="form-select">
                <option value="hd">720 (HD)</option>
                <option value="fullhd">1080 (Full HD)</option>
                <option value="2K">2K (2048x1080)</option>
                <option value="4K">4K (3840x2160)</option>
              </select>
            </div>

            <VideoEffect 
              :selectedEffect="videoEffectType"
              @update:selectedEffect="videoEffectType = $event"
            />
          </div>

          <div class="file-group">
            <div class="form-group">
              <label class="form-label">💾 Thư mục lưu video</label>
              <div class="input-group">
                <input 
                  type="text" 
                  v-model="outputFolder"
                  class="form-input" 
                  placeholder="Chọn thư mục để lưu video..."
                  readonly
                />
                <button class="btn-select-folder" @click="selectOutputFolder">
                  Chọn
                </button>
              </div>
            </div>
            
            <div class="form-group auto-caption-group">
              <div class="checkbox-container">
                <input 
                  type="checkbox" 
                  id="auto-caption-checkbox-v2v"
                  v-model="isAutoCaption"
                  @click="handleAutoCaptionClick"
                  class="checkbox-input"
                />
                <label for="auto-caption-checkbox-v2v" class="checkbox-label">
                  🎬 Auto caption
                </label>
              </div>
            </div>
          </div>

          <!-- Button tạo video -->
          <div class="form-actions">
            <button 
              class="btn-create-video" 
              @click="createVideoFromVideos" 
              :disabled="isCreating"
            >
              {{ isCreating ? 'Đang ghép video...' : 'Ghép video' }}
            </button>
            <button 
              v-if="isCreating"
              class="btn-stop-video" 
              @click="stopVideoCreation"
            >
              Huỷ tiến trình
            </button>
          </div>

          <!-- Status và Progress -->
          <div :class="['download-status', statusType]" v-if="statusMessage">
            <span>{{ statusMessage }}</span>
            <button 
              v-if="statusType === 'success' && videoFolderPath"
              class="btn-open-folder-inline" 
              @click="openVideoFolderFromPath"
              title="Mở thư mục chứa video"
            >
              📁
            </button>
          </div>
          <div v-if="isCreating" class="download-progress">
            <div class="progress-bar">
              <div class="progress-fill" :style="{ width: progress + '%' }"></div>
            </div>
            <div class="progress-text">{{ progress }}%</div>
          </div>
        </div>
      </div>
    </div>
    
    <!-- Audio Required Modal -->
    <ConfirmModal 
      v-if="isShowAudioRequiredModal"
      :title="'Thông báo'"
      :message="'Bạn cần chọn file audio để dùng tính năng auto caption'"
      :type="'info'"
      :confirmText="'OK'"
      :cancelText="''"
      @confirm="closeAudioRequiredModal"
    />
  </div>
</template>

<script setup>
import { ref } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { useTauri } from '../composables/useTauri'
import { useAudioDuration } from '../composables/useAudioDuration'
import ImportAudioSection from '@/components/ImportAudioSection.vue'
import ImportVideoSection from '@/components/ImportVideoSection.vue'
import VideoEffect from '@/components/VideoEffect.vue'
import ConfirmModal from '@/components/ConfirmModal.vue'
import VideoAspectRatioSection from '@/components/VideoAspectRatioSection.vue'
import '../assets/css/image-to-video.css'

const { callCommand } = useTauri()
const { loadFileDuration } = useAudioDuration()

// Video files - store file paths
const videoFiles = ref([])

// Audio files - store file paths
const audioFiles = ref([])

// Remove original audio from videos
const removeOriginalAudio = ref(false)

// Configuration (vẫn giữ giống trang ImageToVideoPage, nhưng không hiển thị UI thời gian / hiệu ứng ảnh)
const imageDuration = ref(6)
const durationOptions = [5, 6, 7, 8, 10, 12, 15, 20, 30]
const videoQuality = ref('fullhd')
const videoAspectRatio = ref('16:9') // Default: 16:9 (Video dài)
const videoEffectType = ref('diagtl')
const imageEffectType = ref('zoom-in')

const imageEffectOptions = [
  { name: 'Không có', value: 'none' },
  { name: 'Zoom in', value: 'zoom-in' },
  { name: 'Zoom out', value: 'zoom-out' },
  { name: 'Fade in', value: 'fade-in' },
  { name: 'Fade out', value: 'fade-out' },
  { name: 'Pan', value: 'pan' },
]

// Output
const outputFolder = ref('')
const isAutoCaption = ref(false)
const isCreating = ref(false)
const statusMessage = ref('')
const statusType = ref('info')
const isShowAudioRequiredModal = ref(false)
const progress = ref(0)
const currentProcessId = ref(null)
const videoFolderPath = ref(null)

// Video history
const videoHistory = ref([])

// Video aspect ratio được điều khiển qua component VideoAspectRatioSection

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
  
  if (imageDuration.value < 5) {
    statusMessage.value = 'Khoảng cách giữa các phần phải tối thiểu 5 giây'
    statusType.value = 'error'
    return
  }
  
  // Validate audio nếu có - Tùy chọn không bắt buộc
  if (audioFiles.value.length > 0) {
    try {
      statusMessage.value = '⏳ Đang kiểm tra thời gian audio...'
      statusType.value = 'info'
      
      // Tính tổng thời gian video từ video input (tạm thời sử dụng công thức giống ảnh)
      const totalVideoTime = (videoFiles.value.length * imageDuration.value) + videoFiles.value.length
      
      // Tính tổng thời gian audio
      let totalAudioTime = 0
      for (const audioFile of audioFiles.value) {
        const audioDuration = await loadFileDuration(audioFile)
        if (audioDuration) {
          totalAudioTime += audioDuration
        }
      }
      
      if (totalAudioTime < totalVideoTime) {
        statusMessage.value = `❌ Thời gian audio (${Math.round(totalAudioTime)}s) ngắn hơn thời gian video (${totalVideoTime}s). Vui lòng thêm audio hoặc giảm thời lượng.`
        statusType.value = 'error'
        return
      }
      
    } catch (error) {
      statusMessage.value = '❌ Lỗi khi kiểm tra audio: ' + error
      statusType.value = 'error'
      return
    }
  } else {
    statusMessage.value = '📹 Ghép video không có âm thanh'
    statusType.value = 'info'
  }
  
  isCreating.value = true
  progress.value = 5
  statusMessage.value = '⏳ Đang bắt đầu ghép video...'
  statusType.value = 'info'
  videoFolderPath.value = null
  
  try {
    progress.value = 20
    statusMessage.value = '⏳ Đang ghép video...'
    
    // Tính CRF dựa trên video quality
    const crfMap = {
      'hd': '23',
      'fullhd': '22',
      '2K': '20',
      '4K': '20'
    }
    const crf = crfMap[videoQuality.value] || '22'
    
    // Tạo output path với timestamp
    const timestamp = Date.now()
    const outputFileName = `final_video_${timestamp}.mp4`
    // Sử dụng path.join để xử lý đúng trên mọi platform
    const outputFolderPath = outputFolder.value.trim()
    const separator = outputFolderPath.includes('\\') ? '\\' : '/'
    const outputPath = outputFolderPath.endsWith(separator) 
      ? `${outputFolderPath}${outputFileName}`
      : `${outputFolderPath}${separator}${outputFileName}`
    
    const result = await callCommand('create_video_from_video', {
      videoFiles: videoFiles.value,
      audioFiles: audioFiles.value,
      outputPath: outputPath,
      crf: crf,
      isHasAutoCaption: isAutoCaption.value,
      videoEffectType: videoEffectType.value,
      videoQuality: videoQuality.value,
      videoAspectRatio: videoAspectRatio.value,
      removeOriginalAudio: removeOriginalAudio.value
    })
    
    progress.value = 100
    
    if (result.includes('Lỗi')) {
      throw new Error(result)
    }
    
    const videoFileNameMatch = result.match(/Video đã được tạo thành công!\s*(.+)/i)
    if (videoFileNameMatch) {
      const videoFilePath = videoFileNameMatch[1].trim()
      const videoFileName = videoFilePath.split('/').pop() || videoFilePath.split('\\').pop() || videoFilePath
      statusMessage.value = `✅ Video đã được tạo thành công! ${videoFileName}`
      
      if (outputFolder.value) {
        videoFolderPath.value = outputFolder.value
      }
      
      videoHistory.value.unshift({
        fileName: videoFileName,
        outputPath: videoFilePath,
        videoCount: videoFiles.value.length,
        quality: videoQuality.value,
        duration: imageDuration.value,
        effect: imageEffectType.value,
        createdAt: new Date().toISOString()
      })
      if (videoHistory.value.length > 20) {
        videoHistory.value = videoHistory.value.slice(0, 20)
      }
    } else {
      statusMessage.value = '✅ ' + result
    }
    
    statusType.value = 'success'
  } catch (error) {
    if (error.toString().includes('đã bị hủy') || error.toString().includes('cancelled')) {
      statusMessage.value = '⚠️ Quá trình ghép video đã bị hủy'
      statusType.value = 'info'
    } else {
      statusMessage.value = '❌ Lỗi: ' + error
      statusType.value = 'error'
    }
  } finally {
    isCreating.value = false
  }
}

const stopVideoCreation = async () => {
  if (!currentProcessId.value) return
  
  try {
    await callCommand('stop_image_video_creation', {
      processId: currentProcessId.value
    })
    currentProcessId.value = null
    statusMessage.value = '⚠️Đã dừng quá trình ghép video và xóa toàn bộ file tạm'
    statusType.value = 'info'
    isCreating.value = false
  } catch (error) {
    statusMessage.value = '❌Lỗi khi dừng: ' + error
    statusType.value = 'error'
  }
}

const openVideoFolderFromPath = async () => {
  if (!videoFolderPath.value) return
  
  try {
    await callCommand('open_folder', { path: videoFolderPath.value })
  } catch (error) {
    statusMessage.value = '❌ Lỗi khi mở thư mục: ' + error
    statusType.value = 'error'
  }
}

// Video handlers
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

const handleAutoCaptionClick = (event) => {
  if (event.target.checked && audioFiles.value.length === 0) {
    event.preventDefault()
    isAutoCaption.value = false // Reset checkbox
    isShowAudioRequiredModal.value = true
  }
}

const closeAudioRequiredModal = () => {
  isShowAudioRequiredModal.value = false
}

// Audio handlers
const handleAudioFilesUpdate = (files) => {
  audioFiles.value = files
  
  if (files.length === 0 && isAutoCaption.value) {
    isAutoCaption.value = false
  }
}

const handleAudioSelected = (files) => {
  // Không cần thông báo khi chọn audio
}

const handleAudioCleared = () => {
  if (isAutoCaption.value) {
    isAutoCaption.value = false
  }
}
</script>


