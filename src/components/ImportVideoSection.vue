<template>
  <div class="file-group video-selection-group">
    <h3 class="form-label">
      Danh sách video 
      <span class="video-count-badge" v-if="videoFiles.length > 0">
        (đang có {{ videoFiles.length }} video)
      </span>
    </h3>
    <div class="file-selector">
      <button class="btn-select-file" @click="selectVideoFiles">
        🎬 Chọn video
      </button>
      <button 
        v-if="videoFiles.length > 0"
        class="btn-clear-all" 
        @click="clearAllVideos"
      >
        🗑️ Xóa tất cả
      </button>
      <button 
        v-if="videoFiles.length > 1"
        class="btn-shuffle-videos" 
        @click="shuffleVideos"
        :disabled="isShuffling"
      >
        {{ isShuffling ? 'Đang sắp xếp...' : '🔀 Sắp xếp ngẫu nhiên' }}
      </button>
      <div 
        v-if="videoFiles.length > 0"
        class="checkbox-container-inline"
      >
        <input 
          type="checkbox" 
          id="mute-original-audio-checkbox"
          v-model="removeOriginalAudio"
          @change="handleRemoveAudioChange"
          class="checkbox-input"
        />
        <label for="mute-original-audio-checkbox" class="checkbox-label">Xóa audio gốc</label>
      </div>
    </div>
  
    <!-- Danh sách video -->
    <draggable
      v-if="videoFiles.length > 0"
      v-model="videoFiles"
      class="file-list"
      ghost-class="ghost-item"
      chosen-class="chosen-item"
      drag-class="drag-item"
      :animation="200"
      :item-key="videoItemKey"
    >
      <template #item="{ element: file, index }">
        <div class="file-item">
          <div
            class="file-item-info"
            @click="openVideoPreview(file)"
          >
            <video
              :ref="el => setVideoRef(file, el)"
              :src="videoUrls[file] || getVideoUrl(file)"
              class="video-thumbnail"
              preload="metadata"
              muted
              playsinline
              @loadedmetadata="handleVideoLoaded($event, file)"
              @loadeddata="handleVideoDataLoaded($event)"
              @seeked="handleVideoSeeked($event)"
            ></video>
            <div class="video-play-overlay">
              <div class="play-icon">▶</div>
            </div>
          </div>
          <button class="file-item-remove" @click.stop="removeVideoFile(index)" title="Xóa">✕</button>
        </div>
      </template>
    </draggable>
    
    <!-- Video Preview Modal -->
    <div v-if="previewVideoUrl" class="video-preview-modal" @click="closeVideoPreview">
      <div class="video-preview-content" @click.stop>
        <button class="video-preview-close" @click="closeVideoPreview">✕</button>
        <video
          :src="previewVideoUrl"
          class="video-preview-player"
          controls
          autoplay
        ></video>
      </div>
    </div>
    
    <!-- Confirm Modal -->
    <ConfirmModal 
      v-if="isShowConfirm"
      :title="'Xác nhận xóa'"
      :message="`Bạn có chắc muốn xóa tất cả ${videoFiles.length} video?`"
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

const emit = defineEmits(['update:videoFiles', 'videos-selected', 'videos-cleared', 'status-message', 'update:removeOriginalAudio'])

// State cho checkbox xóa audio gốc
const removeOriginalAudio = ref(false)

const videoFiles = ref([...props.modelValue])
const videoUrls = ref({})
const videoRefs = ref({})
const isShuffling = ref(false)
const isShowConfirm = ref(false)
const previewVideoUrl = ref(null)

// Dùng cho vuedraggable: với array string, key chính là giá trị string
const videoItemKey = (filePath) => filePath

const getFileName = (filePath) => {
  return filePath.split('/').pop() || filePath.split('\\').pop() || filePath
}

const getVideoUrl = (filePath) => {
  if (!filePath) return null

  if (videoUrls.value[filePath]) {
    return videoUrls.value[filePath]
  }

  try {
    const url = convertFileSrc(filePath)
    videoUrls.value[filePath] = url
    return url
  } catch (error) {
    console.error('Error converting video file path:', error, filePath)
    return null
  }
}

const setVideoRef = (filePath, el) => {
  if (el) {
    videoRefs.value[filePath] = el
  }
}

const handleVideoLoaded = (event, filePath) => {
  // Khi metadata đã load, seek đến frame đầu tiên để hiển thị thumbnail
  const video = event.target
  if (video) {
    // Đảm bảo video đã sẵn sàng
    if (video.readyState >= 1) {
      video.currentTime = 0.1 // Seek đến 0.1s để đảm bảo có frame để hiển thị
    } else {
      // Nếu chưa sẵn sàng, đợi một chút rồi seek
      setTimeout(() => {
        if (video.readyState >= 1) {
          video.currentTime = 0.1
        }
      }, 100)
    }
  }
}

const handleVideoDataLoaded = (event) => {
  // Khi data đã load, đảm bảo hiển thị frame đầu tiên
  const video = event.target
  if (video && video.currentTime === 0) {
    video.currentTime = 0.1
  }
}

const handleVideoSeeked = (event) => {
  // Khi đã seek xong, pause video để giữ thumbnail
  const video = event.target
  if (video) {
    video.pause()
  }
}

const openVideoPreview = (filePath) => {
  previewVideoUrl.value = videoUrls.value[filePath] || getVideoUrl(filePath)
}

const closeVideoPreview = () => {
  previewVideoUrl.value = null
}

const selectVideoFiles = async () => {
  try {
    const selected = await open({
      multiple: true,
      filters: [{
        name: 'Videos',
        extensions: ['mp4', 'mov', 'mkv', 'avi', 'webm']
      }]
    })
    
    if (!selected) return

    const files = Array.isArray(selected) ? selected : [selected]

    for (const file of files) {
      if (!videoFiles.value.includes(file)) {
        videoFiles.value.push(file)
      }
      getVideoUrl(file)
    }

    emit('update:videoFiles', videoFiles.value)
    emit('videos-selected', videoFiles.value)
  } catch (error) {
    emit('status-message', 'Lỗi khi chọn video: ' + error, 'error')
  }
}

const removeVideoFile = (index) => {
  const fileToRemove = videoFiles.value[index]
  videoFiles.value.splice(index, 1)

  if (videoUrls.value[fileToRemove]) {
    delete videoUrls.value[fileToRemove]
  }
  
  if (videoRefs.value[fileToRemove]) {
    delete videoRefs.value[fileToRemove]
  }
  
  emit('update:videoFiles', videoFiles.value)
}

const clearAllVideos = () => {
  isShowConfirm.value = true
}

const confirmClearAll = () => {
  videoFiles.value = []
  videoUrls.value = {}
  videoRefs.value = {}
  emit('update:videoFiles', [])
  emit('videos-cleared')
  isShowConfirm.value = false
}

const cancelClearAll = () => {
  isShowConfirm.value = false
}

const shuffleVideos = async () => {
  if (videoFiles.value.length <= 1 || isShuffling.value) return
  
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
  
  const shuffled = shuffleArray(videoFiles.value)
  videoFiles.value = shuffled
  
  emit('update:videoFiles', videoFiles.value)
  
  isShuffling.value = false
}

const handleRemoveAudioChange = () => {
  emit('update:removeOriginalAudio', removeOriginalAudio.value)
}

watch(() => props.modelValue, (newValue) => {
  videoFiles.value = [...newValue]
  newValue.forEach(file => {
    getVideoUrl(file)
  })
})
</script>

<style scoped>
.video-selection-group {
  display: flex;
  flex-direction: column;
  min-height: 200px;
}

.checkbox-container-inline {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.checkbox-container-inline .checkbox-input {
  width: 18px;
  height: 18px;
  accent-color: #667eea;
  cursor: pointer;
}

.checkbox-container-inline .checkbox-label {
  font-size: 14px;
  font-weight: 500;
  color: #374151;
  cursor: pointer;
  -webkit-user-select: none;
  user-select: none;
  display: flex;
  align-items: center;
  gap: 4px;
}

.checkbox-container-inline .checkbox-label:hover {
  color: #667eea;
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

.btn-shuffle-videos {
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

.btn-shuffle-videos:hover {
  background: #059669;
}

.video-count-badge {
  font-size: 14px;
  font-weight: 400;
  color: #64748b;
  margin-left: 4px;
}

.file-count {
  font-size: 14px;
  color: #64748b;
  font-weight: 500;
}

/* Layout danh sách video (giống ảnh) */
.file-list {
  display: grid;
  grid-template-columns: repeat(6, minmax(0, 1fr));
  gap: 6px;
  flex: 1;
}

.file-item {
  position: relative;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  cursor: move;
  transition: transform 0.15s ease, box-shadow 0.15s ease, border-color 0.15s ease;
  background: transparent;
}

.file-item-info {
  height: 100%;
  min-height: 80px;
  width: 100%;
  position: relative;
  border-radius: 6px;
  overflow: hidden;
  cursor: pointer;
  background: #f1f5f9;
}

.video-thumbnail {
  width: 100%;
  height: 100%;
  min-height: 80px;
  object-fit: cover;
  display: block;
  background: #f1f5f9;
}

.video-thumbnail::-webkit-media-controls {
  display: none !important;
}

.video-thumbnail::-webkit-media-controls-enclosure {
  display: none !important;
}

.video-play-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.3);
  opacity: 0;
  transition: opacity 0.2s ease;
  pointer-events: none;
}

.file-item-info:hover .video-play-overlay {
  opacity: 1;
}

.play-icon {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.9);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 16px;
  color: #667eea;
  padding-left: 3px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
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

/* Video Preview Modal */
.video-preview-modal {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.85);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  animation: fadeIn 0.2s ease;
}

@keyframes fadeIn {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

.video-preview-content {
  position: relative;
  max-width: 90vw;
  max-height: 90vh;
  width: auto;
  height: auto;
}

.video-preview-close {
  position: absolute;
  top: -40px;
  right: 0;
  width: 35px;
  height: 35px;
  border: none;
  background: rgba(255, 255, 255, 0.9);
  color: #111827;
  border-radius: 50%;
  cursor: pointer;
  font-size: 18px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.2s ease;
  z-index: 1001;
}

.video-preview-close:hover {
  background: rgba(255, 255, 255, 1);
}

.video-preview-player {
  max-width: 90vw;
  max-height: 90vh;
  width: auto;
  height: auto;
  border-radius: 8px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.5);
}
</style>


