<template>
  <div class="app-container">
    <Sidebar :current-page="currentPage" @navigate="handleNavigate" />
    <main class="main-content">
      <ImageToVideoPage v-show="currentPage === 'image-to-video'" />
      <AudioToTextPage v-show="currentPage === 'audio-to-text'" />
    </main>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import Sidebar from './components/Sidebar.vue'
import ImageToVideoPage from './pages/ImageToVideoPage.vue'
import AudioToTextPage from './pages/AudioToTextPage.vue'

const currentPage = ref('image-to-video')

const handleNavigate = (page) => {
  currentPage.value = page
}
</script>

