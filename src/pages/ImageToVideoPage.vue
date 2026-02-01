<template>
  <div class="page-content">
    <div class="page-header">
      <h2>Tạo video từ ảnh</h2>
    </div>
    <div class="page-body image-to-video-container">
      <!-- Layout 2 cột -->
      <div class="image-to-video-layout">
        <!-- Cột trái: Media & Phụ đề -->
        <div class="image-to-video-left-column">
          <VideoAspectRatioSection v-model="videoAspectRatio" />

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

          <!-- Logo Section Card -->
          <div class="file-group logo-settings-card">
            <h3 class="card-subtitle-main">🏷️ Watermark / Logo</h3>
            <div class="input-group">
              <input 
                type="text" 
                v-model="logoPath"
                class="form-input" 
                placeholder="Chọn ảnh logo (PNG/JPG)..."
                readonly
              />
              <button class="btn-select-folder" @click="selectLogoFile">Chọn</button>
              <button v-if="logoPath" class="btn-clear-path" @click="logoPath = ''">✕</button>
            </div>
            
            <div class="logo-custom-settings" :class="{ 'is-disabled': !logoPath }">
              <div class="settings-divider"><span>Vị trí Logo (Manual)</span></div>
              <div class="logo-pos-group">
                <label class="form-label-small">Góc Neo:</label>
                <select v-model="logoPosition" class="form-select-small" :disabled="!logoPath">
                  <option value="top_left">Trên - Trái</option>
                  <option value="top_right">Trên - Phải</option>
                  <option value="bottom_left">Dưới - Trái</option>
                  <option value="bottom_right">Dưới - Phải</option>
                </select>
              </div>
              <div class="logo-margins-grid">
                <div class="margin-input-item">
                  <label>Top</label>
                  <input type="number" v-model.number="logoMarginTop" :disabled="!logoPath || logoPosition.includes('bottom')" />
                </div>
                <div class="margin-input-item">
                  <label>Bottom</label>
                  <input type="number" v-model.number="logoMarginBottom" :disabled="!logoPath || logoPosition.includes('top')" />
                </div>
                <div class="margin-input-item">
                  <label>Left</label>
                  <input type="number" v-model.number="logoMarginLeft" :disabled="!logoPath || logoPosition.includes('right')" />
                </div>
                <div class="margin-input-item">
                  <label>Right</label>
                  <input type="number" v-model.number="logoMarginRight" :disabled="!logoPath || logoPosition.includes('left')" />
                </div>
              </div>
            </div>
          </div>

          <!-- Subtitle Section Card -->
          <div class="file-group subtitle-settings-card">
            <h3 class="card-subtitle-main">📄 Phụ đề (Hardsub)</h3>
            
            <!-- Subtitle Mode Selection -->
            <div class="subtitle-mode-selector">
              <label class="mode-option" :class="{ active: subtitleMode === 'none' }">
                <input type="radio" v-model="subtitleMode" value="none" />
                <span>None</span>
              </label>
              <label class="mode-option" :class="{ active: subtitleMode === 'auto' }">
                <input type="radio" v-model="subtitleMode" value="auto" @change="handleSubtitleModeChange" />
                <span>Tự động (AI)</span>
              </label>
              <label class="mode-option" :class="{ active: subtitleMode === 'file' }">
                <input type="radio" v-model="subtitleMode" value="file" @change="handleSubtitleModeChange" />
                <span>Chọn file</span>
              </label>
            </div>

            <!-- File selection (only if mode is 'file') -->
            <div v-if="subtitleMode === 'file'" class="input-group" style="margin-top: 15px;">
              <input 
                type="text" 
                v-model="subtitlePath"
                class="form-input" 
                placeholder="Chọn file .ass hoặc .srt..."
                readonly
              />
              <button class="btn-select-folder" @click="selectSubtitleFile">Chọn</button>
              <button v-if="subtitlePath" class="btn-clear-path" @click="subtitlePath = ''">✕</button>
            </div>

            <!-- Custom settings (only if not 'none') -->
            <div v-if="subtitleMode !== 'none'" class="subtitle-custom-settings">
              <div class="settings-divider"><span>Cấu hình Phụ đề</span></div>
              <div class="subtitle-grid-advanced">
                <div class="sub-setting-item">
                  <label>Ngôn ngữ</label>
                  <select v-model="subtitleLanguage" class="form-select-small">
                    <option value="vi">Tiếng Việt</option>
                    <option value="en">Tiếng Anh</option>
                    <option value="ja">Tiếng Nhật</option>
                  </select>
                </div>
                <div class="sub-setting-item">
                  <label>Phông chữ</label>
                  <select v-model="subtitleFontName" class="form-select-small">
                    <option value="Arial">Arial (Global)</option>
                    <option value="Roboto">Roboto</option>
                    <option value="Noto Sans JP" v-if="subtitleLanguage === 'ja'">Noto Sans JP</option>
                    <option value="Be Vietnam Pro" v-if="subtitleLanguage === 'vi'">Be Vietnam Pro</option>
                  </select>
                </div>
                <div class="sub-setting-item">
                  <label>C cỡ chữ</label>
                  <input type="number" v-model.number="subtitleFontSize" class="form-input-small" />
                </div>
                <div class="sub-setting-item">
                  <label>Cách đáy</label>
                  <input type="number" v-model.number="subtitleMarginV" class="form-input-small" />
                </div>
              </div>
            </div>
            
            <div v-if="subtitleMode === 'auto' && audioFiles.length === 0" class="sub-warning-msg">
              <p>⚠️ Cần có file audio để dùng tính năng AI</p>
            </div>
          </div>
        </div>

        <!-- Cột phải: Settings & Actions -->
        <div class="image-to-video-right-column">
          <!-- Cấu hình ảnh Card -->
          <div class="file-group settings-card-premium">
            <h3 class="card-subtitle-main">⏱️ Cấu hình ảnh</h3>
            <div class="form-group-grid">
              <div class="form-item">
                <label class="form-label-small">Thời gian / Ảnh</label>
                <select v-model="imageDuration" class="form-select">
                  <option v-for="d in durationOptions" :key="d" :value="d">{{ d }}s</option>
                </select>
              </div>
              <div class="form-item">
                <label class="form-label-small">Hiệu ứng ảnh</label>
                <select v-model="imageEffectType" class="form-select">
                  <option v-for="opt in imageEffectOptions" :key="opt.value" :value="opt.value">{{ opt.name }}</option>
                </select>
              </div>
            </div>
          </div>

          <!-- Chất lượng Card -->
          <div class="file-group quality-card-premium">
            <h3 class="card-subtitle-main">📺 Chất lượng video</h3>
            <select v-model="videoQuality" class="form-select">
              <option value="hd">720 (HD)</option>
              <option value="fullhd">1080 (Full HD)</option>
              <option value="2K">2K (2048x1080)</option>
              <option value="4K">4K (3840x2160)</option>
            </select>
          </div>

          <!-- Hiệu ứng Video Card -->
          <div class="file-group effect-card-premium">
            <h3 class="card-subtitle-main">✨ Hiệu ứng video</h3>
            <VideoEffect :selectedEffect="videoEffectType" @update:selectedEffect="videoEffectType = $event" />
          </div>

          <!-- Thư mục lưu Card -->
          <div class="file-group folder-card-premium">
            <h3 class="card-subtitle-main">💾 Thư mục lưu</h3>
            <div class="input-group">
              <input type="text" v-model="outputFolder" class="form-input" placeholder="Chọn thư mục..." readonly />
              <button class="btn-select-folder" @click="selectOutputFolder">Chọn</button>
            </div>
          </div>

          <!-- Action Buttons -->
          <div class="form-actions-full">
            <button class="btn-create-video-v2" @click="createVideoFromImages" :disabled="isCreating">
              {{ isCreating ? 'Đang tạo video...' : 'Tạo Video' }}
            </button>
            <button v-if="isCreating" class="btn-stop-video-v2" @click="stopVideoCreation">Dừng</button>
          </div>

          <!-- Progress -->
          <div v-if="isCreating" class="progress-container-v2">
            <div class="progress-bar-v2"><div class="pf-v2" :style="{ width: progress + '%' }"></div></div>
            <div class="pt-v2">{{ progress }}%</div>
          </div>

          <!-- Status -->
          <div :class="['status-box-v2', statusType]" v-if="statusMessage">
            <span>{{ statusMessage }}</span>
            <button v-if="statusType === 'success' && videoFolderPath" class="btn-open-v2" @click="openVideoFolderFromPath">📁</button>
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
import ImportImageSection from '@/components/ImportImageSection.vue'
import VideoEffect from '@/components/VideoEffect.vue'
import ConfirmModal from '@/components/ConfirmModal.vue'
import VideoAspectRatioSection from '@/components/VideoAspectRatioSection.vue'
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
const logoPath = ref('')
const logoPosition = ref('top_left')
const logoMarginTop = ref(20)
const logoMarginRight = ref(20)
const logoMarginBottom = ref(20)
const logoMarginLeft = ref(20)
const subtitleMode = ref('none') // 'none', 'auto', 'file'
const subtitlePath = ref('')
const subtitleMarginV = ref(30)
const subtitleFontSize = ref(58)
const subtitleLanguage = ref('ja')
const subtitleFontName = ref('Arial')

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
      outputFolder.value = Array.isArray(selected) ? selected[0] : selected
    }
  } catch (error) {
    statusMessage.value = 'Lỗi khi chọn thư mục: ' + error
    statusType.value = 'error'
  }
}

const selectLogoFile = async () => {
  try {
    const selected = await open({
      multiple: false,
      filters: [{ name: 'Image', extensions: ['png', 'jpg', 'jpeg'] }]
    })
    if (selected) {
      logoPath.value = Array.isArray(selected) ? selected[0] : selected
    }
  } catch (error) {
    console.error('Lỗi khi chọn logo:', error)
  }
}

const selectSubtitleFile = async () => {
  try {
    const selected = await open({
      multiple: false,
      filters: [{ name: 'Subtitles', extensions: ['ass', 'srt'] }]
    })
    if (selected) {
      subtitlePath.value = Array.isArray(selected) ? selected[0] : selected
      subtitleMode.value = 'file'
    }
  } catch (error) {
    console.error('Lỗi khi chọn subtitle:', error)
  }
}

const handleSubtitleModeChange = () => {
  if (subtitleMode.value === 'auto') {
    subtitlePath.value = ''
    if (audioFiles.value.length === 0) {
      isShowAudioRequiredModal.value = true
    }
  } else if (subtitleMode.value === 'none') {
    subtitlePath.value = ''
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
  
  // Tạo process_id để có thể stop
  const timestampForId = Date.now();
  const processId = `images_video_${timestampForId}`;
  currentProcessId.value = processId;
  
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
      isAutoCaption: subtitleMode.value === 'auto',
      logoPath: logoPath.value || null,
      logoPosition: logoPosition.value,
      logoMarginTop: logoMarginTop.value,
      logoMarginRight: logoMarginRight.value,
      logoMarginBottom: logoMarginBottom.value,
      logoMarginLeft: logoMarginLeft.value,
      subtitlePath: subtitleMode.value === 'file' ? (subtitlePath.value || null) : null,
      subtitleMarginV: subtitleMarginV.value,
      subtitleFontSize: subtitleFontSize.value,
      subtitleLanguage: subtitleLanguage.value,
      subtitleFontName: subtitleFontName.value,
      processId: processId
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
  
  // Tự động uncheck auto caption khi không còn audio files
  if (files.length === 0 && isAutoCaption.value) {
    isAutoCaption.value = false
  }
}

const handleAudioSelected = (files) => {
  // Không cần thông báo khi chọn audio
}

const handleAudioCleared = () => {
  // Tự động uncheck auto caption khi xóa tất cả audio
  if (isAutoCaption.value) {
    isAutoCaption.value = false
  }
}

</script>

