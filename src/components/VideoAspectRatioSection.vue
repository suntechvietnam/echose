<template>
  <div class="file-group video-size-group video-size-inline">
    <h3 class="form-label video-size-label">Chọn loại video</h3>
    <div class="video-aspect-tabs">
      <button 
        :class="['aspect-tab', 'aspect-tab--compact', { active: modelValue === '16:9' }]"
        @click="$emit('update:modelValue', '16:9')"
      >
        Video dài (16:9)
      </button>
      <button 
        :class="['aspect-tab', 'aspect-tab--compact', { active: modelValue === '9:16' }]"
        @click="$emit('update:modelValue', '9:16')"
      >
        Video ngắn (9:16)
      </button>
    </div>
  </div>
</template>

<script setup>
const props = defineProps({
  modelValue: {
    type: String,
    required: true
  }
})
</script>

<style scoped>
.video-size-inline {
  display: flex;
  align-items: center;
  gap: 12px;
}

.video-size-label {
  margin: 0;
  white-space: nowrap;
}

.video-aspect-tabs {
  flex: 1;
}

.aspect-tab--compact {
  padding: 4px 10px;
  font-size: 13px;
}
</style>
