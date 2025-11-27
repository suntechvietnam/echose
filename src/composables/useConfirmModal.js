import { ref } from 'vue'

export function useConfirmModal() {
  const modalRef = ref(null)
  
  const showConfirm = ({
    title = 'Xác nhận',
    message = 'Bạn có chắc chắn muốn thực hiện hành động này?',
    type = 'warning',
    confirmText = 'Xác nhận',
    cancelText = 'Hủy',
    closeOnOverlay = false
  } = {}) => {
    return new Promise((resolve) => {
      if (!modalRef.value) {
        resolve(false)
        return
      }

      const modal = modalRef.value

      // Setup event handlers
      const handleConfirm = () => {
        modal.setLoading(true)
        // Simulate async operation
        setTimeout(() => {
          modal.hide()
          resolve(true)
          cleanup()
        }, 100)
      }

      const handleCancel = () => {
        resolve(false)
        cleanup()
      }

      const cleanup = () => {
        modal.$off('confirm', handleConfirm)
        modal.$off('cancel', handleCancel)
      }

      // Add event listeners
      modal.$on('confirm', handleConfirm)
      modal.$on('cancel', handleCancel)

      // Show modal
      modal.show()
    })
  }

  return {
    modalRef,
    showConfirm
  }
}