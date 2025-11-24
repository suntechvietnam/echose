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

          <div class="file-group image-selection-group">
            <h3 class="form-label">Danh sách ảnh</h3>
            <div class="file-selector">
              <button class="btn-select-file" @click="selectImageFiles">
                📁 Chọn Ảnh
              </button>
              <button 
                v-if="imageFiles.length > 0"
                class="btn-clear-all" 
                @click="clearAllImages"
              >
                🗑️ Xóa Tất Cả
              </button>
              <button 
                v-if="imageFiles.length > 1"
                class="btn-shuffle-images" 
                @click="shuffleImages"
              >
                🔀 Sắp Xếp Ngẫu Nhiên
              </button>
              <span class="file-count">{{ imageFiles.length }} ảnh đã chọn</span>
            </div>
          
            <!-- Danh sách ảnh -->
            <draggable
              v-if="imageFiles.length > 0"
              v-model="imageFiles"
              class="file-list file-list-flex"
              ghost-class="ghost-item"
              chosen-class="chosen-item"
              drag-class="drag-item"
              :animation="200"
            >
              <template #item="{ element: file, index }">
                <div class="file-item" :key="file">
                  <div class="file-item-info">
                    <img 
                      v-if="imageUrls[file]"
                      :src="imageUrls[file]" 
                      :alt="getImageFileName(file)"
                      class="file-item-image"
                    />
                    <span v-else class="file-item-name">Đang tải ảnh...</span>
                  </div>
                  <button class="file-item-remove" @click="removeImageFile(index)" title="Xóa">✕</button>
                </div>
              </template>
            </draggable>
          </div>

          <!-- Audio Selection Section -->
          <ImportAudioSection 
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

            <div class="form-group video-effect-group">
              <label class="form-label">✨ Hiệu ứng video</label>
              <div class="effect-preview-grid">
                <div 
                  v-for="effect in videoEffectOptions" 
                  :key="effect.value"
                  :class="['effect-preview-item', { active: videoEffectType === effect.value }]"
                  @click="videoEffectType = effect.value"
                  :title="effect.name"
                >
                  <img 
                    v-if="effect.preview"
                    :src="effect.preview" 
                    :alt="effect.name"
                    class="effect-preview-image"
                    @error="handleImageError"
                    loading="lazy"
                  />
                  <div v-else class="effect-preview-fallback">
                    {{ effect.name.substring(0, 2) }}
                  </div>
                  <div v-if="videoEffectType === effect.value" class="effect-check-icon">
                    ✓
                  </div>
                </div>
              </div>
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
import { ref, onUnmounted, watch } from 'vue'
import draggable from 'vuedraggable'
import { open } from '@tauri-apps/api/dialog'
import { readBinaryFile } from '@tauri-apps/api/fs'
import { useTauri } from '../composables/useTauri'
import ImportAudioSection from '@/components/ImportAudioSection.vue'
import '../assets/css/image-to-video.css'

// Import effect images
import noneImg from '../assets/img/effects/none.png'
import circleopenImg from '../assets/img/effects/circleopen.gif'
import diagbrImg from '../assets/img/effects/diagbr.gif'
import diagtlImg from '../assets/img/effects/diagtl.gif'
import diagtrImg from '../assets/img/effects/diagtr.gif'
import dissolveImg from '../assets/img/effects/dissolve.gif'
import hlsliceImg from '../assets/img/effects/hlslice.gif'
import hlwindImg from '../assets/img/effects/hlwind.gif'
import hrsliceImg from '../assets/img/effects/hrslice.gif'
import hrwindImg from '../assets/img/effects/hrwind.gif'
import radialImg from '../assets/img/effects/radial.gif'
import vdsliceImg from '../assets/img/effects/vdslice.gif'
import vdwindImg from '../assets/img/effects/vdwind.gif'
import vusliceImg from '../assets/img/effects/vuslice.gif'
import vuwindImg from '../assets/img/effects/vuwind.gif'

const { callCommand } = useTauri()

// Image files - store file paths
const imageFiles = ref([])
const imageUrls = ref({})

// Audio files - store file paths
const audioFiles = ref([])

// Configuration
const imageDuration = ref(6)
const durationOptions = [5, 6, 7, 8, 10, 12, 15, 20, 30]
const videoQuality = ref('fullhd')
const videoAspectRatio = ref('16:9') // Default: 16:9 (Video dài)
const videoEffectType = ref('none')
const imageEffectType = ref('none')

const imageEffectOptions = [
  { name: 'Không có', value: 'none' },
  { name: 'Zoom in', value: 'zoom-in' },
  { name: 'Zoom out', value: 'zoom-out' },
  { name: 'Fade in', value: 'fade-in' },
  { name: 'Fade out', value: 'fade-out' },
  { name: 'Pan', value: 'pan' },
]

const videoEffectOptions = [
  { name: 'None', value: 'none', preview: noneImg },
  { name: 'Circle Open', value: 'circleopen', preview: circleopenImg },
  { name: 'Diagonal Bottom Right', value: 'diagbr', preview: diagbrImg },
  { name: 'Diagonal Top Left', value: 'diagtl', preview: diagtlImg },
  { name: 'Diagonal Top Right', value: 'diagtr', preview: diagtrImg },
  { name: 'Dissolve', value: 'dissolve', preview: dissolveImg },
  { name: 'Horizontal Left Slice', value: 'hlslice', preview: hlsliceImg },
  { name: 'Horizontal Left Wind', value: 'hlwind', preview: hlwindImg },
  { name: 'Horizontal Right Slice', value: 'hrslice', preview: hrsliceImg },
  { name: 'Horizontal Right Wind', value: 'hrwind', preview: hrwindImg },
  { name: 'Radial', value: 'radial', preview: radialImg },
  { name: 'Vertical Down Slice', value: 'vdslice', preview: vdsliceImg },
  { name: 'Vertical Down Wind', value: 'vdwind', preview: vdwindImg },
  { name: 'Vertical Up Slice', value: 'vuslice', preview: vusliceImg },
  { name: 'Vertical Up Wind', value: 'vuwind', preview: vuwindImg },
]

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

const changeVideoAspectRatio = (aspectRatio) => {
  videoAspectRatio.value = aspectRatio
}

watch(videoAspectRatio, (oldVal, newVal) => {
  if (oldVal !== newVal) {
    clearAllImages()
  }
})

const getImageFileName = (filePath) => {
  return filePath.split('/').pop() || filePath.split('\\').pop() || filePath
}

const getMimeType = (filePath) => {
  const ext = filePath.toLowerCase().split('.').pop()
  const mimeTypes = {
    'jpg': 'image/jpeg',
    'jpeg': 'image/jpeg',
    'png': 'image/png',
    'webp': 'image/webp'
  }
  return mimeTypes[ext] || 'image/jpeg'
}

const loadImageAsBlobUrl = async (filePath) => {
  try {
    // Check if already loaded
    if (imageUrls.value[filePath]) {
      return imageUrls.value[filePath]
    }

    // Read file as binary
    const fileData = await readBinaryFile(filePath)
    
    // Get MIME type
    const mimeType = getMimeType(filePath)
    
    // Create Blob from binary data
    const blob = new Blob([fileData], { type: mimeType })
    
    // Create Blob URL
    const blobUrl = URL.createObjectURL(blob)
    
    // Cache the blob URL
    imageUrls.value[filePath] = blobUrl
    
    return blobUrl
  } catch (error) {
    console.error('Error loading image:', error, filePath)
    return null
  }
}

const selectImageFiles = async () => {
  try {
    // Sử dụng Tauri Dialog API - cách tốt nhất
    // Native dialog, hỗ trợ Command+A/Ctrl+A, có path trực tiếp
    const selected = await open({
      multiple: true,
      filters: [{
        name: 'Images',
        extensions: ['jpg', 'jpeg', 'png', 'gif', 'webp']
      }]
    })
    
    if (selected) {
      // selected có thể là string (single) hoặc string[] (multiple)
      const files = Array.isArray(selected) ? selected : [selected]
      
      // Thêm các file mới vào danh sách (tránh trùng lặp)
      for (const file of files) {
        if (!imageFiles.value.includes(file)) {
          imageFiles.value.push(file)
          // Load image immediately
          loadImageAsBlobUrl(file)
        }
      }
      
      statusType.value = 'success'
    }
  } catch (error) {
    statusMessage.value = 'Lỗi khi chọn ảnh: ' + error
    statusType.value = 'error'
  }
}

const removeImageFile = (index) => {
  const fileToRemove = imageFiles.value[index]
  // Cleanup blob URL để tránh memory leak
  if (imageUrls.value[fileToRemove]) {
    URL.revokeObjectURL(imageUrls.value[fileToRemove])
    delete imageUrls.value[fileToRemove]
  }
  // Remove from array
  imageFiles.value.splice(index, 1)
}

const clearAllImages = () => {
  // Cleanup tất cả blob URLs để tránh memory leak
  Object.values(imageUrls.value).forEach(url => {
    if (url) URL.revokeObjectURL(url)
  })
  // Clear arrays
  imageFiles.value = []
  imageUrls.value = {}
  statusMessage.value = '✅ Đã xóa tất cả ảnh'
  statusType.value = 'success'
}

const shuffleImages = () => {
  if (imageFiles.value.length <= 1) return
  
  // Fisher-Yates shuffle algorithm với kiểm tra không trùng lặp liên tiếp
  const shuffleArray = (array) => {
    const shuffled = [...array]
    let attempts = 0
    const maxAttempts = 100
    
    do {
      // Fisher-Yates shuffle
      for (let i = shuffled.length - 1; i > 0; i--) {
        const j = Math.floor(Math.random() * (i + 1));
        [shuffled[i], shuffled[j]] = [shuffled[j], shuffled[i]]
      }
      attempts++
    } while (hasConsecutiveDuplicates(shuffled) && attempts < maxAttempts)
    
    return shuffled
  }
  
  const hasConsecutiveDuplicates = (arr) => {
    for (let i = 0; i < arr.length - 1; i++) {
      if (arr[i] === arr[i + 1]) {
        return true
      }
    }
    return false
  }
  
  const shuffled = shuffleArray(imageFiles.value)
  imageFiles.value = shuffled
  
  statusMessage.value = '✅ Đã sắp xếp ngẫu nhiên các ảnh'
  statusType.value = 'success'
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
  
  isCreating.value = true
  progress.value = 5
  statusMessage.value = '⏳ Đang bắt đầu tạo video từ ảnh...'
  statusType.value = 'info'
  videoFolderPath.value = null
  
  try {
    progress.value = 20
    statusMessage.value = '⏳ Đang tạo video từ ảnh...'
    
    const result = await callCommand('create_video_from_images', {
      imageFiles: imageFiles.value,
      imageDuration: imageDuration.value,
      imageEffectType: imageEffectType.value,
      videoQuality: videoQuality.value,
      videoAspectRatio: videoAspectRatio.value,
      videoEffectType: videoEffectType.value,
      audioFiles: audioFiles.value,
      outputFolder: outputFolder.value.trim()
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

// Audio handlers
const handleAudioFilesUpdate = (files) => {
  audioFiles.value = files
}

const handleAudioSelected = (files) => {
  statusMessage.value = `✅ Đã chọn ${files.length} file nhạc`
  statusType.value = 'success'
}

const handleAudioCleared = () => {
  statusMessage.value = '✅ Đã xóa tất cả file nhạc'
  statusType.value = 'success'
}

const handleImageError = (event) => {
  // Handle error when effect preview image fails to load
  console.warn('Failed to load effect preview image:', event.target.src)
  // Optionally hide the image or show a placeholder
  event.target.style.display = 'none'
}

// Cleanup blob URLs khi component unmount để tránh memory leak
onUnmounted(() => {
  Object.values(imageUrls.value).forEach(url => {
    if (url) URL.revokeObjectURL(url)
  })
})
</script>

