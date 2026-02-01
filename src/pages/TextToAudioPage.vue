<template>
  <div class="text-to-audio-page">
    <div class="text-to-audio-layout">
      <!-- Left: Text Input -->
      <div class="input-column">
        <div class="text-input-card">
          <div class="card-title">
            <span>📝 Nội dung văn bản</span>
            <div class="success-badge" v-if="isGenerated">
              ✨ Đã tạo thành công
            </div>
          </div>
          <textarea 
            v-model="text" 
            class="tts-textarea" 
            placeholder="Nhập nội dung bạn muốn chuyển thành giọng nói tại đây..."
            @input="isGenerated = false"
          ></textarea>
          <div class="input-footer">
            <span>{{ text.length }} ký tự</span>
            <span>Mẹo: Bạn có thể dùng Gemini AI để tối ưu nội dung</span>
          </div>
        </div>

        <!-- Preview Player (Only shows when audio is generated) -->
        <div v-if="audioUrl" class="audio-player-card animate-fade-in">
          <div class="player-info">
            <div class="player-title">🔊 Kết quả vừa tạo</div>
            <audio ref="audioPlayer" :src="audioUrl" controls autoplay></audio>
          </div>
          <div class="player-actions">
            <button class="btn-open-folder" @click="openOutputFolder">📂</button>
          </div>
        </div>
      </div>

      <!-- Right: Settings -->
      <div class="settings-column">
        <div class="settings-card">
          <div class="card-title">⚙️ Cấu hình giọng đọc</div>
          
          <div class="control-group">
            <label class="control-label">Ngôn ngữ</label>
            <select v-model="selectedLang" class="form-select">
              <option v-for="lang in LANGUAGES" :key="lang.code" :value="lang.code">
                {{ lang.flag }} {{ lang.name }}
              </option>
            </select>
          </div>

          <div class="control-group">
            <label class="control-label">Nhà cung cấp</label>
            <select v-model="provider" class="form-select">
              <option value="edge">Microsoft Edge (Miễn phí)</option>
              <option value="local">Nhân bản (Local)</option>
            </select>
          </div>

          <div class="control-group">
            <label class="control-label">Chọn giọng</label>
            <div class="voice-grid custom-scrollbar">
              <div 
                v-for="voice in currentVoices" 
                :key="voice.id"
                class="voice-card-small"
                :class="{ active: selectedVoice === voice.id }"
                @click="selectedVoice = voice.id"
              >
                <div class="voice-avatar">{{ voice.icon }}</div>
                <span class="voice-name-small">{{ voice.name }}</span>
              </div>
            </div>
          </div>

          <div class="settings-divider">Tùy chỉnh âm thanh</div>

          <div class="control-group">
            <label class="control-label">Tốc độ: <span>{{ rate }}%</span></label>
            <input type="range" v-model="rate" min="-50" max="50" step="5" />
          </div>

          <div class="control-group">
            <label class="control-label">Độ trầm (Pitch): <span>{{ pitch }}Hz</span></label>
            <input type="range" v-model="pitch" min="-100" max="100" step="10" />
          </div>

          <div class="control-group">
            <label class="control-label">Thư mục lưu</label>
            <div class="input-group">
              <input type="text" v-model="outputFolder" class="form-input" readonly />
              <button class="btn-select-folder" @click="selectOutputFolder">Chọn</button>
            </div>
          </div>
        </div>

        <div class="actions-card">
          <button class="btn-main-action btn-preview" :disabled="isGenerating || !text" @click="handlePreview">
            <div v-if="isGenerating" class="loading-spinner"></div>
            <span v-else>🎧 Nghe thử</span>
          </button>
          <button class="btn-main-action btn-generate-audio" :disabled="isGenerating || !text" @click="handleSave">
            <div v-if="isGenerating" class="loading-spinner" style="border-top-color: white;"></div>
            <span v-else>🚀 Xuất File Audio</span>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, watch, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { LANGUAGES, VOICES_DATA, ALL_VOICES } from '@/constants/voices'
import { useTTS } from '@/composables/useTTS'
import '@/assets/css/text-to-audio.css'

const text = ref('Chào mừng bạn đến với Echose. Hãy nhập nội dung để chuyển thành giọng nói.')
const selectedLang = ref('ja')
const selectedVoice = ref('ja-jp-f-default')
const provider = ref('edge')
const apiKey = ref('')
const outputFolder = ref('')
const rate = ref(0)
const pitch = ref(0)
const audioUrl = ref(null)
const isGenerated = ref(false)
const customVoices = ref([])

const { isGenerating, generateAudio } = useTTS()

// Lấy danh sách giọng dựa trên ngôn ngữ
const currentVoices = computed(() => {
  if (provider.value === 'local') {
    // Chỉ lấy giọng clone đúng ngôn ngữ đang chọn
    return customVoices.value.filter(v => (v.lang || 'vi') === selectedLang.value)
  }
  const langData = VOICES_DATA[selectedLang.value]
  if (!langData) return []
  return [...langData.female, ...langData.male]
})

// Khi đổi ngôn ngữ, tự động chọn giọng đầu tiên
watch(selectedLang, (newLang) => {
  const langData = VOICES_DATA[newLang]
  if (langData) {
    if (langData.female.length > 0) selectedVoice.value = langData.female[0].id
    else if (langData.male.length > 0) selectedVoice.value = langData.male[0].id
  }
})

// Khi chọn giọng, áp dụng các thiết lập mặc định của giọng đó
watch(selectedVoice, (newVoiceId) => {
  const config = ALL_VOICES.find(v => v.id === newVoiceId)
  if (config) {
    pitch.value = config.pitch ?? 0
    rate.value = config.rate ?? 0
  }
})

const selectOutputFolder = async () => {
  const selected = await open({ directory: true, multiple: false })
  if (selected) {
    outputFolder.value = selected
    localStorage.setItem('tts_output_folder', selected)
  }
}

const handlePreview = async () => {
  const voiceConfig = provider.value === 'local' 
    ? customVoices.value.find(v => v.id === selectedVoice.value)
    : ALL_VOICES.find(v => v.id === selectedVoice.value)
  
  try {
    const result = await generateAudio(text.value, voiceConfig, {
      pitch: pitch.value,
      rate: rate.value,
      volume: 0,
      provider: provider.value,
      apiKey: apiKey.value,
      outputFolder: null // Preview thì lưu vào temp
    })
    audioUrl.value = result.audioUrl
    isGenerated.value = true
  } catch (err) {
    console.error(err)
    alert('Lỗi khi tạo âm thanh: ' + err)
  }
}

const handleSave = async () => {
  if (!outputFolder.value) {
    alert('Vui lòng chọn thư mục lưu trước')
    return
  }
  const voiceConfig = provider.value === 'local' 
    ? customVoices.value.find(v => v.id === selectedVoice.value)
    : ALL_VOICES.find(v => v.id === selectedVoice.value)

  try {
    const result = await generateAudio(text.value, voiceConfig, {
      pitch: pitch.value,
      rate: rate.value,
      volume: 0,
      provider: provider.value,
      apiKey: apiKey.value,
      outputFolder: outputFolder.value
    })
    audioUrl.value = result.audioUrl
    isGenerated.value = true
    alert('Đã lưu file thành công tại: ' + outputFolder.value)
  } catch (err) {
    console.error(err)
    alert('Lỗi khi lưu file: ' + err)
  }
}

const openOutputFolder = () => {
  if (outputFolder.value) {
    invoke('open_folder', { path: outputFolder.value })
  }
}

onMounted(() => {
  const savedFolder = localStorage.getItem('tts_output_folder')
  if (savedFolder) outputFolder.value = savedFolder
  
  const savedCustom = localStorage.getItem('custom_cloned_voices_v3')
  if (savedCustom) customVoices.value = JSON.parse(savedCustom)
})
</script>

<style scoped>
.animate-fade-in {
  animation: fadeIn 0.3s ease-out;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(10px); }
  to { opacity: 1; transform: translateY(0); }
}

.settings-divider {
  font-size: 11px;
  font-weight: 700;
  color: #94a3b8;
  text-transform: uppercase;
  margin: 10px 0 5px;
  border-bottom: 1px solid #f1f5f9;
  padding-bottom: 5px;
}

.custom-scrollbar::-webkit-scrollbar {
  width: 4px;
}

.custom-scrollbar::-webkit-scrollbar-thumb {
  background: #e2e8f0;
  border-radius: 10px;
}

.btn-open-folder {
  background: #f1f5f9;
  border: none;
  width: 40px;
  height: 40px;
  border-radius: 10px;
  cursor: pointer;
  font-size: 18px;
  transition: all 0.2s;
}

.btn-open-folder:hover {
  background: #e2e8f0;
}
</style>
