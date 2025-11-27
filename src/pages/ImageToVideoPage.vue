<template>
  <div class="page-content">
    <div class="page-header">
      <h2>Tạo video từ ảnh</h2>
    </div>
    <div class="page-body image-to-video-container">
      <!-- Layout 2 cột -->
      <div class="image-to-video-layout">
        <!-- Cột trái: Chọn và hiển thị danh sách ảnh -->
        <div class="image-to-video-left-column">
          <!-- Video size -->
          <div class="file-group video-size-group">
            <h3 class="form-label">Chọn loại video</h3>
            <div class="video-aspect-tabs">
              <button 
                :class="['aspect-tab', { active: videoAspectRatio === '16:9' }]"
                @click="changeVideoAspectRatio('16:9')"
              >
                Video dài (16:9)
              </button>
              <button 
                :class="['aspect-tab', { active: videoAspectRatio === '9:16' }]"
                @click="changeVideoAspectRatio('9:16')"
              >
                Video ngắn (9:16)
              </button>
            </div>
          </div>

          <ImportImageSection 
            :modelValue="imageFiles"
            @update:imageFiles="handleImageFilesUpdate"
            @images-selected="handleImagesSelected"
            @images-cleared="handleImagesCleared"
            @status-message="handleStatusMessage"
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
          <!-- Cấu hình video -->
          <div class="file-group">
            <div class="form-group">
              <div class="form-group image-settings-group">
                <div class="image-settings-item">
                  <label class="form-label">⏱️ Thời gian mỗi ảnh</label>
                  <select v-model="imageDuration" class="form-select">
                    <option v-for="duration in durationOptions" :key="duration" :value="duration">
                      {{ duration }} giây
                    </option>
                  </select>
                </div>

                <div class="image-settings-item">
                  <label class="form-label">✨ Hiệu ứng ảnh</label>
                  <select v-model="imageEffectType" class="form-select">
                    <option v-for="imageEffect in imageEffectOptions" :key="imageEffect.value" :value="imageEffect.value">
                      {{ imageEffect.name }}
                    </option>
                  </select>
                </div>
              </div>
            </div>
          </div>

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
                  id="auto-caption-checkbox"
                  v-model="isAutoCaption"
                  class="checkbox-input"
                />
                <label for="auto-caption-checkbox" class="checkbox-label">
                  🎬 Auto caption
                </label>
              </div>
            </div>
          </div>

          <!-- Button tạo video -->
          <div class="form-actions">
            <button 
              class="btn-create-video" 
              @click="createVideoFromImages" 
              :disabled="isCreating"
            >
              {{ isCreating ? 'Đang tạo video...' : 'Tạo Video' }}
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
  </div>
</template>

<script setup>
import { ref } from 'vue'
import { open } from '@tauri-apps/api/dialog'
import { useTauri } from '../composables/useTauri'
import { useAudioDuration } from '../composables/useAudioDuration'
import ImportAudioSection from '@/components/ImportAudioSection.vue'
import ImportImageSection from '@/components/ImportImageSection.vue'
import VideoEffect from '@/components/VideoEffect.vue'
import '../assets/css/image-to-video.css'

const { callCommand } = useTauri()
const { loadFileDuration } = useAudioDuration()

// Image files - store file paths
const imageFiles = ref([])

// Audio files - store file paths
const audioFiles = ref([])

// Configuration
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
const progress = ref(0)
const currentProcessId = ref(null)
const videoFolderPath = ref(null)

// Video history
const videoHistory = ref([])

const changeVideoAspectRatio = (aspectRatio) => {
  videoAspectRatio.value = aspectRatio
}

const selectOutputFolder = async () => {
  try {
    // Sử dụng Tauri Dialog API - nhất quán với selectImageFiles
    const selected = await open({
      directory: true,
      multiple: false
    })
    
    if (selected) {
      const folderPath = Array.isArray(selected) ? selected[0] : selected
      outputFolder.value = folderPath
      statusMessage.value = '✅ Đã chọn thư mục: ' + folderPath
      statusType.value = 'success'
    }
  } catch (error) {
    statusMessage.value = 'Lỗi khi chọn thư mục: ' + error
    statusType.value = 'error'
  }
}

const createVideoFromImages = async () => {
  if (imageFiles.value.length === 0) {
    statusMessage.value = 'Vui lòng chọn ít nhất một ảnh'
    statusType.value = 'error'
    return
  }
  
  if (!outputFolder.value.trim()) {
    statusMessage.value = 'Vui lòng chọn thư mục để lưu video'
    statusType.value = 'error'
    return
  }
  
  if (imageDuration.value < 5) {
    statusMessage.value = 'Khoảng cách giữa các ảnh phải tối thiểu 5 giây'
    statusType.value = 'error'
    return
  }
  
  // Validate audio nếu có - Tùy chọn không bắt buộc
  if (audioFiles.value.length > 0) {
    try {
      statusMessage.value = '⏳ Đang kiểm tra thời gian audio...'
      statusType.value = 'info'
      
      // Tính tổng thời gian video từ ảnh (giây)
      // Công thức: (số ảnh * thời gian mỗi ảnh) + số ảnh (cho transition)
      const totalVideoTime = (imageFiles.value.length * imageDuration.value) + imageFiles.value.length
      
      // Tính tổng thời gian audio
      let totalAudioTime = 0
      for (const audioFile of audioFiles.value) {
        const audioDuration = await loadFileDuration(audioFile)
        if (audioDuration) {
          totalAudioTime += audioDuration
        }
      }
      
      // Kiểm tra xem audio có đủ dài không
      if (totalAudioTime < totalVideoTime) {
        statusMessage.value = `❌ Thời gian audio (${Math.round(totalAudioTime)}s) ngắn hơn thời gian video (${totalVideoTime}s). Vui lòng thêm audio hoặc giảm thời gian mỗi ảnh.`
        statusType.value = 'error'
        return
      }
      
    } catch (error) {
      statusMessage.value = '❌ Lỗi khi kiểm tra audio: ' + error
      statusType.value = 'error'
      return
    }
  } else {
    // Không có audio - tạo video không âm thanh
    statusMessage.value = '📹 Tạo video không có âm thanh'
    statusType.value = 'info'
  }
  
  isCreating.value = true
  progress.value = 5
  statusMessage.value = '⏳ Đang bắt đầu tạo video từ ảnh...'
  statusType.value = 'info'
  videoFolderPath.value = null
  
  try {
    progress.value = 20
    statusMessage.value = '⏳ Đang tạo video từ ảnh...'
    
    console.log('🎬 Creating video with audio files:', audioFiles.value)
    
    const result = await callCommand('create_video_from_images', {
      imageFiles: imageFiles.value,
      imageDuration: imageDuration.value,
      imageEffectType: imageEffectType.value,
      videoQuality: videoQuality.value,
      videoAspectRatio: videoAspectRatio.value,
      videoEffectType: videoEffectType.value,
      audioFiles: audioFiles.value,
      outputFolder: outputFolder.value.trim(),
      isAutoCaption: isAutoCaption.value
    })
    
    progress.value = 100
    
    if (result.includes('Lỗi')) {
      throw new Error(result)
    }
    
    // Parse result để lấy tên file và folder path
    // Format: "Video đã được tạo thành công! /path/to/filename.mp4"
    const videoFileNameMatch = result.match(/Video đã được tạo thành công!\s*(.+)/i)
    if (videoFileNameMatch) {
      const videoFilePath = videoFileNameMatch[1].trim()
      const videoFileName = videoFilePath.split('/').pop() || videoFilePath.split('\\').pop() || videoFilePath
      statusMessage.value = `✅ Video đã được tạo thành công! ${videoFileName}`
      
      // Lưu folder path từ outputFolder để có thể mở folder
      if (outputFolder.value) {
        videoFolderPath.value = outputFolder.value
      }
      
      // Add to history
      videoHistory.value.unshift({
        fileName: videoFileName,
        outputPath: videoFilePath,
        imageCount: imageFiles.value.length,
        quality: videoQuality.value,
        duration: imageDuration.value,
        effect: imageEffectType.value,
        createdAt: new Date().toISOString()
      })
      // Keep only last 20 items
      if (videoHistory.value.length > 20) {
        videoHistory.value = videoHistory.value.slice(0, 20)
      }
    } else {
      statusMessage.value = '✅ ' + result
    }
    
    statusType.value = 'success'
  } catch (error) {
    if (error.toString().includes('đã bị hủy') || error.toString().includes('cancelled')) {
      statusMessage.value = '⚠️ Quá trình tạo video đã bị hủy'
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
    statusMessage.value = '⚠️Đã dừng quá trình tạo video và xóa toàn bộ file tạm'
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

// Image handlers
const handleImageFilesUpdate = (files) => {
  imageFiles.value = files
}

const handleImagesSelected = (files) => {
  statusType.value = 'success'
}

const handleImagesCleared = () => {
  // Clear any related state if needed
}

const handleStatusMessage = (message, type) => {
  statusMessage.value = message
  statusType.value = type
}

// Audio handlers
const handleAudioFilesUpdate = (files) => {
  audioFiles.value = files
}

const handleAudioSelected = (files) => {
  // Không cần thông báo khi chọn audio
}

const handleAudioCleared = () => {
  // Không cần thông báo khi xóa audio
}

</script>

