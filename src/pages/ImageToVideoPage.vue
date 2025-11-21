<template>
  <div class="page-content">
    <div class="page-header">
      <h2>Tạo Video Nguồn Từ Ảnh</h2>
    </div>
    <div class="page-body">
      <!-- Chọn danh sách ảnh -->
        <div class="file-group">
          <label class="form-label">🖼️ Chọn Danh Sách Ảnh</label>
          <div class="file-selector">
            <button class="btn-select-file" @click="selectImageFiles">
              📁 Chọn Ảnh
            </button>
            <span class="file-count">{{ imageFiles.length }} ảnh đã chọn</span>
          </div>
        
        <!-- Danh sách ảnh -->
        <draggable
          v-if="imageFiles.length > 0"
          v-model="imageFiles"
          class="file-list file-list-flex"
          :animation="200"
          ghost-class="ghost-item"
          chosen-class="chosen-item"
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

      <!-- Cấu hình video -->
      <div class="file-group">
        <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: 20px;">
          <!-- Khoảng cách giữa các ảnh -->
          <div>
            <label class="form-label">⏱️ Khoảng Cách Giữa Các Ảnh (giây)</label>
            <select v-model="imageDuration" class="form-select">
              <option v-for="duration in durationOptions" :key="duration" :value="duration">
                {{ duration }} giây
              </option>
            </select>
          </div>

          <!-- Chất lượng video -->
          <div>
            <label class="form-label">📺 Chất Lượng Video</label>
            <select v-model="videoQuality" class="form-select">
              <option value="2K">2K (2048x1080)</option>
              <option value="4K">4K (3840x2160)</option>
            </select>
          </div>

          <!-- Hiệu ứng -->
          <div>
            <label class="form-label">✨ Hiệu Ứng</label>
            <select v-model="effectType" class="form-select">
              <option value="zoom-random">Zoom nhẹ random in/out</option>
              <option value="slide-fade">Slide left + Fade</option>
              <option value="zoom-ken-burns">Ken Burns (Zoom + Pan)</option>
              <option value="fade-only">Fade đơn giản</option>
              <option value="none">Không có hiệu ứng</option>
            </select>
          </div>
        </div>
      </div>

      <!-- Chọn thư mục lưu video -->
      <div class="file-group">
        <label class="form-label">💾 Thư Mục Lưu Video</label>
        <div class="input-group">
          <input 
            type="text" 
            v-model="outputFolder"
            class="form-input" 
            placeholder="Chọn thư mục để lưu video..."
            readonly
          />
          <button class="btn-select-folder" @click="selectOutputFolder">
            Chọn Thư Mục
          </button>
        </div>
      </div>

      <!-- Button tạo video -->
      <div class="form-actions">
        <button 
          class="btn-create-video" 
          @click="createVideoFromImages" 
          :disabled="isCreating"
        >
          {{ isCreating ? '⏳ Đang tạo video...' : '🎬 Tạo Video' }}
        </button>
        <button 
          v-if="isCreating"
          class="btn-stop-video" 
          @click="stopVideoCreation"
        >
          ⏹️ Dừng
        </button>
      </div>

      <!-- Status và Progress -->
      <div :class="['download-status', statusType]" v-if="statusMessage">
        {{ statusMessage }}
      </div>
      <div v-if="isCreating" class="download-progress">
        <div class="progress-bar">
          <div class="progress-fill" :style="{ width: progress + '%' }"></div>
        </div>
        <div class="progress-text">{{ progress }}%</div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onUnmounted } from 'vue'
import draggable from 'vuedraggable'
import { open } from '@tauri-apps/api/dialog'
import { readBinaryFile } from '@tauri-apps/api/fs'
import { useTauri } from '../composables/useTauri'

const { callCommand } = useTauri()

// Image files - store file paths
const imageFiles = ref([])
const imageUrls = ref({})

// Configuration
const imageDuration = ref(5)
const durationOptions = [5, 6, 7, 8, 10, 12, 15, 20, 30]
const videoQuality = ref('2K')
const effectType = ref('zoom-random')

// Output
const outputFolder = ref('')
const isCreating = ref(false)
const statusMessage = ref('')
const statusType = ref('info')
const progress = ref(0)
const currentProcessId = ref(null)

const getImageFileName = (filePath) => {
  return filePath.split('/').pop() || filePath.split('\\').pop() || filePath
}

const getMimeType = (filePath) => {
  const ext = filePath.toLowerCase().split('.').pop()
  const mimeTypes = {
    'jpg': 'image/jpeg',
    'jpeg': 'image/jpeg',
    'png': 'image/png',
    'gif': 'image/gif',
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
      
      statusMessage.value = `✅ Đã chọn ${imageFiles.value.length} ảnh`
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
  
  try {
    const processId = await callCommand('create_video_from_images', {
      imageFiles: imageFiles.value,
      imageDuration: imageDuration.value,
      videoQuality: videoQuality.value,
      effectType: effectType.value,
      outputFolder: outputFolder.value.trim()
    })
    
    currentProcessId.value = processId
    progress.value = 20
    
    const result = await callCommand('wait_for_video_creation', {
      processId: processId
    })
    
    if (result.includes('Lỗi')) {
      throw new Error(result)
    }
    
    currentProcessId.value = null
    progress.value = 100
    statusMessage.value = '✅ ' + result
    statusType.value = 'success'
  } catch (error) {
    currentProcessId.value = null
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
    await callCommand('stop_video_creation', {
      processId: currentProcessId.value
    })
    currentProcessId.value = null
    statusMessage.value = '⚠️ Đã gửi lệnh dừng quá trình tạo video...'
    statusType.value = 'info'
    isCreating.value = false
  } catch (error) {
    statusMessage.value = '❌ Lỗi khi dừng: ' + error
    statusType.value = 'error'
  }
}

// Cleanup blob URLs khi component unmount để tránh memory leak
onUnmounted(() => {
  Object.values(imageUrls.value).forEach(url => {
    if (url) URL.revokeObjectURL(url)
  })
})
</script>

