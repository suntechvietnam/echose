<template>
  <aside class="sidebar">
    <div class="sidebar-header">
      <div class="app-title">
        <img :src="logo" alt="logo" class="logo" width="165" />
      </div>
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
            :class="{ active: currentPage === 'extract-image-from-video' }"
            @click.prevent="handleClick('extract-image-from-video')"
          >
            <span class="menu-icon">📸</span>
            <span class="menu-text">Tách ảnh từ video</span>
          </a>
        </li>
        <li>
          <a 
            href="#" 
            class="menu-item" 
            :class="{ active: currentPage === 'video-to-video' }"
            @click.prevent="handleClick('video-to-video')"
          >
            <span class="menu-icon">🎞️</span>
            <span class="menu-text">Ghép video</span>
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
        <li>
          <a 
            href="#" 
            class="menu-item" 
            :class="{ active: currentPage === 'voice-library' }"
            @click.prevent="handleClick('voice-library')"
          >
            <span class="menu-icon">🎙️</span>
            <span class="menu-text">Thư viện giọng nói</span>
          </a>
        </li>
      </ul>
    </nav>

    <!-- Profile -->
    <div class="sidebar-profile">
      <div class="profile-avatar">
        {{ userInitial }}
      </div>
      <div class="profile-info">
        <div class="profile-name">
          {{ user.name }}
        </div>
        <div class="profile-plan" :class="user.plan">
          {{ user.plan === 'pro' ? 'Pro' : 'Free' }}
        </div>
      </div>
    </div>
  </aside>
</template>

<script setup>
import { computed } from 'vue'
import logo from '@/assets/img/logo.png'

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

// Tạm thời mock thông tin user
const user = {
  name: 'Guest',
  plan: 'Free' // 'pro' hoặc 'free'
}

const userInitial = computed(() => user.name.charAt(0).toUpperCase())
</script>

