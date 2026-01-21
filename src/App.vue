<template>
  <div class="app-container">
    <Sidebar :current-page="currentPage" @navigate="handleNavigate" />
    <main class="main-content">
      <ImageToVideoPage v-if="currentPage === 'image-to-video'" />
      <VideoToVideoPage v-if="currentPage === 'video-to-video'" />
      <AudioToTextPage v-if="currentPage === 'audio-to-text'" />
      <ExtractImageFromVideoPage v-if="currentPage === 'extract-image-from-video'" />
      <VoiceLibraryPage v-if="currentPage === 'voice-library'" />
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
import VoiceLibraryPage from './pages/VoiceLibraryPage.vue'

const currentPage = ref('voice-library')

const handleNavigate = (page) => {
  console.log('Navigating to:', page)
  currentPage.value = page
}
</script>

