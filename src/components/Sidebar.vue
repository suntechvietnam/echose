<template>
  <aside class="sidebar">
    <div class="sidebar-header">
      <div class="app-title">
        <img :src="logo" alt="logo" class="logo" width="165" />
      </div>
    </div>
    
    <nav class="sidebar-nav">
      <ul class="menu-list">
        <!-- Video Menu Group -->
        <li class="menu-group">
          <div class="menu-group-header" @click="toggleGroup('video')">
            <span class="menu-icon">🎬</span>
            <span class="menu-text">Video</span>
            <span class="menu-arrow" :class="{ expanded: expandedGroups.video }">▼</span>
          </div>
          <ul class="submenu" v-show="expandedGroups.video">
            <li>
              <a 
                href="#" 
                class="menu-item submenu-item" 
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
                class="menu-item submenu-item" 
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
                class="menu-item submenu-item" 
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
                class="menu-item submenu-item" 
                :class="{ active: currentPage === 'video-audio-merge' }"
                @click.prevent="handleClick('video-audio-merge')"
              >
                <span class="menu-icon">🎵</span>
                <span class="menu-text">Ghép video với audio</span>
              </a>
            </li>
            <li>
              <a 
                href="#" 
                class="menu-item submenu-item" 
                :class="{ active: currentPage === 'extract-audio' }"
                @click.prevent="handleClick('extract-audio')"
              >
                <span class="menu-icon">🎙️</span>
                <span class="menu-text">Tách audio từ video</span>
              </a>
            </li>
          </ul>
        </li>

        <!-- Audio Menu Group -->
        <li class="menu-group">
          <div class="menu-group-header" @click="toggleGroup('audio')">
            <span class="menu-icon">🎵</span>
            <span class="menu-text">Audio</span>
            <span class="menu-arrow" :class="{ expanded: expandedGroups.audio }">▼</span>
          </div>
          <ul class="submenu" v-show="expandedGroups.audio">
            <li>
              <a 
                href="#" 
                class="menu-item submenu-item" 
                :class="{ active: currentPage === 'audio-to-text' }"
                @click.prevent="handleClick('audio-to-text')"
              >
                <span class="menu-icon">📝</span>
                <span class="menu-text">Audio/Video to Text</span>
              </a>
            </li>
            <li>
              <a 
                href="#" 
                class="menu-item submenu-item" 
                :class="{ active: currentPage === 'voice-library' }"
                @click.prevent="handleClick('voice-library')"
              >
                <span class="menu-icon">🎙️</span>
                <span class="menu-text">Thư viện Voice</span>
              </a>
            </li>
            <li>
              <a 
                href="#" 
                class="menu-item submenu-item" 
                :class="{ active: currentPage === 'text-to-audio' }"
                @click.prevent="handleClick('text-to-audio')"
              >
                <span class="menu-icon">🔊</span>
                <span class="menu-text">Convert Text to Audio</span>
              </a>
            </li>
          </ul>
        </li>

        <!-- Download Menu Group -->
        <li class="menu-group">
          <div class="menu-group-header" @click="toggleGroup('download')">
            <span class="menu-icon">⬇️</span>
            <span class="menu-text">Download</span>
            <span class="menu-arrow" :class="{ expanded: expandedGroups.download }">▼</span>
          </div>
          <ul class="submenu" v-show="expandedGroups.download">
            <li>
              <a 
                href="#" 
                class="menu-item submenu-item" 
                :class="{ active: currentPage === 'download-youtube' }"
                @click.prevent="handleClick('download-youtube')"
              >
                <span class="menu-icon">📺</span>
                <span class="menu-text">YouTube</span>
              </a>
            </li>
            <li>
              <a 
                href="#" 
                class="menu-item submenu-item" 
                :class="{ active: currentPage === 'download-tiktok' }"
                @click.prevent="handleClick('download-tiktok')"
              >
                <span class="menu-icon">🎵</span>
                <span class="menu-text">TikTok</span>
              </a>
            </li>
            <li>
              <a 
                href="#" 
                class="menu-item submenu-item" 
                :class="{ active: currentPage === 'download-facebook' }"
                @click.prevent="handleClick('download-facebook')"
              >
                <span class="menu-icon">👥</span>
                <span class="menu-text">Facebook</span>
              </a>
            </li>
            <li>
              <a 
                href="#" 
                class="menu-item submenu-item" 
                :class="{ active: currentPage === 'download-instagram' }"
                @click.prevent="handleClick('download-instagram')"
              >
                <span class="menu-icon">📷</span>
                <span class="menu-text">Instagram</span>
              </a>
            </li>
            <li>
              <a 
                href="#" 
                class="menu-item submenu-item" 
                :class="{ active: currentPage === 'download-x' }"
                @click.prevent="handleClick('download-x')"
              >
                <span class="menu-icon">𝕏</span>
                <span class="menu-text">X (Twitter)</span>
              </a>
            </li>
          </ul>
        </li>

        <!-- AI Menu Group -->
        <li class="menu-group">
          <div class="menu-group-header" @click="toggleGroup('ai')">
            <span class="menu-icon">🤖</span>
            <span class="menu-text">AI Tools</span>
            <span class="menu-arrow" :class="{ expanded: expandedGroups.ai }">▼</span>
          </div>
          <ul class="submenu" v-show="expandedGroups.ai">
            <li>
              <a 
                href="#" 
                class="menu-item submenu-item" 
                :class="{ active: currentPage === 'ai-image' }"
                @click.prevent="handleClick('ai-image')"
              >
                <span class="menu-icon">🎨</span>
                <span class="menu-text">AI Image</span>
              </a>
            </li>
            <li>
              <a 
                href="#" 
                class="menu-item submenu-item" 
                :class="{ active: currentPage === 'ai-video' }"
                @click.prevent="handleClick('ai-video')"
              >
                <span class="menu-icon">🎬</span>
                <span class="menu-text">AI Video</span>
              </a>
            </li>
          </ul>
        </li>

        <!-- Reup Menu Group -->
        <li class="menu-group">
          <div class="menu-group-header" @click="toggleGroup('reup')">
            <span class="menu-icon">🔄</span>
            <span class="menu-text">Reup Tools</span>
            <span class="menu-arrow" :class="{ expanded: expandedGroups.reup }">▼</span>
          </div>
          <ul class="submenu" v-show="expandedGroups.reup">
            <li>
              <a 
                href="#" 
                class="menu-item submenu-item" 
                :class="{ active: currentPage === 'reup-youtube' }"
                @click.prevent="handleClick('reup-youtube')"
              >
                <span class="menu-icon">📺</span>
                <span class="menu-text">YouTube Reup</span>
              </a>
            </li>
            <li>
              <a 
                href="#" 
                class="menu-item submenu-item" 
                :class="{ active: currentPage === 'reup-tiktok' }"
                @click.prevent="handleClick('reup-tiktok')"
              >
                <span class="menu-icon">🎵</span>
                <span class="menu-text">TikTok Reup</span>
              </a>
            </li>
          </ul>
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
import { ref, computed } from 'vue'
import logo from '@/assets/img/logo.png'

const props = defineProps({
  currentPage: {
    type: String,
    required: true
  }
})

const emit = defineEmits(['navigate'])

// Track expanded menu groups
const expandedGroups = ref({
  video: false,
  audio: false,
  download: false,
  ai: false,
  reup: false
})

const toggleGroup = (groupName) => {
  expandedGroups.value[groupName] = !expandedGroups.value[groupName]
}

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

<style scoped>
.menu-group {
  margin-bottom: 8px;
}

.menu-group-header {
  display: flex;
  align-items: center;
  padding: 12px 20px;
  cursor: pointer;
  transition: all 0.2s ease;
  border-radius: 8px;
  margin: 0 12px;
}

.menu-group-header:hover {
  background: rgba(255, 255, 255, 0.05);
}

.menu-arrow {
  margin-left: auto;
  font-size: 10px;
  transition: transform 0.2s ease;
  opacity: 0.6;
}

.menu-arrow.expanded {
  transform: rotate(180deg);
}

.submenu {
  list-style: none;
  padding: 0;
  margin: 4px 0 0 0;
}

.submenu-item {
  padding-left: 52px !important;
  font-size: 14px;
}

.submenu-item .menu-icon {
  font-size: 16px;
}
</style>
