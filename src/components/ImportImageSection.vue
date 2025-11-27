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
              :src="imageUrls[file] || getImageUrl(file)" 
              :alt="getImageFileName(file)"
              class="file-item-image"
              @error="handleImageError"
            />
          </div>
          <button class="file-item-remove" @click="removeImageFile(index)" title="Xóa">✕</button>
        </div>
      </template>
    </draggable>
    
    <!-- Confirm Modal -->
    <ConfirmModal 
      v-if="isShowConfirm"
      :title="'Xác nhận xóa'"
      :message="`Bạn có chắc muốn xóa tất cả ${imageFiles.length} ảnh?`"
      :type="'warning'"
      :confirmText="'Xóa'"
      :cancelText="'Hủy'"
      @confirm="confirmClearAll"
      @cancel="cancelClearAll"
    />
  </div>
</template>

<script setup>
import { ref, watch } from 'vue'
import draggable from 'vuedraggable'
import { open } from '@tauri-apps/plugin-dialog'
import { convertFileSrc } from '@tauri-apps/api/core'
import ConfirmModal from '@/components/ConfirmModal.vue'

const props = defineProps({
  modelValue: {
    type: Array,
    default: () => []
  }
})

const emit = defineEmits(['update:imageFiles', 'images-selected', 'images-cleared', 'status-message'])

const imageFiles = ref([...props.modelValue])
const imageUrls = ref({})
const isShuffling = ref(false)
const isShowConfirm = ref(false)

const getImageFileName = (filePath) => {
  return filePath.split('/').pop() || filePath.split('\\').pop() || filePath
}

const getImageUrl = (filePath) => {
  if (!filePath) return null

  if (imageUrls.value[filePath]) {
    return imageUrls.value[filePath]
  }

  try {
    const url = convertFileSrc(filePath)
    imageUrls.value[filePath] = url
    return url
  } catch (error) {
    console.error('Error converting file path:', error, filePath)
    return null
  }
}

const selectImageFiles = async () => {
  try {
    const selected = await open({
      multiple: true,
      filters: [{
        name: 'Images',
        extensions: ['jpg', 'jpeg', 'png', 'webp']
      }]
    })
    
    if (!selected) return

    const files = Array.isArray(selected) ? selected : [selected]

    for (const file of files) {
      if (!imageFiles.value.includes(file)) {
        imageFiles.value.push(file)
      }
      getImageUrl(file)
    }

    emit('update:imageFiles', imageFiles.value)
    emit('images-selected', imageFiles.value)
  } catch (error) {
    emit('status-message', 'Lỗi khi chọn ảnh: ' + error, 'error')
  }
}

const handleImageError = (event) => {
  console.error('Error loading image:', event?.target?.src)
}

const removeImageFile = (index) => {
  const fileToRemove = imageFiles.value[index]
  imageFiles.value.splice(index, 1)

  if (imageUrls.value[fileToRemove]) {
    delete imageUrls.value[fileToRemove]
  }
  
  emit('update:imageFiles', imageFiles.value)
}

const clearAllImages = () => {
  isShowConfirm.value = true
}

const confirmClearAll = () => {
  imageFiles.value = []
  imageUrls.value = {}
  
  emit('update:imageFiles', [])
  emit('images-cleared')
  isShowConfirm.value = false
}

const cancelClearAll = () => {
  isShowConfirm.value = false
}

const shuffleImages = async () => {
  if (imageFiles.value.length <= 1 || isShuffling.value) return
  
  isShuffling.value = true
  
  // Delay nhỏ để hiển thị loading state
  await new Promise(resolve => setTimeout(resolve, 300))
  
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

watch(imageFiles, (newFiles) => {
  emit('update:imageFiles', newFiles)
})

watch(() => props.modelValue, (newValue) => {
  imageFiles.value = [...newValue]
  newValue.forEach(file => {
    getImageUrl(file)
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
  margin-right: 4px;
  background: white;
  border-radius: 0;
  border: 2px solid transparent;
  cursor: move;
  transition: all 0.2s ease;
  overflow: visible;
  width: 120px;
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
  max-width: 120px;
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