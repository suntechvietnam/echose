<template>
  <div class="form-group video-effect-group">
    <label class="form-label">✨ Hiệu ứng video</label>
    <div class="effect-preview-grid">
      <div 
        v-for="effect in videoEffectOptions" 
        :key="effect.value"
        :class="['effect-preview-item', { active: selectedEffect === effect.value }]"
        @click="selectEffect(effect.value)"
        @mouseenter="handleHover(effect.value, true)"
        @mouseleave="handleHover(effect.value, false)"
        :title="effect.name"
      >
        <!-- Show GIF only when hovered or active -->
        <img 
          v-if="effect.preview && (hoveredEffect === effect.value || selectedEffect === effect.value)"
          :src="effect.preview" 
          :alt="effect.name"
          class="effect-preview-image"
          @error="handleImageError"
          loading="lazy"
        />
        <!-- Show text placeholder when not hovered and not active -->
        <div 
          v-else
          class="effect-preview-placeholder"
        >
          {{ getEffectDisplayName(effect.value) }}
        </div>
        <div v-if="selectedEffect === effect.value" class="effect-check-icon">
          ✓
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
// Import effect images
import noneImg from '../assets/img/effects/none.png'
import circleopenImg from '../assets/img/effects/circleopen.gif'
import diagbrImg from '../assets/img/effects/diagbr.gif'
import diagtlImg from '../assets/img/effects/diagtl.gif'
import diagtrImg from '../assets/img/effects/diagtr.gif'
import dissolveImg from '../assets/img/effects/dissolve.gif'
import hlsliceImg from '../assets/img/effects/hlslice.gif'
import hlwindImg from '../assets/img/effects/hlwind.gif'
import hrsliceImg from '../assets/img/effects/hrslice.gif'
import hrwindImg from '../assets/img/effects/hrwind.gif'
import radialImg from '../assets/img/effects/radial.gif'
import vdsliceImg from '../assets/img/effects/vdslice.gif'
import vdwindImg from '../assets/img/effects/vdwind.gif'
import vusliceImg from '../assets/img/effects/vuslice.gif'
import vuwindImg from '../assets/img/effects/vuwind.gif'

// Props and Emits
const props = defineProps({
  selectedEffect: {
    type: String,
    default: 'diagtl'
  }
})

const emit = defineEmits(['update:selectedEffect'])

// Hover state
import { ref } from 'vue'
const hoveredEffect = ref(null)

// Video effect options
const videoEffectOptions = [
  { name: 'None', value: 'none', preview: noneImg },
  { name: 'Circle Open', value: 'circleopen', preview: circleopenImg },
  { name: 'Diagonal Bottom Right', value: 'diagbr', preview: diagbrImg },
  { name: 'Diagonal Top Left', value: 'diagtl', preview: diagtlImg },
  { name: 'Diagonal Top Right', value: 'diagtr', preview: diagtrImg },
  { name: 'Dissolve', value: 'dissolve', preview: dissolveImg },
  { name: 'Horizontal Left Slice', value: 'hlslice', preview: hlsliceImg },
  { name: 'Horizontal Left Wind', value: 'hlwind', preview: hlwindImg },
  { name: 'Horizontal Right Slice', value: 'hrslice', preview: hrsliceImg },
  { name: 'Horizontal Right Wind', value: 'hrwind', preview: hrwindImg },
  { name: 'Radial', value: 'radial', preview: radialImg },
  { name: 'Vertical Down Slice', value: 'vdslice', preview: vdsliceImg },
  { name: 'Vertical Down Wind', value: 'vdwind', preview: vdwindImg },
  { name: 'Vertical Up Slice', value: 'vuslice', preview: vusliceImg },
  { name: 'Vertical Up Wind', value: 'vuwind', preview: vuwindImg },
]

// Methods
const selectEffect = (effectValue) => {
  emit('update:selectedEffect', effectValue)
}

const handleHover = (effectValue, isHovered) => {
  hoveredEffect.value = isHovered ? effectValue : null
}

const getEffectDisplayName = (effectValue) => {
  // Just return the effect value in lowercase, keeping original name
  return effectValue.toLowerCase()
}

const handleImageError = (event) => {
  console.warn('Failed to load effect preview image:', event.target.src)
  event.target.style.display = 'none'
}
</script>

<style scoped>
/* Video effect group */
.video-effect-group {
  margin-top: 15px;
}

/* Effect preview grid */
.effect-preview-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(60px, 1fr));
  gap: 12px;
  margin-top: 12px;
  padding-top: 5px;
}

.effect-preview-item {
  position: relative;
  display: flex;
  flex-direction: column;
  align-items: center;
  border-radius: 5px;
  cursor: pointer;
  overflow: hidden;
}

.effect-preview-image {
  width: 100%;
  height: 80px;
  object-fit: cover;
  border-radius: 4px;
  background: #f8fafc;
  padding: 0;
}

.effect-preview-placeholder {
  width: 100%;
  height: 80px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #f1f5f9 0%, #e2e8f0 100%);
  border: 1px solid #cbd5e1;
  border-radius: 4px;
  font-size: 10px;
  font-weight: 600;
  color: #64748b;
  text-align: center;
  line-height: 1.2;
  letter-spacing: 0.5px;
  transition: all 0.2s ease;
}

.effect-preview-item:hover .effect-preview-placeholder {
  background: linear-gradient(135deg, #ddd6fe 0%, #c7d2fe 100%);
  border-color: #667eea;
  color: #5b21b6;
}

.effect-preview-item.active .effect-preview-placeholder {
  background: linear-gradient(135deg, #667eea 0%, #5568d3 100%);
  border-color: #4f46e5;
  color: white;
}

.effect-preview-fallback {
  width: 100%;
  height: 80px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #f8fafc 0%, #e2e8f0 100%);
  border-radius: 4px;
  margin-bottom: 6px;
  font-size: 16px;
  font-weight: 600;
  color: #64748b;
  text-align: center;
  border: 1px solid #e2e8f0;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.effect-preview-label {
  font-size: 11px;
  color: #64748b;
  text-align: center;
  font-weight: 500;
  line-height: 1.3;
  word-break: break-word;
  width: 100%;
}

.effect-preview-item.active .effect-preview-label {
  color: #667eea;
  font-weight: 600;
}

.effect-check-icon {
  position: absolute;
  top: 4px;
  right: 4px;
  width: 20px;
  height: 20px;
  background: #667eea;
  color: white;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  font-weight: bold;
  z-index: 10;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
}
</style>