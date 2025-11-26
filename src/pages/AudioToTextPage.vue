<template>
  <div class="page-content">
    <div class="page-header">
      <h2>Audio / Video to Text</h2>
    </div>
    <div class="page-body image-to-video-container">
      <!-- Layout 2 cột giống ImageToVideoPage -->
      <div class="image-to-video-layout">
        <!-- Cột trái: Chọn file và hiển thị kết quả -->
        <div class="image-to-video-left-column">
          <!-- File Selection Section -->
          <div class="file-group media-selection-group">
            <h3 class="form-label">Chọn file audio hoặc video</h3>
            <div class="file-selector">
              <button class="btn-select-file" @click="selectMediaFile">
                🎬 Chọn file audio/video
              </button>
              <button 
                v-if="mediaFile"
                class="btn-clear-all" 
                @click="clearMedia"
              >
                🗑️ Xóa File
              </button>
            </div>

            <!-- File Info -->
            <div v-if="mediaFile" class="file-info">
              <div class="file-item">
                <div class="file-item-info">
                  <span class="file-item-icon">
                    {{ getFileIcon(mediaFile) }}
                  </span>
                  <div class="file-item-details">
                    <span class="file-item-name">{{ getFileName(mediaFile) }}</span>
                    <span v-if="fileDuration" class="file-item-duration">
                      <span class="duration-icon">⏱️</span> {{ formatDuration(fileDuration, { showSeconds: true, roundUp: false }) }}
                    </span>
                    <span v-else class="file-item-duration loading">
                      Loading time...
                    </span>
                  </div>
                </div>
                <button class="file-item-remove" @click="clearMedia" title="Xóa">✕</button>
              </div>
            </div>
          </div>

          <!-- Result Section -->
          <div v-if="convertedText" class="file-group result-section">
            <div class="result-section-header">
              <h3 class="form-label">Kết quả</h3>
              <div class="result-actions">
                <button class="btn-select-file btn-copy" @click="copyToClipboard">
                  📋 Sao chép
                </button>
                <button class="btn-select-file btn-download" @click="downloadText">
                  💾 Tải xuống
                </button>
              </div>
            </div>
 
            <div class="form-group">
              <textarea 
                ref="textOutputRef"
                v-model="convertedText" 
                class="form-input text-output"
                readonly
                placeholder="Văn bản sẽ hiển thị ở đây..."
              ></textarea>
            </div>
          </div>
        </div>

        <!-- Cột phải: Convert button và Options -->
        <div class="image-to-video-right-column">
          <!-- Output Format Selection -->
          <div class="file-group">
            <div class="form-group">
              <label class="form-label">📄 Định dạng file output</label>
              <div class="output-format-list">
                <div 
                  v-for="format in outputFormats" 
                  :key="format.value"
                  :class="['format-option', { active: outputFormat === format.value }]"
                  @click="outputFormat = format.value"
                >
                  <span class="format-icon">{{ format.icon }}</span>
                  <span class="format-name">{{ format.name }}</span>
                  <span class="format-desc">{{ format.desc }}</span>
                  <span v-if="outputFormat === format.value" class="format-check">✓</span>
                </div>
              </div>
            </div>
          </div>

          <!-- Convert Button -->
          <div class="form-actions">
            <button 
              class="btn-create-video" 
              :disabled="!mediaFile || isConverting"
              @click="handleConvert"
            >
              {{ isConverting ? 'Đang chuyển đổi...' : `Chuyển đổi sang ${outputFormat.toUpperCase()}` }}
            </button>
          </div>

          <!-- Status message -->
          <div v-if="statusMessage" :class="['download-status', statusType]">
            <span>{{ statusMessage }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, watch, nextTick } from 'vue'
import { open, save } from '@tauri-apps/api/dialog'
import { invoke } from '@tauri-apps/api/tauri'
import { readTextFile, writeTextFile } from '@tauri-apps/api/fs'
import { useAudioDuration } from '@/composables/useAudioDuration'
import '@/assets/css/image-to-video.css'
import '@/assets/css/audio-to-text.css'

// Media file (single file only)
const mediaFile = ref(null)
const fileDuration = ref(null)

// Use composable for duration (works for both audio and video)
const { loadFileDuration } = useAudioDuration()

// Convert state
const isConverting = ref(false)
const convertedText = ref('')
const statusMessage = ref('')
const statusType = ref('info')
const textOutputRef = ref(null)

// Auto resize textarea theo nội dung
const autoResizeTextarea = () => {
  nextTick(() => {
    if (textOutputRef.value) {
      textOutputRef.value.style.height = 'auto'
      textOutputRef.value.style.height = textOutputRef.value.scrollHeight + 'px'
    }
  })
}

// Watch convertedText để tự động resize
watch(convertedText, () => {
  autoResizeTextarea()
})

// Output options
const outputFormat = ref('txt')
const outputFormats = [
  { value: 'txt', name: 'TXT', desc: 'Văn bản', icon: '📄' },
  { value: 'srt', name: 'SRT', desc: 'Caption đơn giản', icon: '🎬' },
  { value: 'ass', name: 'ASS', desc: 'Caption nâng cao', icon: '💬' },
]

const getFileName = (filePath) => {
  return filePath.split('/').pop() || filePath.split('\\').pop() || filePath
}

const getFileIcon = (filePath) => {
  const ext = filePath.split('.').pop()?.toLowerCase()
  if (['mp3', 'wav', 'm4a', 'aac', 'ogg', 'flac'].includes(ext)) {
    return '🎵'
  } else if (['mp4', 'avi', 'mov', 'mkv', 'webm', 'flv'].includes(ext)) {
    return '🎬'
  }
  return '📄'
}

/**
 * Format duration từ giây sang định dạng dễ đọc
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

const selectMediaFile = async () => {
  try {
    const selected = await open({
      multiple: false,
      filters: [{
        name: 'Media Files',
        extensions: ['mp3', 'wav', 'm4a', 'aac', 'ogg', 'flac', 'mp4', 'avi', 'mov', 'mkv', 'webm', 'flv']
      }]
    })
    
    if (selected) {
      const file = Array.isArray(selected) ? selected[0] : selected
      mediaFile.value = file
      fileDuration.value = null
      convertedText.value = ''
      
      // Load duration cho file
      try {
        const duration = await loadFileDuration(file)
        fileDuration.value = duration
      } catch (error) {
        console.error('Lỗi khi load duration:', error)
      }
      
      statusMessage.value = '✅ Đã chọn file'
      statusType.value = 'success'
    }
  } catch (error) {
    console.error('Lỗi khi chọn file:', error)
    statusMessage.value = '❌ Lỗi khi chọn file: ' + error
    statusType.value = 'error'
  }
}

const clearMedia = () => {
  mediaFile.value = null
  fileDuration.value = null
  convertedText.value = ''
  statusMessage.value = '✅ Đã xóa file'
  statusType.value = 'success'
}

const handleConvert = async () => {
  if (!mediaFile.value) return
  
  isConverting.value = true
  convertedText.value = ''
  statusMessage.value = `⏳ Đang chuyển đổi audio sang ${outputFormat.value.toUpperCase()}...`
  statusType.value = 'info'
  
  try {
    // Gọi Tauri command tương ứng với format
    let outputPath
    if (outputFormat.value === 'ass') {
      outputPath = await invoke('convert_audio_to_ass', {
        inputPath: mediaFile.value,
        outputAssPath: null // Để tự động tạo tên file
      })
    } else {
      // TODO: Implement other formats
      throw new Error(`Định dạng ${outputFormat.value.toUpperCase()} sẽ được implement trong phiên bản tiếp theo`)
    }
    
    // Đọc file đã tạo
    const content = await readTextFile(outputPath)
    convertedText.value = content
    autoResizeTextarea()
    
    statusMessage.value = `✅ Chuyển đổi hoàn tất! File ${outputFormat.value.toUpperCase()} đã được tạo tại: ${outputPath}`
    statusType.value = 'success'
  } catch (error) {
    console.error('Lỗi khi chuyển đổi:', error)
    statusMessage.value = `❌ Có lỗi xảy ra khi chuyển đổi: ${error}`
    statusType.value = 'error'
  } finally {
    isConverting.value = false
  }
}

const copyToClipboard = async () => {
  if (!convertedText.value) return
  
  try {
    await navigator.clipboard.writeText(convertedText.value)
    statusMessage.value = '✅ Đã sao chép vào clipboard!'
    statusType.value = 'success'
  } catch (error) {
    console.error('Lỗi khi sao chép:', error)
    statusMessage.value = '❌ Lỗi khi sao chép'
    statusType.value = 'error'
  }
}

const downloadText = async () => {
  if (!convertedText.value) return
  
  try {
    // Tạo tên file mặc định từ file input
    let defaultName = `caption.${outputFormat.value}`
    if (mediaFile.value) {
      const inputName = getFileName(mediaFile.value)
      const baseName = inputName.replace(/\.[^/.]+$/, '') // Bỏ extension
      defaultName = `${baseName}_caption.${outputFormat.value}`
    }
    
    // Tạo filter tương ứng với format
    const formatFilters = {
      txt: { name: 'Text File', extensions: ['txt'] },
      srt: { name: 'SRT Subtitle', extensions: ['srt'] },
      ass: { name: 'ASS Subtitle', extensions: ['ass'] },
      vtt: { name: 'WebVTT Subtitle', extensions: ['vtt'] },
      json: { name: 'JSON File', extensions: ['json'] }
    }
    
    // Mở dialog để chọn nơi lưu file
    const savePath = await save({
      defaultPath: defaultName,
      filters: [
        formatFilters[outputFormat.value] || { name: 'Text File', extensions: ['txt'] },
        { name: 'All Files', extensions: ['*'] }
      ]
    })
    
    if (savePath) {
      // Ghi nội dung vào file
      await writeTextFile(savePath, convertedText.value)
      
      statusMessage.value = `✅ Đã lưu file thành công: ${savePath}`
      statusType.value = 'success'
    }
  } catch (error) {
    console.error('Lỗi khi lưu file:', error)
    statusMessage.value = `❌ Lỗi khi lưu file: ${error}`
    statusType.value = 'error'
  }
}

</script>

