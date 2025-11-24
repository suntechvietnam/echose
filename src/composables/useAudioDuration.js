import { ref, computed } from 'vue'

/**
 * Composable để đọc duration của file audio bằng HTML5 Audio API
 * Hỗ trợ các format: MP3, WAV, M4A, AAC, OGG
 */
export function useAudioDuration() {
  const fileDurations = ref({})
  const loadingDurations = ref(new Set())

  /**
   * Xác định MIME type dựa trên extension của file
   */
  const getMimeType = (filePath) => {
    const ext = filePath.toLowerCase().split('.').pop()
    const mimeTypes = {
      'mp3': 'audio/mpeg',
      'wav': 'audio/wav',
      'm4a': 'audio/mp4',
      'aac': 'audio/aac',
      'ogg': 'audio/ogg',
      'flac': 'audio/flac',
      'webm': 'audio/webm'
    }
    return mimeTypes[ext] || 'audio/mpeg'
  }

  /**
   * Đọc duration của file audio bằng HTML5 Audio API
   * @param {string} filePath - Đường dẫn đến file audio
   * @returns {Promise<number|null>} - Duration tính bằng giây, hoặc null nếu lỗi
   */
  const loadFileDuration = async (filePath) => {
    // Tránh load cùng một file nhiều lần
    if (loadingDurations.value.has(filePath)) {
      return fileDurations.value[filePath] || null
    }

    // Nếu đã có duration, return luôn
    if (fileDurations.value[filePath]) {
      return fileDurations.value[filePath]
    }

    loadingDurations.value.add(filePath)

    try {
      // Ưu tiên dùng Rust backend với ffprobe (nhanh hơn nhiều)
      // Fallback về JavaScript nếu Rust backend không có
      try {
        const { invoke } = await import('@tauri-apps/api/tauri')
        const duration = await invoke('get_audio_duration', { filePath })
        
        if (duration && !isNaN(duration) && duration > 0 && isFinite(duration)) {
          fileDurations.value[filePath] = duration
          return duration
        }
      } catch (rustError) {
        console.warn('Rust backend không khả dụng, dùng JavaScript fallback:', rustError)
      }
      
      // Fallback: Dùng JavaScript HTML5 Audio API (chậm hơn với file lớn)
      const { readBinaryFile } = await import('@tauri-apps/api/fs')
      const fileData = await readBinaryFile(filePath)
      
      // Xác định MIME type
      const mimeType = getMimeType(filePath)
      
      // Tạo Blob từ binary data
      const blob = new Blob([fileData], { type: mimeType })
      const blobUrl = URL.createObjectURL(blob)
      
      // Tạo Audio element để đọc metadata
      const audio = new Audio(blobUrl)
      
      // Đợi metadata được load
      await new Promise((resolve, reject) => {
        const cleanup = () => {
          audio.removeEventListener('loadedmetadata', onLoaded)
          audio.removeEventListener('error', onError)
        }

        const onLoaded = () => {
          cleanup()
          resolve()
        }

        const onError = (e) => {
          cleanup()
          reject(new Error('Không thể đọc file audio'))
        }

        audio.addEventListener('loadedmetadata', onLoaded)
        audio.addEventListener('error', onError)
        
        // Timeout sau 10 giây
        const timeout = setTimeout(() => {
          cleanup()
          reject(new Error('Timeout khi đọc metadata'))
        }, 10000)

        // Clear timeout khi resolve
        audio.addEventListener('loadedmetadata', () => {
          clearTimeout(timeout)
        }, { once: true })
      })
      
      // Lấy duration (tính bằng giây)
      const duration = audio.duration
      
      // Cleanup blob URL
      URL.revokeObjectURL(blobUrl)
      
      if (isNaN(duration) || duration === 0 || !isFinite(duration)) {
        throw new Error('Duration không hợp lệ')
      }
      
      fileDurations.value[filePath] = duration
      return duration
    } catch (error) {
      console.error(`Lỗi khi lấy duration cho ${filePath}:`, error)
      fileDurations.value[filePath] = null
      return null
    } finally {
      loadingDurations.value.delete(filePath)
    }
  }

  /**
   * Load duration cho nhiều files cùng lúc
   * @param {string[]} filePaths - Mảng đường dẫn đến các file audio
   */
  const loadMultipleDurations = async (filePaths) => {
    const promises = filePaths.map(filePath => loadFileDuration(filePath))
    await Promise.allSettled(promises)
  }

  /**
   * Xóa duration của một file
   */
  const removeDuration = (filePath) => {
    if (fileDurations.value[filePath]) {
      delete fileDurations.value[filePath]
    }
  }

  /**
   * Xóa tất cả durations
   */
  const clearAllDurations = () => {
    fileDurations.value = {}
  }

  /**
   * Lấy duration của một file (nếu đã load)
   */
  const getDuration = (filePath) => {
    return fileDurations.value[filePath] || null
  }

  /**
   * Kiểm tra xem file đang được load duration không
   */
  const isLoading = (filePath) => {
    return loadingDurations.value.has(filePath)
  }

  /**
   * Tính tổng duration của nhiều files
   * @param {Ref<string[]>|ComputedRef<string[]>|string[]} filePathsRef - Ref, ComputedRef hoặc mảng đường dẫn đến các file audio
   * @returns {ComputedRef<number>} - Computed tổng duration tính bằng giây (tự động update khi filePaths hoặc durations thay đổi)
   */
  const createTotalDuration = (filePathsRef) => {
    return computed(() => {
      // Hỗ trợ cả ref/computed và array thường
      const filePaths = filePathsRef?.value ?? filePathsRef ?? []
      
      if (!Array.isArray(filePaths)) {
        return 0
      }
      
      return filePaths.reduce((total, filePath) => {
        const duration = fileDurations.value[filePath]
        return total + (duration && !isNaN(duration) && isFinite(duration) ? duration : 0)
      }, 0)
    })
  }

  return {
    fileDurations,
    loadFileDuration,
    loadMultipleDurations,
    removeDuration,
    clearAllDurations,
    getDuration,
    isLoading,
    createTotalDuration
  }
}

