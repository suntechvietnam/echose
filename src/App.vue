<template>
  <div class="app-container">
    <Sidebar :current-page="currentPage" @navigate="handleNavigate" />
    <main class="main-content">
      <DownloadPage v-if="currentPage === 'download'" />
      <AudioMixPage v-else-if="currentPage === 'audio'" />
      <VideoSourcePage v-else-if="currentPage === 'videos'" />
      <VideoCreatePage v-else-if="currentPage === 'create'" />
      <ImageToVideoPage v-else-if="currentPage === 'image-to-video'" />
    </main>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import Sidebar from './components/Sidebar.vue'
import DownloadPage from './pages/DownloadPage.vue'
import AudioMixPage from './pages/AudioMixPage.vue'
import VideoSourcePage from './pages/VideoSourcePage.vue'
import VideoCreatePage from './pages/VideoCreatePage.vue'
import ImageToVideoPage from './pages/ImageToVideoPage.vue'

const currentPage = ref('download')

const handleNavigate = (page) => {
  console.log('Navigating to:', page)
  currentPage.value = page
}

onMounted(() => {
  console.log('App mounted, current page:', currentPage.value)
})
</script>

