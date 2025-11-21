<template>
  <div class="page-content">
    <div class="page-header">
      <h2>Download Video</h2>
    </div>
    <div class="page-body">
      <div class="download-section">
        <div class="form-group">
          <label for="youtube-url" class="form-label">
            🔗 Link YouTube
          </label>
          <div class="input-group">
            <input 
              type="text" 
              id="youtube-url" 
              v-model="youtubeUrl"
              class="form-input" 
              placeholder="https://www.youtube.com/watch?v=..."
              autocomplete="off"
            />
          </div>
        </div>

        <div class="form-group">
          <label for="save-folder" class="form-label">
            📁 Thư Mục Lưu Video
          </label>
          <div class="input-group">
            <input 
              type="text" 
              id="save-folder" 
              v-model="saveFolder"
              class="form-input" 
              placeholder="Chọn thư mục để lưu video..."
              readonly
            />
            <button class="btn-select-folder" @click="selectSaveFolder">
              Chọn Thư Mục
            </button>
          </div>
        </div>

        <div class="form-actions">
          <button 
            class="btn-download" 
            @click="downloadVideo" 
            :disabled="isDownloading"
            id="btn-download"
          >
            {{ isDownloading ? '⏳ Đang xử lý...' : '⬇️ Download Video' }}
          </button>
        </div>

        <div :class="['download-status', statusType]" v-if="statusMessage">
          {{ statusMessage }}
        </div>
        <div v-if="isDownloading" class="download-progress">
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
const { selectFolder } = useFileSelection()

const youtubeUrl = ref('')
const saveFolder = ref('')
const isDownloading = ref(false)
const statusMessage = ref('')
const statusType = ref('info')
const progress = ref(0)

const selectSaveFolder = async () => {
  try {
    const selected = await selectFolder()
    if (selected) {
      saveFolder.value = selected
      statusMessage.value = '✅ Đã chọn thư mục: ' + selected
      statusType.value = 'success'
    }
  } catch (error) {
    statusMessage.value = 'Lỗi khi chọn thư mục: ' + error
    statusType.value = 'error'
  }
}

const downloadVideo = async () => {
  if (!youtubeUrl.value.trim()) {
    statusMessage.value = 'Vui lòng nhập link YouTube'
    statusType.value = 'error'
    return
  }
  
  if (!saveFolder.value.trim()) {
    statusMessage.value = 'Vui lòng chọn thư mục để lưu video'
    statusType.value = 'error'
    return
  }
  
  isDownloading.value = true
  progress.value = 10
  statusMessage.value = '⏳ Đang bắt đầu download video...'
  statusType.value = 'info'
  
  try {
    const result = await callCommand('download_youtube_video', {
      url: youtubeUrl.value.trim(),
      savePath: saveFolder.value.trim()
    })
    
    progress.value = 100
    statusMessage.value = '✅ ' + result
    statusType.value = 'success'
    
    setTimeout(() => {
      youtubeUrl.value = ''
      progress.value = 0
    }, 2000)
  } catch (error) {
    statusMessage.value = '❌ Lỗi: ' + error
    statusType.value = 'error'
  } finally {
    isDownloading.value = false
  }
}
</script>

