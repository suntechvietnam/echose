<template>
  <div class="dialogue-studio-page">
    <div class="page-header">
      <h1 class="page-title">AI Dialogue Studio</h1>
      <p class="page-subtitle">Biến kịch bản thành hội thoại có cảm xúc với Gemini AI & Voice Cloning</p>
    </div>

    <div class="content-grid">
      <!-- PANEL TRÁI: DỮ LIỆU ĐẦU VÀO -->
      <div class="glass-card left-panel">
        <div class="card-inner">
          <div class="panel-tag">SCRIPT EDITOR</div>
          <h3>1. Kịch bản hội thoại</h3>
          
          <textarea 
            v-model="rawScript" 
            class="main-textarea custom-scrollbar" 
            placeholder="Nhập theo định dạng: Tên: Nội dung thoại..."
          ></textarea>

          <div v-if="detectedSpeakers.length > 0" class="mapping-box animate-fade-in">
            <div class="panel-tag">VOICE MAPPING</div>
            <h3>2. Gán giọng nói cho nhân vật</h3>
            <div class="mapping-list">
              <div v-for="speaker in detectedSpeakers" :key="speaker" class="speaker-row">
                <span class="speaker-pill">{{ speaker }}</span>
                <select v-model="speakerMapping[speaker]" class="minimal-select">
                  <option value="">Chọn giọng...</option>
                  <optgroup label="Default Voices">
                    <option v-for="v in standardVoices" :key="v.id" :value="v.id">{{ v.name }}</option>
                  </optgroup>
                  <optgroup label="My Cloned Voices" v-if="clonedVoices.length > 0">
                    <option v-for="v in clonedVoices" :key="v.id" :value="v.id">🎙️ {{ v.name }}</option>
                  </optgroup>
                </select>
              </div>
            </div>
          </div>

          <div class="api-settings animate-fade-in">
            <div class="panel-tag">AI SETTINGS</div>
            <div class="input-with-icon">
              <span class="icon">🔑</span>
              <input 
                type="password" 
                v-model="geminiKeyInput" 
                placeholder="Dán Gemini API Key của bác vào đây..."
                class="minimal-input"
              />
            </div>
            <p class="setting-hint">Yêu cầu Key để tránh bị lỗi 403 do dùng chung.</p>
          </div>

          <button 
            class="btn-action" 
            :class="isAnalyzing ? 'btn-working' : 'primary-glow'"
            :disabled="!rawScript || isAnalyzing" 
            @click="analyzeWithGemini"
          >
            <template v-if="isAnalyzing">
              <div class="loading-ring-small"></div>
              <span>Đang yêu cầu đạo diễn Gemini...</span>
            </template>
            <span v-else>🪄 Gemini Phân tích & Tối ưu</span>
          </button>
        </div>
      </div>

      <!-- PANEL PHẢI: HIỂN THỊ KẾT QUẢ & SẢN XUẤT -->
      <div class="glass-card right-panel">
        <div class="card-inner">
          <div class="panel-header-row">
            <div class="panel-tag">WORKSTATION</div>
            <select v-model="selectedLang" class="lang-picker">
              <option value="vi">Tiếng Việt</option>
              <option value="ja">Japanese</option>
              <option value="en">English</option>
            </select>
          </div>

          <!-- LOGIC HIỂN THỊ (ĐÃ ĐỔI THỨ TỰ ƯU TIÊN) -->
          
          <!-- 1. ƯU TIÊN HIỆN ĐANG PHÂN TÍCH -->
          <div v-if="isAnalyzing" class="status-state animate-fade-in">
            <div class="ai-loader">
               <div class="dot"></div><div class="dot"></div><div class="dot"></div>
            </div>
            <p>Gemini đang "thổi hồn" vào kịch bản...</p>
          </div>

          <!-- 2. ƯU TIÊN HIỆN LỖI (NẾU CÓ) -->
          <div v-else-if="analysisError" class="status-state error animate-slide-up">
            <div class="error-icon">⚠️</div>
            <h3>Không thể xử lý AI</h3>
            <p v-if="analysisError.includes('429')">Bác ơi, Key này đang bị quá tải. Bác vui lòng đợi 60s hoặc dùng chế độ mặc định nhé.</p>
            <p v-else>{{ analysisError }}</p>
            <div class="error-actions" style="margin-top: 1.5rem; display: flex; gap: 1rem;">
               <button class="btn-action primary-glow" style="padding: 0.6rem 1.2rem; width: auto;" @click="analyzeWithGemini">Thử lại</button>
               <button class="btn-action" style="padding: 0.6rem 1.2rem; width: auto; background: #334155;" @click="useDefaultAnalysis">Dùng thông số mặc định</button>
            </div>
          </div>

          <!-- 3. HIỆN DANH SÁCH KẾT QUẢ (NẾU THÀNH CÔNG) -->
          <div v-else-if="analyzedSegments.length > 0" class="results-area custom-scrollbar animate-fade-in">
             <div v-for="(seg, idx) in analyzedSegments" :key="idx" class="seg-card" :class="{ active: currentProcessingIdx === idx }">
                <div class="seg-header">
                   <div class="seg-speaker">{{ seg.speaker }}</div>
                   <div class="seg-metrics">
                      <span class="m-item">Speed: <b>{{ seg.speed }}x</b></span>
                      <span class="m-item">Pitch: <b>{{ seg.pitch }}Hz</b></span>
                   </div>
                </div>
                <p class="seg-text">{{ seg.text }}</p>
                <div class="seg-reason" v-if="seg.reason"><b>AI Insight:</b> {{ seg.reason }}</div>
                <div class="seg-badge" v-if="seg.status" :class="seg.status">{{ seg.statusText }}</div>
             </div>
          </div>

          <!-- 4. CUỐI CÙNG MỚI HIỆN MÀN HÌNH CHỜ -->
          <div v-else class="status-state idle animate-fade-in">
            <div class="idle-icon">🎭</div>
            <h3>Sẵn sàng xử lý</h3>
            <p>Hãy nhấn nút tối ưu bên trái để Gemini bắt đầu phân tích cảm xúc kịch bản.</p>
          </div>

          <!-- PHẦN XUẤT FILE: THIẾT KẾ LẠI CAO CẤP -->
          <div class="production-footer" v-if="analyzedSegments.length > 0">
            <div class="footer-label">CẤU HÌNH XUẤT FILE</div>
            <div class="folder-selection-group">
              <div class="input-container">
                <span class="folder-icon">📂</span>
                <input type="text" v-model="outputFolder" readonly placeholder="Chọn thư mục lưu kết quả..." />
              </div>
              <button class="btn-browse" @click="selectOutputFolder">Chọn thư mục</button>
            </div>
            
            <button 
              class="btn-produce-final" 
              :disabled="isProducing || !outputFolder" 
              @click="produceDialogue"
            >
              <div v-if="isProducing" class="loader-spinner"></div>
              <span v-if="!isProducing">🚀 XUẤT AUDIO HOÀN CHỈNH</span>
              <span v-else>Đang trộn âm thanh... ({{ currentProcessingIdx + 1 }}/{{ analyzedSegments.length }})</span>
            </button>
          </div>

          <!-- FINAL RESULT PLAYER -->
          <div v-if="finalAudioUrl" class="success-player animate-slide-up">
            <p>🎉 Kết quả phòng thu của bác:</p>
            <audio :src="finalAudioUrl" controls class="w-full"></audio>
            <button class="btn-folder-link" @click="openTargetFolder">� Xem trong thư mục</button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, watch, onMounted } from 'vue'
import { invoke, convertFileSrc } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { GoogleGenerativeAI } from '@google/generative-ai'
import { VOICES_DATA, ALL_VOICES } from '@/constants/voices'
import { useTTS } from '@/composables/useTTS'
import { GEMINI_API_KEY } from '@/config/gemini'

/**
 * LOGIC KHI CLICK NÚT TỐI ƯU (analyzeWithGemini):
 * 1. Khởi tạo State: Dòng 184-188: Set isAnalyzing=true (Hiện màn hình loading), xóa dữ liệu cũ/lỗi cũ.
 * 2. Gọi Gemini: Dòng 190-201: Gửi Prompt yêu cầu phân tích Context câu nói và trả về JSON.
 * 3. Xử lý kịch bản: Dòng 213-228: Bóc tách JSON từ phản hồi của Gemini một cách an toàn.
 * 4. Phản hồi UI: Stage này isAnalyzing = false. Nếu lỗi gán vào analysisError, thành công gán vào analyzedSegments.
 */

const rawScript = ref('')
const selectedLang = ref('vi')
const speakerMapping = ref({})
const isAnalyzing = ref(false)
const isProducing = ref(false)
const analyzedSegments = ref([])
const analysisError = ref(null)
const currentProcessingIdx = ref(-1)
const outputFolder = ref('')
const finalAudioUrl = ref(null)
const geminiKeyInput = ref('')

const { generateAudio } = useTTS()
// API Key mặc định (được đọc từ file cấu hình riêng biệt)
const DEFAULT_KEY = GEMINI_API_KEY || ''

// Load voices
const clonedVoices = ref([])
const standardVoices = computed(() => {
  const data = VOICES_DATA[selectedLang.value]
  return data ? [...data.female, ...data.male] : []
})

// Speaker Detection (Supports standard and Japanese full-width colon)
const detectedSpeakers = computed(() => {
  if (!rawScript.value) return []
  const speakers = new Set()
  rawScript.value.split('\n').forEach(line => {
    const match = line.match(/^([^:：]+)[:：]/)
    if (match) speakers.add(match[1].trim())
  })
  return Array.from(speakers)
})

watch(detectedSpeakers, (newSpeakers) => {
  newSpeakers.forEach(s => { if (!speakerMapping.value[s]) speakerMapping.value[s] = '' })
})

const analyzeWithGemini = async () => {
  if (!rawScript.value) return
  
  isAnalyzing.value = true          
  analyzedSegments.value = []      
  analysisError.value = null       
  
  try {
    const activeKey = geminiKeyInput.value || DEFAULT_KEY
    if (!activeKey) {
        // Nếu không có Key, không cần chờ AI, dùng mặc định luôn cho nhanh
        useDefaultAnalysis()
        return
    }

    // Khởi tạo SDK chính chủ (Giống hệt dự án Kidnihon của bác)
    const genAI = new GoogleGenerativeAI(activeKey)
    const model = genAI.getGenerativeModel({ model: "gemini-flash-latest" })

    const prompt = `Bạn là đạo diễn âm thanh chuyên nghiệp. Hãy phân tích hội thoại sau và trả về MẢNG JSON.
    Nội dung: "${rawScript.value}"
    Yêu cầu: {"speaker": "tên", "text": "câu nói", "speed": float (0.8-1.2), "pitch": 0, "reason": "lý do"}.
    Chỉ trả về JSON Array.`

    const result = await model.generateContent(prompt)
    const response = await result.response
    const txt = response.text().trim()

    if (!txt) throw new Error("AI phản hồi trống rỗng.")

    // Bóc tách JSON an toàn
    const jsonStart = txt.indexOf('[')
    const jsonEnd = txt.lastIndexOf(']')
    
    if (jsonStart !== -1 && jsonEnd !== -1) {
       const cleanJson = txt.substring(jsonStart, jsonEnd + 1)
       analyzedSegments.value = JSON.parse(cleanJson)
       if (analyzedSegments.value.length === 0) throw new Error("AI không tìm thấy lời thoại.")
    } else {
       throw new Error("Format error")
    }

  } catch (err) {
    console.error("AI Analysis failed, falling back to local:", err)
    // TỰ ĐỘNG CỨU CÁNH: Nếu AI lỗi, tự động chia kịch bản bằng code nội bộ
    useDefaultAnalysis()
  } finally {
    isAnalyzing.value = false       
  }
}

// LOGIC CỨU CÁNH: Chia kịch bản không cần AI
const useDefaultAnalysis = () => {
  if (!rawScript.value) return
  const lines = rawScript.value.split('\n')
  const segments = []
  
  lines.forEach(line => {
    // Regex nhận diện "Tên: Nội dung"
    const match = line.match(/^([^:：]+)[:：](.*)/)
    if (match) {
      segments.push({
        speaker: match[1].trim(),
        text: match[2].trim(),
        speed: 1.0,
        pitch: 0,
        reason: "Tự động phân tích (AI đang bận)"
      })
    }
  })
  
  if (segments.length > 0) {
    analyzedSegments.value = segments
    analysisError.value = null // Xóa lỗi nếu có vì đã có data dự phòng
  } else {
    analysisError.value = "Kịch bản không đúng định dạng 'Tên: Câu nói'. Bác kiểm tra lại nhé!"
  }
}

const produceDialogue = async () => {
  if (analyzedSegments.value.length === 0 || !outputFolder.value) return
  for (const speaker of detectedSpeakers.value) {
    if (!speakerMapping.value[speaker]) { alert(`Lỗi: Bác chưa chọn giọng cho "${speaker}"`); return; }
  }

  isProducing.value = true
  finalAudioUrl.value = null
  const tempFiles = []
  
  try {
    for (let i = 0; i < analyzedSegments.value.length; i++) {
        currentProcessingIdx.value = i
        const seg = analyzedSegments.value[i]
        const voiceId = speakerMapping.value[seg.speaker]
        const voiceConfig = [...ALL_VOICES, ...clonedVoices.value].find(v => v.id === voiceId)
        
        seg.status = 'processing'; seg.statusText = 'Rendering...'

        const result = await generateAudio(seg.text, voiceConfig, {
            rate: (seg.speed - 1) * 100, 
            pitch: seg.pitch,
            provider: voiceConfig.isCustom ? 'local' : 'edge',
            readingSpeed: seg.speed,
            lang: selectedLang.value
        })
        tempFiles.push(result.filePath)
        seg.status = 'done'; seg.statusText = 'Completed'
    }

    const finalPath = `${outputFolder.value}/dialogue_${Date.now()}.mp3`
    const mergedPath = await invoke('merge_audio_files', { inputPaths: tempFiles, outputPath: finalPath })
    finalAudioUrl.value = convertFileSrc(mergedPath)
    alert("🚀 Hội thoại của bác đã được lắp ghép hoàn chỉnh!")
  } catch (err) {
    alert("Lỗi sản xuất: " + err)
  } finally {
    isProducing.value = false; currentProcessingIdx.value = -1
  }
}

const selectOutputFolder = async () => {
  const selected = await open({ directory: true, multiple: false })
  if (selected) { outputFolder.value = selected; localStorage.setItem('studio_output_folder', selected) }
}

const openTargetFolder = () => { if (outputFolder.value) invoke('open_folder', { path: outputFolder.value }) }

onMounted(() => {
  const savedV3 = localStorage.getItem('custom_cloned_voices_v3')
  if (savedV3) clonedVoices.value = JSON.parse(savedV3)
  const savedFolder = localStorage.getItem('studio_output_folder')
  if (savedFolder) outputFolder.value = savedFolder
  
  const savedKey = localStorage.getItem('gemini_api_key_studio')
  if (savedKey) geminiKeyInput.value = savedKey
})

watch(geminiKeyInput, (newVal) => {
  localStorage.setItem('gemini_api_key_studio', newVal)
})
</script>

<style scoped>
/* COLOR SYSTEM */
:root {
  --bg-dark: #0a0a0c;
  --card-bg: rgba(255, 255, 255, 0.03);
  --primary: #a855f7;
  --primary-glow: rgba(168, 85, 247, 0.3);
  --secondary: #06b6d4;
  --accent: #f43f5e;
  --silver: #94a3b8;
  --glass: blur(12px);
}

.dialogue-studio-page {
  padding: 3rem;
  background: radial-gradient(circle at 50% 0%, #1a1a20, #0a0a0c);
  min-height: 100vh;
  color: #fff;
  font-family: 'Outfit', sans-serif;
}

.page-header { margin-bottom: 3rem; text-align: center; }
.page-title { 
  font-size: 3rem; font-weight: 900; letter-spacing: -2px;
  background: linear-gradient(to right, var(--primary), var(--secondary));
  -webkit-background-clip: text; -webkit-text-fill-color: transparent;
}
.page-subtitle { color: var(--silver); margin-top: 0.5rem; }

.content-grid {
  display: grid; grid-template-columns: 460px 1fr; gap: 2.5rem; max-width: 1600px; margin: 0 auto;
}

.glass-card {
  background: var(--card-bg);
  backdrop-filter: var(--glass);
  border: 1px solid rgba(255, 255, 255, 0.05);
  border-radius: 2rem;
  padding: 0.5rem;
  box-shadow: 0 30px 60px rgba(0,0,0,0.4);
}

.card-inner { padding: 1.5rem; height: 100%; display: flex; flex-direction: column; }

.panel-tag {
  font-size: 0.65rem; font-weight: 800; color: var(--secondary);
  letter-spacing: 2px; text-transform: uppercase; margin-bottom: 0.5rem;
  opacity: 0.7;
}

h3 { margin-bottom: 1.5rem; font-size: 1.1rem; font-weight: 600; color: var(--silver); }

.main-textarea {
  width: 100%; height: 380px; background: rgba(0,0,0,0.4); border: 1px solid rgba(255,255,255,0.05);
  border-radius: 1.5rem; padding: 1.5rem; color: #fff; font-size: 1rem; line-height: 1.6;
  outline: none; transition: 0.3s; resize: none; margin-bottom: 1.5rem;
}
.main-textarea:focus { border-color: var(--primary); box-shadow: 0 0 20px var(--primary-glow); }

.api-settings { margin-bottom: 2rem; background: rgba(0,0,0,0.2); padding: 1rem; border-radius: 1rem; }
.input-with-icon { position: relative; display: flex; align-items: center; }
.input-with-icon .icon { position: absolute; left: 12px; font-size: 0.8rem; }
.minimal-input {
  width: 100%; background: #16161a; border: 1px solid rgba(255,255,255,0.05); color: #fff;
  padding: 0.7rem 0.7rem 0.7rem 2.5rem; border-radius: 0.75rem; font-size: 0.8rem; outline: none;
}
.minimal-input:focus { border-color: var(--secondary); }
.setting-hint { font-size: 0.65rem; color: #475569; margin-top: 0.5rem; padding-left: 0.5rem; }

.speaker-pill {
  background: rgba(255,255,255,0.05); padding: 0.4rem 1rem; border-radius: 2rem;
  font-size: 0.8rem; font-weight: 700; color: var(--secondary); border: 1px solid rgba(6, 182, 212, 0.2);
}

.mapping-list { display: flex; flex-direction: column; gap: 1rem; margin-top: 1rem; margin-bottom: 2rem; }
.speaker-row { display: flex; align-items: center; justify-content: space-between; gap: 1rem; }

.minimal-select {
  flex: 1; background: #16161a; border: 1px solid rgba(255,255,255,0.1); color: #fff;
  padding: 0.6rem; border-radius: 0.8rem; font-size: 0.85rem; cursor: pointer;
}

.btn-action {
  width: 100%; padding: 1.2rem; border-radius: 1rem; border: none; font-weight: 900;
  cursor: pointer; transition: 0.3s; font-size: 1rem;
}
.primary-glow {
  background: linear-gradient(135deg, var(--primary), #7c3aed); color: #fff;
  box-shadow: 0 10px 30px var(--primary-glow);
}
.primary-glow:hover { transform: translateY(-3px); box-shadow: 0 15px 40px var(--primary-glow); }

/* WORKSTATION STATES */
.right-panel { height: 850px; }
.status-state {
  flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center;
  text-align: center; color: var(--silver); padding: 2rem;
}
.idle-icon { font-size: 5rem; opacity: 0.15; margin-bottom: 2rem; }
.ai-loader { display: flex; gap: 8px; margin-bottom: 1.5rem; }
.ai-loader .dot {
  width: 12px; height: 12px; background: var(--primary); border-radius: 50%;
  animation: pulse 1.5s infinite;
}
.ai-loader .dot:nth-child(2) { animation-delay: 0.2s; }
.ai-loader .dot:nth-child(3) { animation-delay: 0.4s; }

.error-icon { font-size: 4rem; color: var(--accent); margin-bottom: 1rem; }
.error h3 { color: var(--accent); }
.btn-retry {
  margin-top: 1.5rem; padding: 0.8rem 2rem; background: var(--accent); color: white;
  border: none; border-radius: 0.75rem; cursor: pointer; font-weight: 700;
}

.results-area { flex: 1; overflow-y: auto; padding-right: 1.5rem; display: flex; flex-direction: column; gap: 1.5rem; }
.seg-card {
  background: rgba(255,255,255,0.02); border-left: 4px solid var(--primary);
  border-radius: 1.2rem; padding: 1.5rem; transition: 0.3s;
}
.seg-card.active { background: rgba(168, 85, 247, 0.05); border-left-color: var(--secondary); }
.seg-speaker { font-weight: 900; color: var(--primary); font-size: 0.75rem; text-transform: uppercase; margin-bottom: 0.8rem; }
.seg-metrics { display: flex; gap: 1.5rem; font-size: 0.75rem; color: var(--silver); margin-bottom: 1rem; }
.seg-metrics b { color: #fff; }
.seg-text { font-size: 1.1rem; line-height: 1.5; color: #f1f5f9; }
.seg-reason { margin-top: 1rem; font-size: 0.8rem; color: var(--silver); background: rgba(0,0,0,0.2); padding: 0.8rem; border-radius: 0.8rem; opacity: 0.8; }

.production-footer {
  margin-top: 1.5rem;
  padding: 1.5rem;
  background: rgba(0, 0, 0, 0.4);
  border-top: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 0 0 1.5rem 1.5rem;
}

.footer-label {
  font-size: 0.65rem;
  letter-spacing: 1.5px;
  color: var(--silver);
  margin-bottom: 0.8rem;
  font-weight: 700;
  opacity: 0.6;
}

.folder-selection-group {
  display: flex;
  gap: 10px;
  margin-bottom: 1.2rem;
}

.folder-selection-group .input-container {
  flex: 1;
  position: relative;
  display: flex;
  align-items: center;
}

.folder-selection-group .folder-icon {
  position: absolute;
  left: 12px;
  opacity: 0.5;
}

.folder-selection-group input {
  width: 100%;
  background: #111114;
  border: 1px solid rgba(255, 255, 255, 0.05);
  color: #fff;
  padding: 0.7rem 0.7rem 0.7rem 2.5rem;
  border-radius: 0.75rem;
  font-size: 0.85rem;
  outline: none;
}

.btn-browse {
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  color: var(--silver);
  padding: 0 1.2rem;
  border-radius: 0.75rem;
  font-size: 0.8rem;
  font-weight: 600;
  cursor: pointer;
  white-space: nowrap;
  transition: 0.2s;
}

.btn-browse:hover {
  background: rgba(255, 255, 255, 0.1);
  color: #fff;
}

.btn-produce-final {
  width: 100%;
  padding: 1.2rem;
  background: linear-gradient(135deg, var(--secondary), #0891b2);
  color: #fff;
  border: none;
  border-radius: 1.2rem;
  font-weight: 900;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  transition: 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  box-shadow: 0 8px 25px rgba(6, 182, 212, 0.2);
}

.btn-produce-final:hover:not(:disabled) {
  transform: translateY(-2px);
  filter: brightness(1.1);
  box-shadow: 0 12px 30px rgba(6, 182, 212, 0.3);
}

.btn-produce-final:disabled {
  opacity: 0.4;
  cursor: not_allowed;
  filter: grayscale(0.5);
}

.loader-spinner {
  width: 18px;
  height: 18px;
  border: 2.5px solid rgba(255, 255, 255, 0.2);
  border-top-color: #fff;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

.success-player {
  margin-top: 1.5rem;
  padding: 1.5rem;
  background: rgba(6, 182, 212, 0.05);
  border: 1px solid rgba(6, 182, 212, 0.2);
  border-radius: 1.5rem;
}

.custom-scrollbar::-webkit-scrollbar { width: 5px; }
.custom-scrollbar::-webkit-scrollbar-thumb { background: rgba(255,255,255,0.05); border-radius: 10px; }
</style>
