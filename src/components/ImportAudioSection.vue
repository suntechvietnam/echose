<template>
  <div class="file-group audio-selection-group">
    <h3 class="form-label">Danh sách file audio</h3>
    <div class="file-selector">
      <button class="btn-select-file" @click="selectAudioFiles">
        🎵 Chọn file audio
      </button>
      <button 
        v-if="audioFiles.length > 0"
        class="btn-clear-all" 
        @click="clearAllAudio"
      >
        🗑️ Xóa Tất Cả
      </button>
      <button 
        v-if="audioFiles.length > 1"
        class="btn-shuffle-images" 
        @click="shuffleAudio"
      >
        🔀 Sắp Xếp Ngẫu Nhiên
      </button>
      <span class="file-count">Đang có {{ audioFiles.length }} files</span>
    </div>

    <!-- Danh sách file nhạc -->
    <draggable
      v-if="audioFiles.length > 0"
      v-model="audioFiles"
      class="file-list"
      ghost-class="ghost-item"
      chosen-class="chosen-item"
      drag-class="drag-item"
      :animation="200"
    >
      <template #item="{ element: file, index }">
        <div class="file-item" :key="file">
          <div class="file-item-info">
            <span class="file-item-name">{{ getFileName(file) }}</span>
            <span v-if="fileDurations[file]" class="file-item-duration">
              <span class="duration-icon">⏱️</span> {{ formatDuration(fileDurations[file], { showSeconds: true, roundUp: false }) }}
            </span>
            <span v-else class="file-item-duration loading">
              Loading time...
            </span>
          </div>
          <button class="file-item-remove" @click="removeAudioFile(index)" title="Xóa">✕</button>
        </div>
      </template>
    </draggable>

    <!-- Tổng thời lượng -->
    <div v-if="audioFiles.length > 0" class="total-duration">
      <strong>Tổng thời lượng: {{ formatDuration(totalDuration) }}</strong>
    </div>
  </div>
</template>

<script setup>
import { ref, watch } from 'vue'
import draggable from 'vuedraggable'
import { open } from '@tauri-apps/api/dialog'
import { useAudioDuration } from '@/composables/useAudioDuration'

const emit = defineEmits(['update:audioFiles', 'audio-selected', 'audio-cleared'])

// Audio files
const audioFiles = ref([])

// Use composable for audio duration
const { fileDurations, loadMultipleDurations, removeDuration, clearAllDurations, createTotalDuration } = useAudioDuration()

// Tạo computed totalDuration từ composable
const totalDuration = createTotalDuration(audioFiles)

const getFileName = (filePath) => {
  return filePath.split('/').pop() || filePath.split('\\').pop() || filePath
}

/**
 * Format duration từ giây sang định dạng dễ đọc
 * @param {number} seconds - Thời lượng tính bằng giây
 * @param {object} options - Tùy chọn format
 * @param {boolean} options.showSeconds - Hiển thị giây (mặc định: true)
 * @param {boolean} options.roundUp - Làm tròn lên nếu có giây (mặc định: false)
 * @returns {string} - Chuỗi định dạng thời lượng
 */
const formatDuration = (seconds, options = {}) => {
  const { showSeconds = true, roundUp = false } = options
  
  if (!seconds || isNaN(seconds)) {
    return showSeconds ? '0 phút 0 giây' : '0 phút'
  }
  
  const totalMins = Math.floor(seconds / 60)
  const secs = Math.floor(seconds % 60)
  
  // Xử lý >= 60 phút (hiển thị giờ)
  if (totalMins >= 60) {
    const hours = Math.floor(totalMins / 60)
    const mins = totalMins % 60
    
    if (mins === 0) {
      return `${hours} giờ`
    } else {
      return `${hours} giờ ${mins} phút`
    }
  }
  
  // Xử lý < 60 phút
  if (roundUp && secs > 0) {
    // Làm tròn lên nếu có giây
    return `${totalMins + 1} phút`
  }
  
  if (secs === 0) {
    return `${totalMins} phút`
  } else {
    return showSeconds 
      ? `${totalMins} phút ${secs} giây`
      : `${totalMins} phút`
  }
}

const selectAudioFiles = async () => {
  try {
    const selected = await open({
      multiple: true,
      filters: [{
        name: 'Audio Files',
        extensions: ['mp3', 'wav', 'm4a', 'aac', 'ogg']
      }]
    })
    
    if (selected) {
      const files = Array.isArray(selected) ? selected : [selected]
      
      // Thêm các file mới vào danh sách (tránh trùng lặp)
      const newFiles = []
      for (const file of files) {
        if (!audioFiles.value.includes(file)) {
          audioFiles.value.push(file)
          newFiles.push(file)
        }
      }
      
      // Load duration cho các file mới (song song)
      if (newFiles.length > 0) {
        loadMultipleDurations(newFiles)
      }
      
      emit('update:audioFiles', audioFiles.value)
      emit('audio-selected', audioFiles.value)
    }
  } catch (error) {
    console.error('Lỗi khi chọn file nhạc:', error)
  }
}

const removeAudioFile = (index) => {
  const fileToRemove = audioFiles.value[index]
  audioFiles.value.splice(index, 1)
  
  // Xóa duration khi xóa file
  removeDuration(fileToRemove)
  
  emit('update:audioFiles', audioFiles.value)
}

const clearAllAudio = () => {
  audioFiles.value = []
  clearAllDurations()
  emit('update:audioFiles', [])
  emit('audio-cleared')
}

const shuffleAudio = () => {
  if (audioFiles.value.length <= 1) return
  
  const shuffled = [...audioFiles.value]
  // Fisher-Yates shuffle algorithm
  for (let i = shuffled.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [shuffled[i], shuffled[j]] = [shuffled[j], shuffled[i]]
  }
  audioFiles.value = shuffled
  emit('update:audioFiles', audioFiles.value)
}

// Watch để emit khi audioFiles thay đổi
watch(audioFiles, (newFiles) => {
  emit('update:audioFiles', newFiles)
}, { deep: true })

</script>

<style scoped>
.audio-selection-group {
  margin-top: 1.5rem;
  flex: 0 0 auto;
  max-height: 300px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
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

.file-list {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 0.75rem;
  flex: 1;
  min-height: 0;
  overflow-y: auto;
}

.file-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.75rem;
  border: 1px solid #ddd;
  border-radius: 4px;
  background: white;
  cursor: move;
  transition: all 0.2s ease;
}

.file-item:hover {
  border-color: #667eea;
  box-shadow: 0 2px 4px rgba(102, 126, 234, 0.1);
}

.file-item-info {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  flex: 1;
}

.file-item-name {
  font-weight: 500;
  color: #333;
}

.file-item-duration {
  font-size: 0.85rem;
  color: #666;
  margin-top: 0.25rem;
  display: flex;
  align-items: center;
  gap: 0.25rem;
}

.duration-icon {
  font-size: 0.75rem;
  line-height: 1;
}

.file-item-duration.loading {
  color: #999;
  font-style: italic;
}

.file-item-remove {
  padding: 0.25rem 0.5rem;
  border: none;
  background: #ff4444;
  color: white;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.9rem;
  opacity: 0;
  transition: opacity 0.2s ease, background 0.2s ease;
  z-index: 1000;
  position: relative;
}

.file-item-info:hover ~ .file-item-remove,
.file-item:hover .file-item-remove {
  opacity: 1;
}

.file-item-remove:hover {
  background: #cc0000;
}

.total-duration {
  margin-top: 1rem;
  padding: 0.75rem;
  background: #f0f0f0;
  border-radius: 4px;
  text-align: center;
  color: #333;
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

