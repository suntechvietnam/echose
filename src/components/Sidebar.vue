<template>
  <aside class="sidebar">
    <div class="sidebar-header">
      <h1 class="app-title">YtbFlow</h1>
      <p class="app-subtitle">Quản lý video</p>
    </div>
    
    <nav class="sidebar-nav">
      <ul class="menu-list">
        <li>
          <a 
            href="#" 
            class="menu-item" 
            :class="{ active: currentPage === 'download' }"
            @click.prevent="handleClick('download')"
          >
            <span class="menu-icon">⬇️</span>
            <span class="menu-text">Download Video</span>
          </a>
        </li>
        <li>
          <a 
            href="#" 
            class="menu-item" 
            :class="{ active: currentPage === 'audio' }"
            @click.prevent="handleClick('audio')"
          >
            <span class="menu-icon">🎵</span>
            <span class="menu-text">Quản Lý File Nhạc Nguồn</span>
          </a>
        </li>
        <li>
          <a 
            href="#" 
            class="menu-item" 
            :class="{ active: currentPage === 'videos' }"
            @click.prevent="handleClick('videos')"
          >
            <span class="menu-icon">🎬</span>
            <span class="menu-text">Quản Lý Video Nguồn</span>
          </a>
        </li>
        <li>
          <a 
            href="#" 
            class="menu-item" 
            :class="{ active: currentPage === 'create' }"
            @click.prevent="handleClick('create')"
          >
            <span class="menu-icon">✨</span>
            <span class="menu-text">Tạo Video</span>
          </a>
        </li>
        <li>
          <a 
            href="#" 
            class="menu-item" 
            :class="{ active: currentPage === 'image-to-video' }"
            @click.prevent="handleClick('image-to-video')"
          >
            <span class="menu-icon">🖼️</span>
            <span class="menu-text">Tạo Video Nguồn</span>
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

