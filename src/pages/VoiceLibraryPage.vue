<template>
  <div class="voice-library-page page-container">
    <div class="page-header">
      <div class="header-left">
        <h1 class="page-title">AI Voice Library</h1>
        <p class="page-subtitle">Thư viện giọng nói đa quốc gia: Việt, Anh, Hàn, Nhật, Trung</p>
      </div>
      <div class="header-right">
        <button class="btn-premium-action" @click="showCloningModal = true">
          <span>✨ Clone Your Voice</span>
        </button>
      </div>
    </div>

    <!-- Language & Provider Selector -->
    <div class="top-controls">
      <div class="language-tabs-mini custom-scrollbar">
        <button 
          v-for="lang in languages" 
          :key="lang.code"
          class="lang-tab-mini"
          :class="{ active: currentLang === lang.code }"
          @click="updateLang(lang.code)"
        >
          <span>{{ lang.flag }} {{ lang.name }}</span>
        </button>
      </div>

      <div class="provider-selector">
        <select v-model="currentProvider" class="premium-select">
          <option value="edge">🌐 Microsoft (Free)</option>
          <option value="openai">🔥 OpenAI (Premium)</option>
          <option value="metavoice">💎 MetaVoice (Cinematic)</option>
          <option value="eleven">👑 ElevenLabs (Studio)</option>
        </select>
      </div>
    </div>

    <div class="content-grid">
      <div class="main-column">
        <!-- Input Section -->
        <div class="glass-panel input-section">
          <textarea
            v-model="ttsText"
            class="tts-textarea"
            :placeholder="`Nhập nội dung bằng ${currentLangName} để nghe thử...`"
          ></textarea>
        </div>

        <!-- Voice Selection -->
        <div class="voice-selection-grid">
          <!-- Female Voices -->
          <div class="glass-panel voice-group">
            <h3 class="group-title female">--- Female Voices ---</h3>
            <div class="voice-list custom-scrollbar">
              <button
                v-for="voice in filteredFemaleVoices"
                :key="voice.id"
                class="voice-item"
                :class="{ active: selectedVoice === voice.id }"
                @click="selectedVoice = voice.id"
              >
                <span class="voice-icon">{{ voice.icon }}</span>
                <div class="voice-info">
                  <span class="voice-name">{{ voice.name }}</span>
                  <span class="voice-desc">{{ voice.desc }}</span>
                </div>
              </button>
            </div>
          </div>

          <!-- Male Voices -->
          <div class="glass-panel voice-group">
            <h3 class="group-title male">--- Male Voices ---</h3>
            <div class="voice-list custom-scrollbar">
              <button
                v-for="voice in filteredMaleVoices"
                :key="voice.id"
                class="voice-item"
                :class="{ active: selectedVoice === voice.id }"
                @click="selectedVoice = voice.id"
              >
                <span class="voice-icon">{{ voice.icon }}</span>
                <div class="voice-info">
                  <span class="voice-name">{{ voice.name }}</span>
                  <span class="voice-desc">{{ voice.desc }}</span>
                </div>
              </button>
            </div>
          </div>
        </div>
      </div>

      <div class="side-column">
        <!-- Control Panel -->
        <div class="glass-panel control-panel-ultra">
          <div class="panel-header-mini">
            <h3 class="panel-title-nano">THIẾT LẬP GIỌNG NÓI</h3>
            <span class="premium-badge" v-if="currentProvider !== 'edge'">PREMIUM</span>
          </div>

          <!-- Selected Voice Card -->
          <div class="current-voice-card">
             <div class="cv-icon">{{ currentVoiceConfig?.icon || '🎤' }}</div>
             <div class="cv-info">
                <div class="cv-name">{{ currentVoiceConfig?.name || 'Chưa chọn' }}</div>
                <div class="cv-id">{{ currentVoiceConfig?.voice }}</div>
             </div>
          </div>

          <!-- AI Switch -->
          <div class="ai-opt-toggle">
            <label class="switch-glass">
              <input type="checkbox" v-model="useAIOptimization">
              <span class="slider-glass"></span>
            </label>
            <span class="ai-label">AI Tự nắn giọng (Gemini)</span>
          </div>
          
          <div class="sliders-stack">
             <div class="slider-item">
               <div class="slider-label">Độ trầm: <span>{{ pitch }}Hz</span></div>
               <input type="range" v-model="pitch" min="-100" max="100" step="10" />
             </div>
             <div class="slider-item">
               <div class="slider-label">Tốc độ: <span>{{ rate }}%</span></div>
               <input type="range" v-model="rate" min="-50" max="50" step="5" />
             </div>
             <div class="slider-item">
               <div class="slider-label">Âm lượng: <span>{{ volume }}%</span></div>
               <input type="range" v-model="volume" min="-50" max="50" step="5" />
             </div>
             <div class="slider-item">
               <div class="slider-label">Bass (Trầm): <span>{{ bass }}dB</span></div>
               <input type="range" v-model="bass" min="-10" max="20" step="1" />
             </div>
             <div class="slider-item">
               <div class="slider-label">Treble (Cao): <span>{{ treble }}dB</span></div>
               <input type="range" v-model="treble" min="-10" max="20" step="1" />
             </div>
          </div>

          <!-- API Key Section (Only if premium) -->
          <div v-if="currentProvider !== 'edge'" class="api-key-box animate-fade-in">
             <label>Nhập API Key để dùng giọng người thật:</label>
             <input type="password" v-model="apiKey" placeholder="Dán key của bạn vào đây..." class="nano-input-pk" />
          </div>

          <div class="folder-mini-box">
            <div class="folder-row">
              <input type="text" v-model="outputFolder" readonly placeholder="Thư mục lưu file..." />
              <button @click="selectOutputFolder" class="btn-nano-action">Chọn</button>
            </div>
          </div>

          <div class="action-grid-nano">
            <button class="btn-p" :disabled="isGenerating || isAIAnalyzing" @click="handlePreview">
              <div v-if="isGenerating || isAIAnalyzing" class="loader-nano"></div>
              <span v-else>Nghe thử</span>
            </button>
            <button class="btn-s" :disabled="isGenerating || isAIAnalyzing" @click="handleSave">
              <div v-if="isGenerating || isAIAnalyzing" class="loader-nano"></div>
              <span v-else>Lưu File</span>
            </button>
          </div>

          <div v-if="statusMsg" class="status-msg-nano" :class="statusType">
            {{ statusMsg }}
          </div>

          <div v-if="audioUrl" class="mini-player-dock animate-fade-in">
            <audio ref="audioRef" controls :key="audioUrl" autoplay class="ap-nano">
              <source :src="audioUrl" type="audio/mpeg" />
            </audio>
            <div class="nano-debug-row">
               <button @click="openTempFolder" class="nano-link">🗑️ Thư mục tạm</button>
               <button @click="openInFolder" class="nano-link">📂 Thư mục lưu</button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Modal: Clone Voice -->
    <div v-if="showCloningModal" class="modal-overlay">
      <div class="modal-content glass-panel animate-zoom-in">
        <div class="modal-header">
          <h3>🎙️ Clone giọng nói mới (MetaVoice)</h3>
          <button class="btn-close-modal" @click="showCloningModal = false">✕</button>
        </div>
        <div class="modal-body">
          <div class="cloning-step">
            <label>1. Đặt tên gợi nhớ cho giọng này:</label>
            <input type="text" v-model="cloningName" placeholder="Ví dụ: Giọng của Kien, Giọng sếp..." class="premium-input-v2" />
          </div>
          
          <div class="cloning-step">
            <label>2. Tải lên file giọng mẫu (30s - 1 phút):</label>
            <div class="reference-upload-box" @click="selectReferenceFile">
              <div v-if="!referencePath" class="upload-placeholder">
                <span class="up-icon">📁</span>
                <p>Nén vào đây hoặc Click để chọn file .wav / .mp3</p>
              </div>
              <div v-else class="upload-success">
                <span class="up-icon">✅</span>
                <p>{{ referenceFileName }}</p>
              </div>
            </div>
          </div>

          <div class="cloning-info-box">
            <p>💡 <b>Mẹo:</b> Để kết quả tốt nhất, hãy dùng file âm thanh sạch, không có nhạc nền hoặc tiếng ồn.</p>
          </div>
        </div>
        <div class="modal-footer">
          <button class="btn-cancel" @click="showCloningModal = false">Hủy</button>
          <button class="btn-confirm" :disabled="!cloningName || !referencePath" @click="saveClonedVoice">
            <span>🚀 Lưu vào thư viện</span>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { VOICES_DATA as voicesData, ALL_VOICES as allVoices } from '@/constants/voices'
import { useTTS } from '@/composables/useTTS'

const ttsText = ref('Xin chào, tôi là AI của hệ thống Echose.')
const selectedVoice = ref('vi-vn-female-1')
const audioUrl = ref(null)
const pitch = ref(0)
const rate = ref(0)
const volume = ref(0)
const bass = ref(0)
const treble = ref(0)
const outputFolder = ref('')
const currentLang = ref('vi')
const currentProvider = ref('edge')
const apiKey = ref('')
const isAIAnalyzing = ref(false)
const useAIOptimization = ref(true)
const lastSSML = ref(null)

// Cloning State
const showCloningModal = ref(false)
const cloningName = ref('')
const referencePath = ref('')
const referenceFileName = ref('')
const customVoices = ref([]) // Dũng để lưu danh sách giọng đã clone

// Gemini Key
const GEMINI_KEY = 'AIzaSyDO6Z5xfjjC9jJOtUn0T06OLxeJ_bIEZ1s'

const languages = [
  { code: 'vi', name: 'Tiếng Việt', flag: '🇻🇳' },
  { code: 'en', name: 'English', flag: '🇺🇸' },
  { code: 'ja', name: 'Japanese', flag: '🇯🇵' },
  { code: 'ko', name: 'Korean', flag: '🇰🇷' },
  { code: 'zh', name: 'Chinese', flag: '🇨🇳' },
  { code: 'meta', name: 'MetaVoice', flag: '💎' },
]

const { isGenerating, generateAudio } = useTTS()

const currentLangName = computed(() => languages.find(l => l.code === currentLang.value)?.name)

watch(selectedVoice, (newVoiceId) => {
  const config = allVoices.find(v => v.id === newVoiceId)
  if (config) {
    pitch.value = config.pitch ?? 0
    rate.value = config.rate ?? 0
    volume.value = config.volume ?? 0
    bass.value = config.bass ?? 0
    treble.value = config.treble ?? 0
  }
})

const updateLang = (code) => {
  currentLang.value = code
}

watch(currentLang, (newLang) => {
  if (voicesData[newLang]) {
    if (voicesData[newLang].female && voicesData[newLang].female.length > 0) {
      selectedVoice.value = voicesData[newLang].female[0].id
    } else if (voicesData[newLang].male && voicesData[newLang].male.length > 0) {
      selectedVoice.value = voicesData[newLang].male[0].id
    }
  }
  
  const greetingMap = {
    vi: 'Xin chào, tôi là AI của hệ thống Echose.',
    en: 'Hello, I am the AI from Echose system.',
    ja: 'こんにちは、EchoseシステムのAIです。',
    ko: '안녕하세요, Echose hệ thống của AI입니다.',
    zh: '你好，我是来自Echose系统的AI。'
  }
  if (greetingMap[newLang]) {
    ttsText.value = greetingMap[newLang]
  }
}, { immediate: true })

const selectOutputFolder = async () => {
  const selected = await open({ directory: true, multiple: false })
  if (selected) {
    outputFolder.value = selected
    localStorage.setItem('tts_output_folder', selected)
  }
}

const statusMsg = ref('')
const statusType = ref('info')
const setStatus = (msg, type = 'info') => {
  statusMsg.value = msg
  statusType.value = type
  if (type === 'success') setTimeout(() => { statusMsg.value = '' }, 5000)
}

const analyzeTextWithGemini = async (text) => {
  if (!useAIOptimization.value) return null
  isAIAnalyzing.value = true
  setStatus('Gemini đang "thổi hồn" vào văn bản...', 'info')
  
  try {
    const prompt = `Bạn là chuyên gia âm thanh điện ảnh. Hãy tối ưu văn bản sau để giọng đọc AI (Microsoft Neural) nghe NHƯ NGƯỜI THẬT.
    1. Rewrite văn bản: Nếu là giọng trẻ con (con/em), hãy thêm từ đệm "ạ", "dạ", "nhỉ", "nhé" và dùng đại từ (con, em) cho phù hợp.
    2. Chèn SSML: Sử dụng thẻ <prosody> để điều chỉnh pitch, rate cho từng cụm từ quan trọng. Sử dụng <break time="150ms"/> ở những chỗ cần lấy hơi. Sử dụng <emphasis level="strong"> cho từ khóa.
    3. Trả về JSON: {"ssml": "nội dung SSML đầy đủ bắt đầu bằng <speak> và đóng bằng </speak>", "optimizedText": "văn bản đã rewrite sạch sẽ", "reason": "lý do gợi ý", "pitch": number, "rate": number, "bass": number, "treble": number}
    
    Hãy làm cho phát âm có nhấn nhá, có lúc nhanh lúc chậm, không được đều đều.
    Văn bản: "${text}"
    Giọng đang chọn: ${currentVoiceConfig.value?.name || 'Mặc định'}`

    const response = await fetch(`https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key=${GEMINI_KEY}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        contents: [{ parts: [{ text: prompt }] }]
      })
    })

    const data = await response.json()
    const resultText = data.candidates[0].content.parts[0].text
    const matched = resultText.match(/\{.*\}/s)
    if (matched) {
      const result = JSON.parse(matched[0])
      lastSSML.value = result.ssml
      if (result.optimizedText) ttsText.value = result.optimizedText
      
      pitch.value = result.pitch ?? pitch.value
      rate.value = result.rate ?? rate.value
      bass.value = result.bass ?? bass.value
      treble.value = result.treble ?? treble.value

      setStatus(`AI: ${result.reason}`, 'info')
      return result
    }
  } catch (err) {
    console.error('Gemini Error:', err)
    setStatus('AI phân tích lỗi, dùng văn bản gốc.', 'error')
  } finally {
    isAIAnalyzing.value = false
  }
  return null
}

const handlePreview = async () => {
  if (!ttsText.value) return
  lastSSML.value = null
  await analyzeTextWithGemini(ttsText.value)
  
  setStatus('Đang tạo âm thanh chất lượng Studio...', 'info')
  try {
    const result = await generateAudio(lastSSML.value || ttsText.value, currentVoiceConfig.value, {
      pitch: pitch.value,
      rate: rate.value,
      volume: volume.value,
      bass: bass.value,
      treble: treble.value,
      provider: currentProvider.value,
      apiKey: apiKey.value,
      outputFolder: null
    })
    audioUrl.value = result.audioUrl
    setStatus(`Sẵn sàng!`, 'success')
  } catch (err) {
    setStatus('Lỗi: ' + String(err), 'error')
  }
}

const handleSave = async () => {
  if (!ttsText.value) return
  if (!outputFolder.value) {
    setStatus('Vui lòng chọn thư mục lưu!', 'error')
    return
  }
  lastSSML.value = null
  await analyzeTextWithGemini(ttsText.value)
  
  setStatus('Đang lưu file chất lượng cao...', 'info')
  try {
    const result = await generateAudio(lastSSML.value || ttsText.value, currentVoiceConfig.value, {
      pitch: pitch.value,
      rate: rate.value,
      volume: volume.value,
      bass: bass.value,
      treble: treble.value,
      provider: currentProvider.value,
      apiKey: apiKey.value,
      outputFolder: outputFolder.value
    })
    audioUrl.value = result.audioUrl
    setStatus('Lưu thành công!', 'success')
  } catch (err) {
    setStatus('Lỗi: ' + String(err), 'error')
  }
}

const openInFolder = () => {
  if (outputFolder.value) invoke('open_folder', { path: outputFolder.value })
  else setStatus('Chưa chọn thư mục!', 'error')
}

const openTempFolder = async () => {
  try {
    const home = await invoke('get_home_dir') 
    invoke('open_folder', { path: `${home}/echose_temp` })
  } catch {
    invoke('open_folder', { path: 'echose_temp' }) 
  }
}

// Cloning Logic
const selectReferenceFile = async () => {
  try {
    const selected = await open({
      multiple: false,
      filters: [{ name: 'Audio', extensions: ['mp3', 'wav'] }]
    })
    if (selected) {
      referencePath.value = Array.isArray(selected) ? selected[0] : selected
      referenceFileName.value = referencePath.value.split('/').pop()
    }
  } catch (err) {
    console.error(err)
  }
}

const saveClonedVoice = () => {
  const newVoice = {
    id: `custom-${Date.now()}`,
    voice: 'metavoice-local',
    name: cloningName.value,
    desc: 'Giọng tùy chỉnh (MetaVoice)',
    icon: '👤',
    referencePath: referencePath.value,
    isCustom: true
  }
  
  customVoices.value.push(newVoice)
  localStorage.setItem('custom_cloned_voices', JSON.stringify(customVoices.value))
  
  // Tự động chuyển qua tab meta và chọn giọng mới
  currentLang.value = 'meta'
  selectedVoice.value = newVoice.id
  
  // Reset modal
  showCloningModal.value = false
  cloningName.value = ''
  referencePath.value = ''
  referenceFileName.value = ''
  
  setStatus('Đã thêm giọng mới vào thư viện!', 'success')
}

const loadCustomVoices = () => {
  const saved = localStorage.getItem('custom_cloned_voices')
  if (saved) {
    customVoices.value = JSON.parse(saved)
  }
}

// Update filtered lists to include custom voices when MetaVoice is selected
const filteredFemaleVoices = computed(() => {
  if (currentLang.value === 'meta') {
    return customVoices.value.filter(v => v.isCustom)
  }
  return voicesData[currentLang.value]?.female || []
})

const filteredMaleVoices = computed(() => {
  if (currentLang.value === 'meta') return []
  return voicesData[currentLang.value]?.male || []
})

const currentVoiceConfig = computed(() => {
  const all = [...allVoices, ...customVoices.value]
  return all.find(v => v.id === selectedVoice.value)
})

onMounted(() => {
  const savedFolder = localStorage.getItem('tts_output_folder')
  if (savedFolder) outputFolder.value = savedFolder
  loadCustomVoices()
})
</script>

<style scoped>
.voice-library-page {
  padding: 2.5rem;
  max-width: 1400px;
  margin: 0 auto;
  min-height: 100vh;
  background: linear-gradient(135deg, #0f172a 0%, #1e1b4b 100%);
  color: #f8fafc;
  font-family: 'Inter', system-ui, sans-serif;
}

.page-title {
  font-size: 1.75rem;
  font-weight: 900;
  text-transform: uppercase;
  background: linear-gradient(to right, #ffffff, #94a3b8);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.page-subtitle { font-size: 0.875rem; color: #64748b; margin-top: 0.25rem; }

.top-controls {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 2rem;
  gap: 1.5rem;
}

.language-tabs-mini {
  display: flex; gap: 0.5rem; overflow-x: auto; padding-bottom: 5px; flex: 1;
}

.lang-tab-mini {
  white-space: nowrap; padding: 0.5rem 1rem;
  background: rgba(30, 41, 59, 0.5);
  border: 1px solid rgba(255, 255, 255, 0.05);
  border-radius: 2rem;
  color: #94a3b8; font-size: 0.8rem; font-weight: 600;
  cursor: pointer; transition: all 0.2s;
}

.lang-tab-mini.active { background: #6366f1; color: white; border-color: #818cf8; }

.premium-select {
  background: #1e293b; border: 1px solid #4f46e5; color: white;
  padding: 0.5rem 1rem; border-radius: 0.75rem; font-weight: 600; cursor: pointer;
}

.content-grid { display: grid; grid-template-columns: 1fr 320px; gap: 2rem; }

.input-section { padding: 1.5rem; border-left: 4px solid #6366f1; margin-bottom: 2rem; }
.tts-textarea {
  width: 100%; height: 120px; background: rgba(0, 0, 0, 0.3);
  border: 1px solid #3f3f46; border-radius: 1rem; padding: 1rem;
  color: white; font-size: 1rem; resize: none; outline: none;
}

.voice-selection-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 1.5rem; }
.voice-group { padding: 1.25rem; max-height: 420px; display: flex; flex-direction: column; }
.group-title { font-size: 0.65rem; font-weight: 900; margin-bottom: 1rem; text-transform: uppercase; text-align: center; opacity: 0.5; }
.group-title.female { color: #ec4899; }
.group-title.male { color: #3b82f6; }

.voice-list { flex: 1; overflow-y: auto; padding-right: 0.5rem; display: flex; flex-direction: column; gap: 0.75rem; }

.voice-item {
  display: flex; flex-direction: column; align-items: center;
  padding: 1rem; background: #1e293b; border: 1px solid #334155;
  border-radius: 1rem; cursor: pointer; transition: all 0.2s;
}
.voice-item:hover { background: #334155; transform: translateY(-2px); }
.voice-item.active { background: rgba(99, 102, 241, 0.15) !important; border-color: #6366f1 !important; }

.voice-icon { font-size: 1.5rem; margin-bottom: 0.5rem; }
.voice-name { font-size: 0.75rem; font-weight: 800; color: white; }
.voice-desc { font-size: 0.65rem; color: #64748b; }

.control-panel-ultra {
  padding: 1.25rem; display: flex; flex-direction: column; gap: 1rem; align-self: flex-start; margin-top: -3rem;
}

.panel-header-mini { display: flex; justify-content: space-between; align-items: center; }
.panel-title-nano { font-size: 0.7rem; font-weight: 800; color: #64748b; }
.premium-badge { font-size: 0.6rem; background: #d97706; color: white; padding: 2px 6px; border-radius: 4px; }

.current-voice-card { display: flex; align-items: center; gap: 0.75rem; padding: 0.75rem; background: rgba(255, 255, 255, 0.03); border-radius: 0.75rem; }
.cv-icon { font-size: 1.5rem; }
.cv-name { font-size: 0.8rem; font-weight: 800; color: #818cf8; }
.cv-id { font-size: 0.6rem; color: #475569; }

.ai-opt-toggle {
  display: flex; align-items: center; gap: 0.75rem; padding: 0.5rem 0.75rem;
  background: rgba(99, 102, 241, 0.1); border-radius: 0.75rem; border: 1px solid rgba(99, 102, 241, 0.2);
}
.ai-label { font-size: 0.65rem; font-weight: 700; color: #818cf8; }

.sliders-stack { display: flex; flex-direction: column; gap: 0.75rem; }
.slider-item { display: flex; flex-direction: column; gap: 0.25rem; }
.slider-label { font-size: 0.7rem; color: #94a3b8; display: flex; justify-content: space-between; }
.slider-label span { color: white; font-weight: 700; }

.api-key-box { display: flex; flex-direction: column; gap: 0.5rem; }
.api-key-box label { font-size: 0.65rem; color: #fbbf24; }
.nano-input-pk { background: rgba(0, 0, 0, 0.2); border: 1px solid #d97706; border-radius: 0.5rem; padding: 0.5rem; color: white; font-size: 0.7rem; }

.folder-mini-box { display: flex; gap: 0.5rem; }
.folder-mini-box input { flex: 1; background: rgba(255, 255, 255, 0.02); border: 1px solid rgba(255, 255, 255, 0.05); border-radius: 0.5rem; padding: 0.4rem; font-size: 0.65rem; color: #475569; }
.btn-nano-action { background: #334155; border: none; color: white; padding: 0 0.75rem; border-radius: 0.5rem; font-size: 0.7rem; cursor: pointer; }

.action-grid-nano { display: grid; grid-template-columns: 1fr 1fr; gap: 0.5rem; }
.btn-p, .btn-s { padding: 0.6rem; border-radius: 0.6rem; font-weight: 800; font-size: 0.75rem; cursor: pointer; border: none; color: white; transition: all 0.2s; }
.btn-p { background: #475569; }
.btn-s { background: linear-gradient(135deg, #6366f1 0%, #a855f7 100%); }
.btn-p:hover, .btn-s:hover { transform: translateY(-1px); }

.status-msg-nano { font-size: 0.65rem; padding: 0.5rem; border-radius: 0.5rem; text-align: center; }
.status-msg-nano.info { color: #60a5fa; background: rgba(59, 130, 246, 0.1); }
.status-msg-nano.success { color: #4ade80; background: rgba(34, 197, 94, 0.1); }
.status-msg-nano.error { color: #f87171; background: rgba(239, 68, 68, 0.1); }

.mini-player-dock { background: rgba(0, 0, 0, 0.3); padding: 0.75rem; border-radius: 1rem; }
.ap-nano { width: 100%; height: 32px; }

.nano-debug-row { display: flex; justify-content: space-between; margin-top: 0.5rem; }
.nano-link { background: none; border: none; color: #64748b; font-size: 0.6rem; text-decoration: underline; cursor: pointer; }

/* Switch Glass */
.switch-glass { position: relative; display: inline-block; width: 34px; height: 18px; }
.switch-glass input { opacity: 0; width: 0; height: 0; }
.slider-glass { position: absolute; cursor: pointer; top: 0; left: 0; right: 0; bottom: 0; background-color: rgba(255, 255, 255, 0.1); transition: .4s; border-radius: 20px; }
.slider-glass:before { position: absolute; content: ""; height: 12px; width: 12px; left: 3px; bottom: 3px; background-color: white; transition: .4s; border-radius: 50%; }
input:checked + .slider-glass { background-color: #6366f1; }
input:checked + .slider-glass:before { transform: translateX(16px); }

.loader-nano { width: 14px; height: 14px; border: 2px solid rgba(255, 255, 255, 0.3); border-top-color: white; border-radius: 50%; animation: spin 1s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }

.custom-scrollbar::-webkit-scrollbar { width: 4px; height: 4px; }
.custom-scrollbar::-webkit-scrollbar-thumb { background: rgba(255, 255, 255, 0.1); border-radius: 10px; }

/* Cloning Modal & New UI Styles */
.header-right {
  display: flex;
  align-items: center;
}

.btn-premium-action {
  background: linear-gradient(135deg, #6366f1 0%, #a855f7 100%);
  color: white;
  border: none;
  padding: 0.6rem 1.2rem;
  border-radius: 0.75rem;
  font-weight: 700;
  font-size: 0.85rem;
  cursor: pointer;
  box-shadow: 0 4px 15px rgba(99, 102, 241, 0.3);
  transition: all 0.3s;
}

.btn-premium-action:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(99, 102, 241, 0.4);
}

.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.7);
  backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal-content {
  width: 100%;
  max-width: 500px;
  padding: 2rem;
  border: 1px solid rgba(255, 255, 255, 0.1);
  position: relative;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 2rem;
}

.modal-header h3 {
  margin: 0;
  font-size: 1.25rem;
  color: #818cf8;
}

.btn-close-modal {
  background: none;
  border: none;
  color: #64748b;
  font-size: 1.2rem;
  cursor: pointer;
}

.cloning-step {
  margin-bottom: 1.5rem;
}

.cloning-step label {
  display: block;
  font-size: 0.85rem;
  margin-bottom: 0.75rem;
  color: #94a3b8;
}

.premium-input-v2 {
  width: 100%;
  background: rgba(0, 0, 0, 0.2);
  border: 1px solid #334155;
  padding: 0.75rem 1rem;
  border-radius: 0.75rem;
  color: white;
  outline: none;
}

.reference-upload-box {
  border: 2px dashed #4f46e5;
  background: rgba(79, 70, 229, 0.05);
  border-radius: 1rem;
  padding: 2rem;
  text-align: center;
  cursor: pointer;
  transition: all 0.2s;
}

.reference-upload-box:hover {
  background: rgba(79, 70, 229, 0.1);
  border-color: #6366f1;
}

.up-icon { font-size: 2rem; margin-bottom: 1rem; display: block; }

.cloning-info-box {
  background: rgba(59, 130, 246, 0.05);
  padding: 1rem;
  border-radius: 0.75rem;
  font-size: 0.75rem;
  color: #94a3b8;
  line-height: 1.5;
}

.modal-footer {
  display: flex;
  gap: 1rem;
  margin-top: 2rem;
}

.btn-cancel {
  flex: 1;
  background: #1e293b;
  border: 1px solid #334155;
  color: white;
  padding: 0.75rem;
  border-radius: 0.75rem;
  cursor: pointer;
}

.btn-confirm {
  flex: 2;
  background: linear-gradient(135deg, #6366f1 0%, #a855f7 100%);
  color: white;
  border: none;
  padding: 0.75rem;
  border-radius: 0.75rem;
  font-weight: 700;
  cursor: pointer;
}

.btn-confirm:disabled { opacity: 0.5; cursor: not-allowed; }

.animate-zoom-in {
  animation: zoomIn 0.3s ease-out;
}

@keyframes zoomIn {
  from { opacity: 0; transform: scale(0.95); }
  to { opacity: 1; transform: scale(1); }
}
</style>
