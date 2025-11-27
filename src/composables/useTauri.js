import { invoke } from '@tauri-apps/api/core'

export function useTauri() {
  const callCommand = async (cmd, args = {}) => {
    try {
      return await invoke(cmd, args)
    } catch (error) {
      console.error(`Error calling ${cmd}:`, error)
      throw error
    }
  }

  return {
    callCommand
  }
}

