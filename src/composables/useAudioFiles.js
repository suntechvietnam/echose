import { ref, computed } from 'vue'
import { useTauri } from './useTauri'
import { useFileSelection } from './useFileSelection'

export function useAudioFiles() {
  const { callCommand } = useTauri()
  const { selectedFiles: audioFiles, selectFiles, removeFile: baseRemoveFile } = useFileSelection()
  const fileDurations = ref({})
  const statusMessage = ref('')
  const statusType = ref('info')

  const getFileName = (filePath) => {
    return filePath.split('/').pop() || filePath.split('\\').pop() || filePath
  }

  const formatDuration = (seconds) => {
    if (!seconds || isNaN(seconds)) return '0 phút 0 giây'
    
    const totalMins = Math.floor(seconds / 60)
    const secs = Math.floor(seconds % 60)
    
    if (totalMins >= 60) {
      // Hiển thị theo giờ và phút
      const hours = Math.floor(totalMins / 60)
      const mins = totalMins % 60
      
      if (mins === 0) {
        return `${hours} giờ`
      } else {
        return `${hours} giờ ${mins} phút`
      }
    } else {
      // Hiển thị theo phút và giây
      if (secs === 0) {
        return `${totalMins} phút`
      } else {
        return `${totalMins} phút ${secs} giây`
      }
    }
  }

  const totalDuration = computed(() => {
    return audioFiles.value.reduce((total, file) => {
      const duration = fileDurations.value[file]
      return total + (duration && !isNaN(duration) ? duration : 0)
    }, 0)
  })

  const loadFileDuration = async (filePath) => {
    try {
      const duration = await callCommand('get_audio_duration', { filePath })
      fileDurations.value[filePath] = duration
    } catch (error) {
      console.error(`Lỗi khi lấy duration cho ${filePath}:`, error)
      fileDurations.value[filePath] = null
    }
  }

  const removeFile = (index) => {
    const fileToRemove = audioFiles.value[index]
    baseRemoveFile(index)
    // Xóa duration khi xóa file
    if (fileToRemove && fileDurations.value[fileToRemove]) {
      delete fileDurations.value[fileToRemove]
    }
  }

  const shuffleFiles = () => {
    const shuffled = [...audioFiles.value]
    // Fisher-Yates shuffle algorithm
    for (let i = shuffled.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [shuffled[i], shuffled[j]] = [shuffled[j], shuffled[i]]
    }
    audioFiles.value = shuffled
  }

  const selectAudioFiles = async () => {
    try {
      await selectFiles(['mp3'], true)
      statusMessage.value = `✅ Đã chọn ${audioFiles.value.length} file MP3`
      statusType.value = 'success'
      
      // Load duration cho các file mới được chọn
      for (const file of audioFiles.value) {
        if (!fileDurations.value[file]) {
          loadFileDuration(file)
        }
      }
    } catch (error) {
      statusMessage.value = 'Lỗi khi chọn file MP3: ' + error
      statusType.value = 'error'
    }
  }

  return {
    audioFiles,
    fileDurations,
    getFileName,
    formatDuration,
    totalDuration,
    removeFile,
    shuffleFiles,
    selectAudioFiles
  }
}

