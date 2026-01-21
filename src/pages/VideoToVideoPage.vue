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
        </div>

        <!-- Cột phải: Options và controls -->
        <div class="image-to-video-right-column">
          <!-- Ở trang ghép video không hiển thị cấu hình "Thời gian mỗi ảnh" và "Hiệu ứng ảnh" -->

          <div class="file-group">
            <div class="form-group">
              <label class="form-label">⚙️ Chế độ ghép</label>
              <div class="mode-selection">
                <div class="radio-option">
                  <input type="radio" id="mode-fast" value="fast" v-model="mergeMode">
                  <label for="mode-fast" title="Giữ nguyên chất lượng gốc, cực nhanh. Yêu cầu video đầu vào phải giống hệt nhau về độ phân giải và FPS.">
                    🚀 Siêu tốc (Cùng loại)
                  </label>
                </div>
                <div class="radio-option">
                  <input type="radio" id="mode-convert" value="convert" v-model="mergeMode">
                  <label for="mode-convert" title="Tự động đồng bộ độ phân giải. Hỗ trợ mọi loại video.">
                    🔄 Chuyển đổi (Đa năng)
                  </label>
                </div>
              </div>
            </div>

            <!-- Chỉ hiện chọn chất lượng và hiệu ứng khi ở Mode Convert -->
            <div v-if="mergeMode === 'convert'">
                <div class="form-group">
                  <label class="form-label">📺 Chất lượng video ra</label>
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
  </div>
</template>

<script setup>
import { ref } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { useTauri } from '../composables/useTauri'
import ImportVideoSection from '@/components/ImportVideoSection.vue'
import VideoEffect from '@/components/VideoEffect.vue'
import VideoAspectRatioSection from '@/components/VideoAspectRatioSection.vue'
import '../assets/css/image-to-video.css'

const { callCommand } = useTauri()

// Video files - store file paths
const videoFiles = ref([])

// Remove original audio from videos
const removeOriginalAudio = ref(false)

// Configuration (vẫn giữ giống trang ImageToVideoPage, nhưng không hiển thị UI thời gian / hiệu ứng ảnh)
const imageDuration = ref(6)
const videoQuality = ref('fullhd')
const videoAspectRatio = ref('16:9') // Default: 16:9 (Video dài)
const videoEffectType = ref('diagtl')
const imageEffectType = ref('zoom-in')
const mergeMode = ref('convert') // Default: Convert (An toàn)

// Output
const outputFolder = ref('')
const isCreating = ref(false)
const statusMessage = ref('')
const statusType = ref('info')
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
      audioFiles: [],
      outputPath: outputPath,
      crf: crf,
      isHasAutoCaption: false,
      videoEffectType: videoEffectType.value,
      videoQuality: videoQuality.value,
      videoAspectRatio: videoAspectRatio.value,
      removeOriginalAudio: removeOriginalAudio.value,
      mergeMode: mergeMode.value
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
</script>


