import { ref, computed } from 'vue'
import { useTauri } from './useTauri'

export function useVideoFiles() {
  const { callCommand } = useTauri()
  const videoFiles = ref([])
  const videoDurations = ref({})
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

  const totalVideoDuration = computed(() => {
    return videoFiles.value.reduce((total, file) => {
      const duration = videoDurations.value[file]
      return total + (duration && !isNaN(duration) ? duration : 0)
    }, 0)
  })

  const loadVideoDuration = async (filePath) => {
    try {
      // Sử dụng cùng command vì ffprobe có thể xử lý cả video
      const duration = await callCommand('get_audio_duration', { filePath })
      videoDurations.value[filePath] = duration
    } catch (error) {
      console.error(`Lỗi khi lấy duration cho video ${filePath}:`, error)
      videoDurations.value[filePath] = null
    }
  }

  const removeVideoFile = (index) => {
    const fileToRemove = videoFiles.value[index]
    videoFiles.value.splice(index, 1)
    // Xóa duration khi xóa file
    if (fileToRemove && videoDurations.value[fileToRemove]) {
      delete videoDurations.value[fileToRemove]
    }
  }

  const shuffleVideoFiles = () => {
    const shuffled = [...videoFiles.value]
    // Fisher-Yates shuffle algorithm
    for (let i = shuffled.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [shuffled[i], shuffled[j]] = [shuffled[j], shuffled[i]]
    }
    videoFiles.value = shuffled
  }

  const selectVideoFiles = async () => {
    try {
      const selected = await callCommand('select_files', {
        fileTypes: ['mp4'],
        multiple: true
      })
      
      if (selected && selected.length > 0) {
        const newFiles = Array.isArray(selected) ? selected : [selected]
        // Thêm các file mới vào danh sách (tránh trùng lặp)
        for (const file of newFiles) {
          if (!videoFiles.value.includes(file)) {
            videoFiles.value.push(file)
          }
        }
        statusMessage.value = `✅ Đã chọn ${videoFiles.value.length} file MP4`
        statusType.value = 'success'
        
        // Load duration cho các file mới được chọn
        for (const file of newFiles) {
          if (!videoDurations.value[file]) {
            loadVideoDuration(file)
          }
        }
      }
    } catch (error) {
      statusMessage.value = 'Lỗi khi chọn file MP4: ' + error
      statusType.value = 'error'
    }
  }

  return {
    videoFiles,
    videoDurations,
    getFileName,
    formatDuration,
    totalVideoDuration,
    removeVideoFile,
    shuffleVideoFiles,
    selectVideoFiles
  }
}

