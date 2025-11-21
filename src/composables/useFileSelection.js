import { ref } from 'vue'
import { useTauri } from './useTauri'

export function useFileSelection() {
  const { callCommand } = useTauri()
  const selectedFiles = ref([])

  const selectFiles = async (fileTypes = [], multiple = true) => {
    try {
      const selected = await callCommand('select_files', {
        fileTypes,
        multiple
      })
      
      if (selected && selected.length > 0) {
        selectedFiles.value = Array.isArray(selected) ? selected : [selected]
        return selectedFiles.value
      }
      return []
    } catch (error) {
      console.error('Error selecting files:', error)
      throw error
    }
  }

  const selectFolder = async () => {
    try {
      const selected = await callCommand('select_folder')
      return selected || null
    } catch (error) {
      console.error('Error selecting folder:', error)
      throw error
    }
  }

  const removeFile = (index) => {
    selectedFiles.value.splice(index, 1)
  }

  const clearFiles = () => {
    selectedFiles.value = []
  }

  return {
    selectedFiles,
    selectFiles,
    selectFolder,
    removeFile,
    clearFiles
  }
}

