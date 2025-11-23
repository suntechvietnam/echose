<template>
  <div class="app-container">
    <Sidebar :current-page="currentPage" @navigate="handleNavigate" />
    <main class="main-content">
      <ImageToVideoPage v-if="currentPage === 'image-to-video'" />
    </main>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import Sidebar from './components/Sidebar.vue'
import ImageToVideoPage from './pages/ImageToVideoPage.vue'

const currentPage = ref('image-to-video')

const handleNavigate = (page) => {
  console.log('Navigating to:', page)
  currentPage.value = page
}
</script>

