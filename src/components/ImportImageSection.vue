<template>
  <div class="file-group image-selection-group">
    <h3 class="form-label">Danh sách ảnh</h3>
    <div class="file-selector">
      <button class="btn-select-file" @click="selectImageFiles">
        📁 Chọn ảnh
      </button>
      <button 
        v-if="imageFiles.length > 0"
        class="btn-clear-all" 
        @click="clearAllImages"
      >
        🗑️ Xóa tất cả
      </button>
      <button 
        v-if="imageFiles.length > 1"
        class="btn-shuffle-images" 
        @click="shuffleImages"
        :disabled="isShuffling"
      >
        {{ isShuffling ? 'Đang sắp xếp...' : '🔀 Sắp xếp ngẫu nhiên' }}
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
</template>

<script setup>
import { ref, onUnmounted, watch } from 'vue'
import draggable from 'vuedraggable'
import { open } from '@tauri-apps/api/dialog'
import { readBinaryFile } from '@tauri-apps/api/fs'

const props = defineProps({
  modelValue: {
    type: Array,
    default: () => []
  }
})

const emit = defineEmits(['update:imageFiles', 'images-selected', 'images-cleared', 'status-message'])

// Image files - store file paths, sync with parent
const imageFiles = ref([...props.modelValue])
const imageUrls = ref({})
const isShuffling = ref(false)

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
      
      emit('update:imageFiles', imageFiles.value)
      emit('images-selected', imageFiles.value)
    }
  } catch (error) {
    emit('status-message', 'Lỗi khi chọn ảnh: ' + error, 'error')
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
  
  emit('update:imageFiles', imageFiles.value)
}

const clearAllImages = () => {
  // Cleanup tất cả blob URLs để tránh memory leak
  Object.values(imageUrls.value).forEach(url => {
    if (url) URL.revokeObjectURL(url)
  })
  // Clear arrays
  imageFiles.value = []
  imageUrls.value = {}
  
  emit('update:imageFiles', [])
  emit('images-cleared')
  emit('status-message', '✅ Đã xóa tất cả ảnh', 'success')
}

const shuffleImages = async () => {
  if (imageFiles.value.length <= 1 || isShuffling.value) return
  
  isShuffling.value = true
  
  // Delay nhỏ để hiển thị loading state
  await new Promise(resolve => setTimeout(resolve, 100))
  
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
  
  emit('update:imageFiles', imageFiles.value)
  
  isShuffling.value = false
}

// Watch để emit khi imageFiles thay đổi
watch(imageFiles, (newFiles) => {
  emit('update:imageFiles', newFiles)
}, { deep: true })

// Watch để sync khi parent modelValue thay đổi
watch(() => props.modelValue, (newValue) => {
  imageFiles.value = [...newValue]
  // Reload images if needed
  newValue.forEach(file => {
    if (!imageUrls.value[file]) {
      loadImageAsBlobUrl(file)
    }
  })
}, { deep: true })

// Cleanup blob URLs khi component unmount để tránh memory leak
onUnmounted(() => {
  Object.values(imageUrls.value).forEach(url => {
    if (url) URL.revokeObjectURL(url)
  })
})
</script>

<style scoped>
.image-selection-group {
  display: flex;
  flex-direction: column;
  min-height: 200px;
}

.file-selector {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 12px;
  flex-wrap: wrap;
}

.btn-select-file {
  padding: 8px 16px;
  background: #667eea;
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.2s ease;
}

.btn-select-file:hover {
  background: #5568d3;
}

.btn-clear-all {
  padding: 8px 16px;
  background: #ef4444;
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.2s ease;
}

.btn-clear-all:hover {
  background: #dc2626;
}

.btn-shuffle-images {
  padding: 8px 16px;
  background: #10b981;
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.2s ease;
}

.btn-shuffle-images:hover {
  background: #059669;
}

.file-count {
  font-size: 14px;
  color: #64748b;
  font-weight: 500;
}

/* Flex layout for image list - natural wrapping */
.file-list-flex {
  display: flex !important;
  flex-direction: row !important;
  flex-wrap: wrap !important;
  gap: 0 !important;
  flex: 1;
  align-content: flex-start;
}

/* File item in flex layout */
.file-list-flex .file-item {
  position: relative;
  display: flex;
  margin: 3px;
  background: white;
  border-radius: 0;
  border: 2px solid transparent;
  cursor: move;
  transition: all 0.2s ease;
  overflow: visible;
  width: 160px;
  flex-shrink: 0;
  flex-grow: 0;
  align-items: center;
  justify-content: center;
}

/* File item info container */
.file-item-info {
  display: block;
  width: auto;
}

/* File item name text */
.file-item-name {
  font-size: 14px;
  color: #1e293b;
  word-break: break-all;
}

/* Image display */
.file-item-image {
  max-width: 150px;
  width: auto;
  height: auto;
  object-fit: contain;
  display: block;
  padding: 0;
  border: none;
}

/* Remove button */
.file-item-remove {
  position: absolute;
  top: 4px;
  right: 4px;
  padding: 4px 8px;
  background: rgba(239, 68, 68, 0.9);
  color: white;
  border: none;
  border-radius: 4px;
  font-size: 12px;
  font-weight: bold;
  cursor: pointer;
  z-index: 10;
  opacity: 0;
  transition: opacity 0.2s ease;
  width: auto;
  height: auto;
  display: flex;
  align-items: center;
  justify-content: center;
}

.file-item:hover .file-item-remove {
  opacity: 1;
}

.file-item-remove:hover {
  background: rgba(220, 38, 38, 1);
}

/* Drag and drop styles */
.ghost-item {
  opacity: 0.4;
  background: #e2e8f0;
  border: 2px dashed #667eea;
}

.chosen-item {
  opacity: 0.8;
  transform: scale(0.98);
  cursor: grabbing;
}

.drag-item {
  border: 2px solid #667eea !important;
  box-shadow: 0 0 8px rgba(102, 126, 234, 0.3);
}
</style>