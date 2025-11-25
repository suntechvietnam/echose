<template>
  <aside class="sidebar">
    <div class="sidebar-header">
      <h1 class="app-title">YtbFlow</h1>
      <p class="app-subtitle">You're Guest</p>
    </div>
    
    <nav class="sidebar-nav">
      <ul class="menu-list">
        <li>
          <a 
            href="#" 
            class="menu-item" 
            :class="{ active: currentPage === 'image-to-video' }"
            @click.prevent="handleClick('image-to-video')"
          >
            <span class="menu-icon">🖼️</span>
            <span class="menu-text">Tạo video từ ảnh</span>
          </a>
        </li>
        <li>
          <a 
            href="#" 
            class="menu-item" 
            :class="{ active: currentPage === 'audio-to-text' }"
            @click.prevent="handleClick('audio-to-text')"
          >
            <span class="menu-icon">📝</span>
            <span class="menu-text">Audio / Video to text</span>
          </a>
        </li>
      </ul>
    </nav>
  </aside>
</template>

<script setup>
const props = defineProps({
  currentPage: {
    type: String,
    required: true
  }
})

const emit = defineEmits(['navigate'])

const handleClick = (page) => {
  emit('navigate', page)
}
</script>

