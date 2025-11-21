<template>
  <div class="page-content">
    <div class="page-header">
      <h2>Tạo Video</h2>
    </div>
    <div class="page-body">
      <div class="create-video-section">
        <!-- Chọn Audio Files (MP3) -->
        <div class="file-group">
          <label class="form-label">🎵 File Audio (MP3)</label>
          <div class="file-selector">
            <button class="btn-select-file" @click="selectAudioFiles">
              📁 Chọn File MP3
            </button>
            <span class="file-count">{{ audioFiles.length }} file đã chọn</span>
          </div>
          <div v-if="audioFiles.length > 0" class="file-list">
            <div v-for="(file, index) in audioFiles" :key="index" class="file-item">
              <span class="file-item-name">{{ getFileName(file) }}</span>
              <button class="file-item-remove" @click="removeAudioFile(index)" title="Xóa">✕</button>
            </div>
          </div>
        </div>

        <!-- Chọn Ảnh -->
        <div class="file-group">
          <label class="form-label">🖼️ Ảnh</label>
          <div class="file-selector">
            <button class="btn-select-file" @click="selectImageFiles">
              📁 Chọn Ảnh
            </button>
            <span class="file-count">{{ imageFiles.length }} file đã chọn</span>
          </div>
          <div v-if="imageFiles.length > 0" class="file-list">
            <div v-for="(file, index) in imageFiles" :key="index" class="file-item">
              <span class="file-item-name">{{ getFileName(file) }}</span>
              <button class="file-item-remove" @click="removeImageFile(index)" title="Xóa">✕</button>
            </div>
          </div>
        </div>

        <!-- Chọn Video (MP4) -->
        <div class="file-group">
          <label class="form-label">🎬 Video Nguồn (MP4)</label>
          <div class="file-selector">
            <button class="btn-select-file" @click="selectVideoFiles">
              📁 Chọn File MP4
            </button>
            <span class="file-count">{{ videoFiles.length }} file đã chọn</span>
          </div>
          <div v-if="videoFiles.length > 0" class="file-list">
            <div v-for="(file, index) in videoFiles" :key="index" class="file-item">
              <span class="file-item-name">{{ getFileName(file) }}</span>
              <button class="file-item-remove" @click="removeVideoFile(index)" title="Xóa">✕</button>
            </div>
          </div>
        </div>

        <!-- Chọn Chất Lượng Video -->
        <div class="file-group">
          <label class="form-label">⚙️ Chất Lượng Video</label>
          <select v-model="quality" class="quality-select" title="Chọn chất lượng video">
            <option value="1080p">Full HD (1080p)</option>
            <option value="720p">HD (720p)</option>
            <option value="480p">SD (480p)</option>
            <option value="360p">360p</option>
          </select>
        </div>

        <!-- Chọn Thư Mục Lưu -->
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

        <!-- Tên File Output -->
        <div class="file-group">
          <label class="form-label">📝 Tên File Output</label>
          <input 
            type="text" 
            v-model="outputFilename"
            class="form-input" 
            placeholder="output_video.mp4"
          />
        </div>

        <!-- Button Tạo Video -->
        <div class="form-actions">
          <button 
            class="btn-create-video" 
            @click="createVideo" 
            :disabled="isCreating"
            id="btn-create-video"
          >
            {{ isCreating ? '⏳ Đang tạo video...' : '✨ Tạo Video' }}
          </button>
          <button 
            v-if="isCreating"
            class="btn-stop-video" 
            @click="stopVideo"
            id="btn-stop-video"
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
  </div>
</template>

<script setup>
import { ref } from 'vue'
import { useTauri } from '../composables/useTauri'
import { useFileSelection } from '../composables/useFileSelection'

const { callCommand } = useTauri()
const { selectFiles, selectFolder } = useFileSelection()

const audioFiles = ref([])
const imageFiles = ref([])
const videoFiles = ref([])
const quality = ref('1080p')
const outputFolder = ref('')
const outputFilename = ref('output_video.mp4')
const isCreating = ref(false)
const statusMessage = ref('')
const statusType = ref('info')
const progress = ref(0)
const currentProcessId = ref(null)

const getFileName = (filePath) => {
  return filePath.split('/').pop() || filePath.split('\\').pop() || filePath
}

const selectAudioFiles = async () => {
  try {
    const selected = await selectFiles(['mp3', 'm4a', 'wav'], true)
    if (selected && selected.length > 0) {
      audioFiles.value = selected
    }
  } catch (error) {
    statusMessage.value = 'Lỗi khi chọn file audio: ' + error
    statusType.value = 'error'
  }
}

const selectImageFiles = async () => {
  try {
    const selected = await selectFiles(['jpg', 'jpeg', 'png', 'gif', 'webp'], true)
    if (selected && selected.length > 0) {
      imageFiles.value = selected
    }
  } catch (error) {
    statusMessage.value = 'Lỗi khi chọn ảnh: ' + error
    statusType.value = 'error'
  }
}

const selectVideoFiles = async () => {
  try {
    const selected = await selectFiles(['mp4', 'mov', 'avi'], true)
    if (selected && selected.length > 0) {
      videoFiles.value = selected
    }
  } catch (error) {
    statusMessage.value = 'Lỗi khi chọn video: ' + error
    statusType.value = 'error'
  }
}

const selectOutputFolder = async () => {
  try {
    const selected = await selectFolder()
    if (selected) {
      outputFolder.value = selected
      statusMessage.value = '✅ Đã chọn thư mục: ' + selected
      statusType.value = 'success'
    }
  } catch (error) {
    statusMessage.value = 'Lỗi khi chọn thư mục: ' + error
    statusType.value = 'error'
  }
}

const removeAudioFile = (index) => {
  audioFiles.value.splice(index, 1)
}

const removeImageFile = (index) => {
  imageFiles.value.splice(index, 1)
}

const removeVideoFile = (index) => {
  videoFiles.value.splice(index, 1)
}

const createVideo = async () => {
  if (audioFiles.value.length === 0 && imageFiles.value.length === 0 && videoFiles.value.length === 0) {
    statusMessage.value = 'Vui lòng chọn ít nhất một file (audio, ảnh hoặc video)'
    statusType.value = 'error'
    return
  }
  
  if (!outputFolder.value.trim()) {
    statusMessage.value = 'Vui lòng chọn thư mục để lưu video'
    statusType.value = 'error'
    return
  }
  
  isCreating.value = true
  progress.value = 10
  statusMessage.value = '⏳ Đang bắt đầu tạo video...'
  statusType.value = 'info'
  
  try {
    const processId = await callCommand('start_create_video', {
      audioFiles: audioFiles.value,
      imageFiles: imageFiles.value,
      videoFiles: videoFiles.value,
      outputFolder: outputFolder.value.trim(),
      outputFilename: outputFilename.value.trim() || 'output_video.mp4',
      quality: quality.value
    })
    
    currentProcessId.value = processId
    
    const result = await callCommand('wait_for_video_creation', {
      processId
    })
    
    currentProcessId.value = null
    progress.value = 100
    statusMessage.value = '✅ ' + result
    statusType.value = 'success'
  } catch (error) {
    currentProcessId.value = null
    if (error.includes('đã bị hủy') || error.includes('cancelled')) {
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

const stopVideo = async () => {
  if (!currentProcessId.value) return
  
  try {
    await callCommand('stop_video_creation', {
      processId: currentProcessId.value
    })
    currentProcessId.value = null
    statusMessage.value = '⚠️ Đã gửi lệnh dừng quá trình tạo video...'
    statusType.value = 'info'
  } catch (error) {
    statusMessage.value = '❌ Lỗi khi dừng: ' + error
    statusType.value = 'error'
  }
}
</script>

