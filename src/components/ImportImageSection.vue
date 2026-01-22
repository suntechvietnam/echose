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
      class="file-list"
      ghost-class="ghost-item"
      chosen-class="chosen-item"
      drag-class="drag-item"
      :animation="200"
      :item-key="imageItemKey"
    >
      <template #item="{ element: file, index }">
        <div class="file-item">
          <img 
            v-if="imageUrls[file]"
            :src="imageUrls[file]"
            class="file-item-img"
            @error="handleImageError(file)"
          />
          <div v-else class="file-item-loading">
            <span class="spinner-small"></span>
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

// Dùng cho vuedraggable: với array string, key chính là giá trị string
const imageItemKey = (filePath) => filePath

// Tải URL cho danh sách file
const loadImageUrls = (files) => {
  files.forEach(file => {
    if (!imageUrls.value[file]) {
      try {
        const url = convertFileSrc(file)
        imageUrls.value[file] = url
      } catch (error) {
        console.error('Lỗi chuyển đổi đường dẫn ảnh:', error, file)
      }
    }
  })
}

// Gọi hàm tải URL ngay từ đầu nếu có dữ liệu cũ
if (imageFiles.value.length > 0) {
  loadImageUrls(imageFiles.value)
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
    }
    loadImageUrls(imageFiles.value)

    emit('update:imageFiles', imageFiles.value)
    emit('images-selected', imageFiles.value)
  } catch (error) {
    emit('status-message', 'Lỗi khi chọn ảnh: ' + error, 'error')
  }
}

const handleImageError = (file) => {
  console.error('Không thể load ảnh:', file)
  // Thử load lại một lần nữa nếu lỗi
  try {
    imageUrls.value[file] = convertFileSrc(file)
  } catch (e) {}
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

watch(() => props.modelValue, (newValue) => {
  imageFiles.value = [...newValue]
  loadImageUrls(newValue)
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

/* Layout danh sách (giới hạn 3 hàng, scroll nếu nhiều hơn) */
.file-list {
  display: grid;
  grid-template-columns: repeat(6, minmax(0, 1fr));
  gap: 8px;
  flex: 1;
  max-height: 280px; /* ~3 rows (80px + gap) */
  overflow-y: auto;
  padding-right: 4px;
}

/* Custom scrollbar */
.file-list::-webkit-scrollbar {
  width: 6px;
}

.file-list::-webkit-scrollbar-track {
  background: rgba(255, 255, 255, 0.05);
  border-radius: 3px;
}

.file-list::-webkit-scrollbar-thumb {
  background: rgba(102, 126, 234, 0.5);
  border-radius: 3px;
}

.file-list::-webkit-scrollbar-thumb:hover {
  background: rgba(102, 126, 234, 0.7);
}

.file-item {
  position: relative;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  cursor: move;
  transition: transform 0.15s ease, box-shadow 0.15s ease, border-color 0.15s ease;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  aspect-ratio: 1;
}

.file-item:hover {
  transform: scale(1.02);
  border-color: rgba(102, 126, 234, 0.5);
}

.file-item-img {
  height: 100%;
  width: 100%;
  object-fit: cover;
  display: block;
}

.file-item-loading {
  height: 100%;
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255, 255, 255, 0.05);
}

.spinner-small {
  width: 20px;
  height: 20px;
  border: 2px solid rgba(102, 126, 234, 0.3);
  border-top-color: #667eea;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.file-item-remove {
  position: absolute;
  top: 10px;
  right: 10px;
  padding: 2px 2px;
  width: 25px;
  height: 25px;
  border: none;
  background: rgba(239, 68, 68, 0.9);
  color: white;
  border-radius: 4px;
  cursor: pointer;
  font-size: 10px;
  opacity: 0;
  transition: opacity 0.2s ease, background 0.2s ease;
  z-index: 2;
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
  border: 3px dashed #667eea;
}

.chosen-item {
  opacity: 0.9;
  transform: scale(0.97);
  cursor: grabbing;
}

.drag-item {
  border: 2px solid #667eea !important;
  box-shadow: 0 0 8px rgba(102, 126, 234, 0.4);
}
</style>