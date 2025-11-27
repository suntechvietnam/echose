<template>
  <div class="modal-overlay" @click="handleOverlayClick">
    <div class="modal-container" @click.stop>
      <div class="modal-header">
        <div class="modal-icon">
          <span v-if="type === 'warning'">⚠️</span>
          <span v-else-if="type === 'error'">❌</span>
          <span v-else-if="type === 'success'">✅</span>
          <span v-else-if="type === 'info'">ℹ️</span>
          <span v-else>❓</span>
        </div>
        <h3 class="modal-title">{{ title }}</h3>
      </div>
      
      <div class="modal-body">
        <p class="modal-message">{{ message }}</p>
      </div>
      
      <div class="modal-footer">
        <button 
          class="btn btn-cancel" 
          @click="handleCancel"
          :disabled="loading"
        >
          {{ cancelText }}
        </button>
        <button 
          class="btn btn-confirm" 
          @click="handleConfirm"
          :disabled="loading"
          :class="[`btn-${type}`]"
        >
          {{ loading ? 'Đang xử lý...' : confirmText }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue'

const props = defineProps({
  title: {
    type: String,
    default: 'Xác nhận'
  },
  message: {
    type: String,
    required: true
  },
  type: {
    type: String,
    default: 'warning', // 'warning', 'error', 'success', 'info', 'question'
    validator: (value) => ['warning', 'error', 'success', 'info', 'question'].includes(value)
  },
  confirmText: {
    type: String,
    default: 'Xác nhận'
  },
  cancelText: {
    type: String,
    default: 'Hủy'
  },
  closeOnOverlay: {
    type: Boolean,
    default: false
  }
})

const emit = defineEmits(['confirm', 'cancel', 'close'])

const loading = ref(false)

const handleConfirm = () => {
  loading.value = true
  emit('confirm')
}

const handleCancel = () => {
  emit('cancel')
}

const handleOverlayClick = () => {
  if (props.closeOnOverlay) {
    emit('cancel')
  }
}


</script>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  backdrop-filter: blur(2px);
}

.modal-container {
  background: white;
  border-radius: 12px;
  padding: 0;
  min-width: 400px;
  max-width: 500px;
  box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04);
  animation: modalSlideIn 0.3s ease-out;
}

@keyframes modalSlideIn {
  from {
    opacity: 0;
    transform: scale(0.9) translateY(-10px);
  }
  to {
    opacity: 1;
    transform: scale(1) translateY(0);
  }
}

.modal-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 20px 24px 16px;
  border-bottom: 1px solid #e5e7eb;
}

.modal-icon {
  font-size: 24px;
  line-height: 1;
}

.modal-title {
  font-size: 18px;
  font-weight: 600;
  color: #1f2937;
  margin: 0;
}

.modal-body {
  padding: 16px 24px;
}

.modal-message {
  font-size: 14px;
  line-height: 1.5;
  color: #4b5563;
  margin: 0;
}

.modal-footer {
  display: flex;
  gap: 12px;
  padding: 16px 24px 20px;
  justify-content: flex-end;
}

.btn {
  padding: 8px 16px;
  border: none;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  min-width: 80px;
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.btn-cancel {
  background: #f3f4f6;
  color: #374151;
}

.btn-cancel:hover:not(:disabled) {
  background: #e5e7eb;
}

.btn-confirm {
  color: white;
}

.btn-warning {
  background: #ef4444;
}

.btn-warning:hover:not(:disabled) {
  background: #ef4444;
}

.btn-error {
  background: #ef4444;
}

.btn-error:hover:not(:disabled) {
  background: #dc2626;
}

.btn-success {
  background: #10b981;
}

.btn-success:hover:not(:disabled) {
  background: #059669;
}

.btn-info {
  background: #3b82f6;
}

.btn-info:hover:not(:disabled) {
  background: #2563eb;
}

.btn-question {
  background: #6366f1;
}

.btn-question:hover:not(:disabled) {
  background: #4f46e5;
}
</style>