<template>
  <div class="page-content">
    <div class="page-header">
      <h2>Quản Lý File Nhạc Nguồn</h2>
    </div>
    <div class="page-body">
      <!-- Audio section -->
      <div class="audio-mix-section">
        <!-- Chọn File MP3 -->
        <div class="file-group">
          <label class="form-label">🎵 Chọn File MP3</label>
          <div class="file-selector">
            <button class="btn-select-file" @click="selectAudioFiles">
              📁 Chọn File MP3
            </button>
            <span class="file-count">{{ audioFiles.length }} file đã chọn</span>
          </div>
          
          <!-- Danh sách file nguồn -->
          <draggable
            v-if="audioFiles.length > 0"
            v-model="audioFiles"
            class="file-list file-list-grid"
            :animation="200"
            ghost-class="ghost-item"
            chosen-class="chosen-item"
          >
            <template #item="{ element: file, index }">
              <div class="file-item" :key="file">
                <div class="file-item-info">
                  <span class="file-item-name">{{ getAudioFileName(file) }}</span>
                  <span v-if="fileDurations[file]" class="file-item-duration">Thời lượng: {{ formatAudioDuration(fileDurations[file]) }}</span>
                  <span v-else class="file-item-duration loading">Đang tải...</span>
                </div>
                <button class="file-item-remove" @click="removeFile(index)" title="Xóa">✕</button>
              </div>
            </template>
          </draggable>
          
          <!-- Status bar -->
          <div v-if="audioFiles.length > 0" class="file-list-status-bar">
            <div class="status-bar-info">
              <span class="status-bar-label">📊 Tổng thời lượng:</span>
              <span class="status-bar-value">{{ formatAudioDuration(totalDuration) }}</span>
            </div>
            <button class="btn-shuffle" @click="shuffleFiles" title="Sắp xếp ngẫu nhiên">
              🔀 Sắp xếp ngẫu nhiên
            </button>
          </div>
        </div>

        <!-- Chọn File Video Nguồn -->
        <div class="file-group">
          <label class="form-label">🎬 Chọn File MP4</label>
          <div class="file-selector">
            <button class="btn-select-file" @click="selectVideoFiles">
              📁 Chọn File MP4
            </button>
            <span class="file-count">{{ videoFiles.length }} file đã chọn</span>
          </div>
          
          <!-- Danh sách file video nguồn -->
          <draggable
            v-if="videoFiles.length > 0"
            v-model="videoFiles"
            class="file-list file-list-grid"
            :animation="200"
            ghost-class="ghost-item"
            chosen-class="chosen-item"
          >
            <template #item="{ element: file, index }">
              <div class="file-item" :key="file">
                <div class="file-item-info">
                  <span class="file-item-name">{{ getVideoFileName(file) }}</span>
                  <span v-if="videoDurations[file]" class="file-item-duration">Thời lượng: {{ formatVideoDuration(videoDurations[file]) }}</span>
                  <span v-else class="file-item-duration loading">Đang tải...</span>
                </div>
                <button class="file-item-remove" @click="removeVideoFile(index)" title="Xóa">✕</button>
              </div>
            </template>
          </draggable>
          
          <!-- Status bar -->
          <div v-if="videoFiles.length > 0" class="file-list-status-bar">
            <div class="status-bar-info">
              <span class="status-bar-label">📊 Tổng thời lượng:</span>
              <span class="status-bar-value">{{ formatVideoDuration(totalVideoDuration) }}</span>
            </div>
            <button class="btn-shuffle" @click="shuffleVideoFiles" title="Sắp xếp ngẫu nhiên">
              🔀 Sắp xếp ngẫu nhiên
            </button>
          </div>
        </div>

        <!-- Chọn Thư Mục Lưu và Tên File Output -->
        <div class="file-group">
          <div style="display: flex; gap: 20px; align-items: flex-start;">
            <div style="flex: 1;">
              <label class="form-label">💾 Thư Mục Lưu File Video</label>
              <div class="input-group">
                <input 
                  type="text" 
                  v-model="outputFolder"
                  class="form-input" 
                  placeholder="Chọn thư mục để lưu file video..."
                  readonly
                />
                <button class="btn-select-folder" @click="selectOutputFolder">
                  Chọn Thư Mục
                </button>
              </div>
            </div>
            <div style="flex: 1;">
              <label class="form-label">📝 Tên File Output</label>
              <input 
                type="text" 
                v-model="outputFilename"
                class="form-input" 
                placeholder="output_video.mp4"
              />
            </div>
          </div>
        </div>

        <!-- Button video -->
        <div class="form-actions">
          <button 
            class="btn-create-video" 
            @click="createAudioMix" 
            :disabled="isCreating"
            id="btn-create-audio"
          >
            {{ isCreating ? '⏳ Đang tạo video...' : '🎵 Tạo video' }}
          </button>
          <button 
            v-if="isCreating"
            class="btn-stop-video" 
            @click="stopAudioCreation"
            id="btn-stop-audio"
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
import draggable from 'vuedraggable'
import { useTauri } from '../composables/useTauri'
import { useFileSelection } from '../composables/useFileSelection'
import { useAudioFiles } from '../composables/useAudioFiles'
import { useVideoFiles } from '../composables/useVideoFiles'

const { callCommand } = useTauri()
const { selectFolder } = useFileSelection()

// Audio files logic - completely separated
const {
  audioFiles,
  fileDurations,
  getFileName: getAudioFileName,
  formatDuration: formatAudioDuration,
  totalDuration,
  removeFile,
  shuffleFiles,
  selectAudioFiles
} = useAudioFiles()

// Video files logic - completely separated
const {
  videoFiles,
  videoDurations,
  getFileName: getVideoFileName,
  formatDuration: formatVideoDuration,
  totalVideoDuration,
  removeVideoFile,
  shuffleVideoFiles,
  selectVideoFiles
} = useVideoFiles()

// Output configuration
const outputFolder = ref('')
const outputFilename = ref('output_video.mp4')
const isCreating = ref(false)
const statusMessage = ref('')
const statusType = ref('info')
const progress = ref(0)
const currentProcessId = ref(null)

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

const createAudioMix = async () => {
  if (audioFiles.value.length === 0) {
    statusMessage.value = 'Vui lòng chọn ít nhất một file MP3'
    statusType.value = 'error'
    return
  }
  
  if (videoFiles.value.length === 0) {
    statusMessage.value = 'Vui lòng chọn ít nhất một file MP4'
    statusType.value = 'error'
    return
  }
  
  if (totalDuration.value <= 0) {
    statusMessage.value = 'Vui lòng đợi thời lượng của các file được tải xong'
    statusType.value = 'error'
    return
  }
  
  // Kiểm tra thời lượng video đã được load chưa
  const hasVideoLoading = videoFiles.value.some(file => !videoDurations.value[file])
  if (hasVideoLoading) {
    statusMessage.value = 'Vui lòng đợi thời lượng của các file video được tải xong'
    statusType.value = 'error'
    return
  }
  
  if (!outputFolder.value.trim()) {
    statusMessage.value = 'Vui lòng chọn thư mục để lưu file video'
    statusType.value = 'error'
    return
  }
  
  isCreating.value = true
  progress.value = 5
  statusMessage.value = `⏳ Đang bắt đầu tạo video với tổng thời lượng ${formatAudioDuration(totalDuration.value)}...`
  statusType.value = 'info'
  
  try {
    const outputFolderPath = outputFolder.value.trim()
    
    // Step 1: Ghép MP3 files
    progress.value = 10
    statusMessage.value = '📝 Bước 1/3: Đang ghép các file MP3...'
    
    const step1Id = await callCommand('create_merged_mp3_step1', {
      audioFiles: audioFiles.value,
      outputFolder: outputFolderPath
    })
    
    currentProcessId.value = step1Id
    const step1Result = await callCommand('wait_for_video_creation', {
      processId: step1Id
    })
    
    if (step1Result.includes('Lỗi')) {
      throw new Error(step1Result)
    }
    
    // Lấy merged MP3 path và duration
    const mergedMp3Path = await callCommand('get_step1_merged_mp3_path', {
      processId: step1Id,
      outputFolder: outputFolderPath
    })
    const mp3Duration = await callCommand('get_audio_duration', {
      filePath: mergedMp3Path
    })
    
    progress.value = 33
    statusMessage.value = '✅ Bước 1/3 hoàn thành: Đã ghép MP3 thành công'
    
    // Step 2: Ghép MP4 files
    progress.value = 40
    statusMessage.value = '📝 Bước 2/3: Đang ghép các file MP4 (có thể mất vài phút)...'
    
    const step2Id = await callCommand('create_merged_video_step2', {
      videoFiles: videoFiles.value,
      mp3Duration: mp3Duration,
      outputFolder: outputFolderPath
    })
    
    currentProcessId.value = step2Id
    const step2Result = await callCommand('wait_for_video_creation', {
      processId: step2Id
    })
    
    if (step2Result.includes('Lỗi')) {
      throw new Error(step2Result)
    }
    
    // Lấy merged video path
    const mergedVideoPath = await callCommand('get_step2_merged_video_path', {
      processId: step2Id,
      outputFolder: outputFolderPath
    })
    
    progress.value = 66
    statusMessage.value = '✅ Bước 2/3 hoàn thành: Đã ghép video thành công'
    
    // Step 3: Mix audio
    progress.value = 70
    statusMessage.value = '📝 Bước 3/3: Đang mix audio giữa MP3 và video...'
    
    const step3Id = await callCommand('mix_video_audio_step3', {
      mergedMp3Path: mergedMp3Path,
      mergedVideoPath: mergedVideoPath,
      outputFolder: outputFolderPath,
      outputFilename: outputFilename.value.trim() || 'output_video.mp4'
    })
    
    currentProcessId.value = step3Id
    const step3Result = await callCommand('wait_for_video_creation', {
      processId: step3Id
    })
    
    if (step3Result.includes('Lỗi')) {
      throw new Error(step3Result)
    }
    
    currentProcessId.value = null
    progress.value = 100
    statusMessage.value = '✅ ' + step3Result
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

const stopAudioCreation = async () => {
  try {
    // Dừng tất cả các process đang chạy và giải phóng bộ nhớ
    const result = await callCommand('stop_all_video_creation', {})
    
    // Reset tất cả state
    currentProcessId.value = null
    isCreating.value = false
    progress.value = 0
    statusMessage.value = '⚠️ ' + result
    statusType.value = 'info'
  } catch (error) {
    // Ngay cả khi có lỗi, vẫn reset state để tránh treo
    currentProcessId.value = null
    isCreating.value = false
    progress.value = 0
    statusMessage.value = '⚠️ Đã dừng quá trình tạo video (có thể một số process đã kết thúc)'
    statusType.value = 'info'
  }
}
</script>

