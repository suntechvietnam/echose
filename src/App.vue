<template>
  <div class="app-container">
    <Sidebar :current-page="currentPage" @navigate="handleNavigate" />
    <main class="main-content">
      <ImageToVideoPage v-show="currentPage === 'image-to-video'" />
      <VideoToVideoPage v-show="currentPage === 'video-to-video'" />
      <AudioToTextPage v-show="currentPage === 'audio-to-text'" />
      <ExtractImageFromVideoPage v-show="currentPage === 'extract-image-from-video'" />
    </main>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import Sidebar from './components/Sidebar.vue'
import ImageToVideoPage from './pages/ImageToVideoPage.vue'
import VideoToVideoPage from './pages/VideoToVideoPage.vue'
import AudioToTextPage from './pages/AudioToTextPage.vue'
import ExtractImageFromVideoPage from './pages/ExtractImageFromVideoPage.vue'

const currentPage = ref('image-to-video')

const handleNavigate = (page) => {
  currentPage.value = page
}
</script>

