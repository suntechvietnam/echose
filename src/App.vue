<template>
  <div class="app-container">
    <Sidebar :current-page="currentPage" @navigate="handleNavigate" />
    <main class="main-content">
      <ImageToVideoPage v-if="currentPage === 'image-to-video'" />
      <VideoToVideoPage v-if="currentPage === 'video-to-video'" />
      <AudioToTextPage v-if="currentPage === 'audio-to-text'" />
      <ExtractImageFromVideoPage v-if="currentPage === 'extract-image-from-video'" />
      <VoiceLibraryPage v-if="currentPage === 'voice-library'" />
      <TextToAudioPage v-if="currentPage === 'text-to-audio'" />
      <DialogueStudioPage v-if="currentPage === 'dialogue-studio'" />
      <VideoAudioMergePage v-if="currentPage === 'video-audio-merge'" />
      <ExtractAudioPage v-if="currentPage === 'extract-audio'" />
      
      <AIImagePage v-if="currentPage === 'ai-image'" />
      <AIVideoPage v-if="currentPage === 'ai-video'" />
      <VoiceCloningPage v-if="currentPage === 'voice-cloning'" @navigate="handleNavigate" />
      
      <!-- Download Tools -->
      <DownloadYoutubePage v-if="currentPage === 'download-youtube'" />
      <DownloadTiktokPage v-if="currentPage === 'download-tiktok'" />
      <DownloadFacebookPage v-if="currentPage === 'download-facebook'" />
      <DownloadInstagramPage v-if="currentPage === 'download-instagram'" />
      <DownloadXPage v-if="currentPage === 'download-x'" />
      
      <!-- Reup Tools -->
      <ReupYoutubePage v-if="currentPage === 'reup-youtube'" />
      <ReupTiktokPage v-if="currentPage === 'reup-tiktok'" />
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
import TextToAudioPage from './pages/TextToAudioPage.vue'
import DialogueStudioPage from './pages/DialogueStudioPage.vue'
import VideoAudioMergePage from './pages/VideoAudioMergePage.vue'
import ExtractAudioPage from './pages/ExtractAudioPage.vue'
import AIImagePage from './pages/AIImagePage.vue'
import AIVideoPage from './pages/AIVideoPage.vue'
import VoiceCloningPage from './pages/VoiceCloningPage.vue'
import ReupYoutubePage from './pages/ReupYoutubePage.vue'
import ReupTiktokPage from './pages/ReupTiktokPage.vue'
import DownloadYoutubePage from './pages/DownloadYoutubePage.vue'
import DownloadTiktokPage from './pages/DownloadTiktokPage.vue'
import DownloadFacebookPage from './pages/DownloadFacebookPage.vue'
import DownloadInstagramPage from './pages/DownloadInstagramPage.vue'
import DownloadXPage from './pages/DownloadXPage.vue'

const currentPage = ref('voice-library')

const handleNavigate = (page) => {
  console.log('Navigating to:', page)
  currentPage.value = page
}
</script>

