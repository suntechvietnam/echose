<template>
  <div class="voice-cloning-page voice-theme page-container">
    <div class="page-header">
      <div class="header-left">
        <h1 class="page-title">Voice Cloning Studio (Coqui XTTS v2)</h1>
        <p class="page-subtitle">Nhân bản giọng nói chuyên nghiệp với công nghệ XTTS v2 (Mac M1 Accelerated)</p>
      </div>
      <div class="header-right">
      </div>
    </div>



    <!-- 📚 VOICE MANAGEMENT VIEW -->
    <div class="voice-management-layout">
       <!-- Left Sidebar: Voice List -->
       <div class="voice-sidebar glass-panel">
          <div class="sidebar-header">
             <h3>Giọng của tôi</h3>
             <button class="btn-add-new" @click="createNewVoice">➕ Thêm giọng mới</button>
          </div>
          <div class="voice-list-scroll custom-scrollbar">
             <div v-for="voice in savedVoices" 
                  :key="voice.id" 
                  class="voice-item-compact"
                  :class="{ active: selectedVoiceId === voice.id }"
                  @click="selectVoice(voice)"
             >
                <span class="v-icon">🎙️</span>
                <div class="v-info">
                   <div class="v-name">{{ voice.name }}</div>
                   <div class="v-meta">{{ voice.refCount }} mẫu • {{ voice.lang.toUpperCase() }}</div>
                </div>
                <button class="v-delete" @click.stop="deleteVoice(voice.id)">✕</button>
             </div>
             <div v-if="savedVoices.length === 0" class="empty-sidebar">
                <p>Chưa có giọng nào.</p>
             </div>
          </div>
       </div>

       <!-- Right: Workspace -->
       <div class="voice-workspace glass-panel">
          <div v-if="!selectedVoiceId && !isCreatingNew" class="welcome-workspace">
             <div class="welcome-icon">🔮</div>
             <h2>Chọn một giọng để sử dụng hoặc thêm giọng mới</h2>
             <p>Bạn có thể nhân bản bất kỳ giọng nói nào từ file âm thanh mẫu (WAV/MP3).</p>
             <button class="btn-primary-large" @click="createNewVoice">Bắt đầu ngay</button>
          </div>

          <div v-else class="active-workspace animation-fade-in">
             <div class="workspace-header">
                <h2>{{ isCreatingNew ? 'Tạo giọng nhân bản mới' : 'Cấu hình giọng: ' + selectedVoiceName }}</h2>
                <div class="header-actions">
                   <button v-if="!isCreatingNew" class="btn-update-ref" @click="addReferenceFile">➕ Thêm file mẫu cải thiện chất lượng</button>
                </div>
             </div>

             <!-- Reference Files List -->
             <div class="reference-section">
                <h4 class="sub-title">1. Các file mẫu (References)</h4>
                <div class="ref-files-grid">
                   <div v-for="(path, idx) in referencePaths" :key="idx" class="ref-file-tag">
                      <span>🎙️ {{ getFileName(path) }}</span>
                      <button @click="removeReference(idx)">✕</button>
                   </div>
                   <div v-if="referencePaths.length === 0" class="empty-ref">Vui lòng thêm ít nhất 1 file mẫu</div>
                </div>
                <div class="ref-actions" v-if="isCreatingNew">
                   <button class="btn-sm-action" @click="addReferenceFile">📁 Chọn file</button>
                   <button class="btn-sm-action" :class="{recording: isRecording}" @click="toggleRecording">🎙️ {{ isRecording ? 'Dừng thu' : 'Thu âm' }}</button>
                </div>
             </div>

             <div v-if="isCreatingNew" class="new-voice-name-box">
                <label>Đặt tên cho giọng này:</label>
                <input v-model="newVoiceName" placeholder="Ví dụ: Giọng của Kien, Giọng Sếp..." class="glow-input-large" />
             </div>

             <!-- Creation Area (If some ref exists) -->
             <div class="generation-area" v-if="referencePaths.length > 0">
                <h4 class="sub-title">2. Nội dung cần đọc</h4>
                
                <div class="lang-selector-group">
                  <select v-model="selectedLang" class="glow-select-small">
                    <option value="vi">Tiếng Việt (vi)</option>
                    <option value="en">Tiếng Anh (en)</option>
                    <option value="ja">Tiếng Nhật (ja)</option>
                    <option value="ko">Tiếng Hàn (ko)</option>
                  </select>
                  <button class="btn-split" :disabled="!text || isProcessing" @click="handleManualSplit">✂️ AI Split</button>
                </div>

                <textarea v-model="text" class="glow-textarea" placeholder="Nhập văn bản cần đọc..."></textarea>
                
                <div class="speed-config-mini">
                   <label>Tốc độ: <b>{{ readingSpeed }}x</b></label>
                   <div class="speed-presets">
                      <button v-for="s in [0.5, 0.8, 1.0, 1.2, 1.5]" :key="s" :class="{active: readingSpeed === s}" @click="readingSpeed = s">{{ s }}</button>
                   </div>
                </div>

                <div class="workspace-footer">
                   <button v-if="isCreatingNew" class="btn-save-voice" :disabled="!newVoiceName" @click="saveNewVoice">💾 Lưu giọng vào thư viện</button>
                   <button v-else class="btn-update-voice" @click="updateExistingVoice">💾 Cập nhật file mẫu</button>
                   
                   <div class="main-action-buttons">
                      <button v-if="!isProcessing" class="btn-clone-main" :disabled="!text" @click="startCloning">🔮 TẠO FILE ÂM THANH</button>
                      <button v-else class="btn-stop-main" @click="stopCloning">⏹️ DỪNG LẠI</button>
                   </div>
                </div>

                <!-- Result View (Inline) -->
                <div v-if="outputUrl || isProcessing" class="result-box-inline animate-slide-up">
                   <div v-if="isProcessing" class="proc-view">
                      <div class="loader-voice"></div>
                      <p>{{ statusMsg }}</p>
                   </div>
                   <div v-else class="audio-view">
                      <audio ref="audioPlayer" :src="outputUrl" controls autoplay class="w-full"></audio>
                      <div class="audio-actions">
                         <button @click="downloadAudio">📥 Tải về</button>
                         <button @click="saveResultToLibrary">💾 Lưu vào file đã tạo</button>
                      </div>
                   </div>
                </div>
             </div>
          </div>
       </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { invoke, convertFileSrc } from '@tauri-apps/api/core'
import { open, save, message, ask } from '@tauri-apps/plugin-dialog'
import { join, documentDir } from '@tauri-apps/api/path' 
import { mkdir, BaseDirectory } from '@tauri-apps/plugin-fs' 

// Khai báo emit đúng chuẩn Vue 3
const emit = defineEmits(['navigate'])

const debugLog = ref("")

const log = (msg) => {
  console.log(msg)
  debugLog.value += msg + "\n"
}

// 1. TEST TỐC ĐỘ
const testSpeedDifference = async () => {
  if (referencePaths.value.length === 0) {
    alert("Vui lòng chọn file mẫu trước!")
    return
  }
  
  log("⏳ Đang test tốc độ...")
  const text = "今回の決定は、会社の将来のために不可欠な投資であると同時に、社員の皆さまの努力に報いることができず、大変な心苦しさを感じています。"
  const refPath = referencePaths.value[0]
  
  try {
    // Test 1.0 (Bình thường)
    log("▶️ Đang tạo file tốc độ 1.0...")
    const start1 = Date.now()
    const path1 = await invoke('clone_voice_metavoice', {
        text: text,
        referenceAudioPath: refPath,
        outputPath: '/tmp/test_speed_1.0.wav',
        lang: 'ja',
        speed: 1.0
    });
    const dur1 = (Date.now() - start1) / 1000
    const audio1 = new Audio(convertFileSrc(path1))
    
    // Test 0.7 (Chậm)
    log("▶️ Đang tạo file tốc độ 0.7...")
    const start2 = Date.now()
    const path2 = await invoke('clone_voice_metavoice', {
        text: text,
        referenceAudioPath: refPath,
        outputPath: '/tmp/test_speed_0.7.wav',
        lang: 'ja',
        speed: 0.7
    });
    const dur2 = (Date.now() - start2) / 1000
    
    // So sánh thời gian xử lý (hoặc anh có thể nghe trực tiếp)
    log("✅ KẾT QUẢ:")
    log(`- File 1.0: Saved at ${path1}`)
    log(`- File 0.7: Saved at ${path2}`)
    
    // Lấy độ dài file thực tế
    const duration1 = await invoke('get_audio_duration', { path: path1 })
    const duration2 = await invoke('get_audio_duration', { path: path2 })
    
    log(`⏱️ Độ dài file 1.0: ${duration1.toFixed(2)}s`)
    log(`⏱️ Độ dài file 0.7: ${duration2.toFixed(2)}s`)
    
    if (duration2 > duration1) {
      log("🎉 THÀNH CÔNG: File 0.7 dài hơn file 1.0! Tốc độ đã thay đổi.")
    } else {
      log("❌ THẤT BẠI: Độ dài như nhau.")
    }
    
  } catch (e) {
    log("❌ Lỗi: " + e)
  }
}

// 2. TEST LƯU
const testSaveFunction = async () => {
   log("💾 Bắt đầu quy trình lưu giả lập...")
   
   try {
     const voiceId = 'test-' + Date.now();
     log(`1. Tạo ID: ${voiceId}`)
     
     const newVoice = {
       id: voiceId,
       name: "Test Voice " + Date.now(),
       voice: 'xtts-v2-local',
       isCustom: true
     };
     
     log("2. Đọc localStorage cũ...")
     const currentLibrary = JSON.parse(localStorage.getItem('custom_cloned_voices') || '[]');
     log(`   - Số lượng hiện tại: ${currentLibrary.length}`)
     
     const updatedLibrary = [...currentLibrary, newVoice];
     log("3. Ghi đè localStorage mới...")
     localStorage.setItem('custom_cloned_voices', JSON.stringify(updatedLibrary));
     
     // Verify lại ngay lập tức
     const check = JSON.parse(localStorage.getItem('custom_cloned_voices') || '[]');
     if (check.length === currentLibrary.length + 1) {
       log(`✅ LƯU THÀNH CÔNG! Số lượng mới: ${check.length}`)
       log("🚀 Chuẩn bị chuyển trang (emit navigate)...")
       setTimeout(() => {
         emit('navigate', 'voice-library');
         log("✅ Đã gọi emit. Nếu màn hình chuyển, chức năng OK.")
       }, 1000)
     } else {
       log("❌ LƯU THẤT BẠI. LocalStorage không đổi.")
     }
     
   } catch(e) {
     log("❌ Lỗi: " + e)
   }
}

// State
const text = ref('')
const selectedLang = ref('en')
const referencePaths = ref([])
const outputUrl = ref(null)
const outputPathRaw = ref(null)
const isProcessing = ref(false)
const statusMsg = ref('')
const totalChunks = ref(0)
const processedChunks = ref(0)
const isRecording = ref(false)
const isPlaying = ref(false)
const showLibrary = ref(false)
const savedVoices = ref([])
const savedResults = ref([]) // NEW: Lưu kết quả audio
const currentTab = ref('voices') // NEW: Tab switch
const geminiApiKey = ref('') // Vẫn giữ để người dùng ghi đè nếu muốn
const showGeminiKey = ref(false)
const textChunks = ref([]) // Mảng chứa các đoạn đã chia
const shouldCancel = ref(false) // Cờ dừng xử lý
const readingSpeed = ref(1.0) // Tốc độ đọc (Default 1.0)

// Gemini System Key (Lấy từ bộ cấu hình hệ thống)
const GEMINI_SYSTEM_KEY = 'AIzaSyAqB51ROyUbsoFgPvLo0QXwwRhyUqcAVlw'

// Refs
const audioPlayer = ref(null)
let mediaRecorder = null
let audioChunks = []

// 🚀 RESET VÀ LOAD LOGIC MỚI
onMounted(() => {
  // Reset các giọng clone cũ nếu dùng key cũ (tùy chọn bác có thể đổi key để xoá sạch)
  const NEW_STORAGE_KEY = 'custom_cloned_voices_v3';
  const OLD_STORAGE_KEY = 'custom_cloned_voices';
  
  const stored = localStorage.getItem(NEW_STORAGE_KEY);
  if (stored) {
    savedVoices.value = JSON.parse(stored);
  } else {
    // Nếu chưa có v3, xoá sạch cái cũ để bắt đầu lại theo yêu cầu của bác
    localStorage.removeItem(OLD_STORAGE_KEY);
  }
  
  // Load Saved Results
  const storedResults = localStorage.getItem('saved_generated_audios');
  if (storedResults) {
    savedResults.value = JSON.parse(storedResults);
  }

  const savedKey = localStorage.getItem('gemini_api_key');
  if (savedKey) {
    geminiApiKey.value = savedKey;
  }
})

const STORAGE_KEY = 'custom_cloned_voices_v3';

// 🔄 VOICE SELECTION & EDITING
const isCreatingNew = ref(false)
const selectedVoiceId = ref(null)
const selectedVoiceName = ref('')
const newVoiceName = ref('')

const createNewVoice = () => {
   isCreatingNew.value = true;
   selectedVoiceId.value = null;
   referencePaths.value = [];
   newVoiceName.value = '';
   outputUrl.value = null;
}

const selectVoice = (voice) => {
   isCreatingNew.value = false;
   selectedVoiceId.value = voice.id;
   selectedVoiceName.value = voice.name;
   referencePaths.value = [...(voice.paths || [voice.referencePath])];
   selectedLang.value = voice.lang || 'vi';
   outputUrl.value = null;
}

const saveNewVoice = () => {
   if (!newVoiceName.value || referencePaths.value.length === 0) return;
   
   const newVoice = {
      id: 'custom-' + Date.now(),
      name: newVoiceName.value,
      voice: 'xtts-v2-local',
      paths: [...referencePaths.value],
      referencePath: referencePaths.value[0],
      lang: selectedLang.value,
      refCount: referencePaths.value.length,
      isCustom: true
   };
   
   savedVoices.value = [newVoice, ...savedVoices.value];
   localStorage.setItem(STORAGE_KEY, JSON.stringify(savedVoices.value));
   
   alert(`🎉 Đã thêm giọng "${newVoiceName.value}" thành công!`);
   isCreatingNew.value = false;
   selectedVoiceId.value = newVoice.id;
   selectedVoiceName.value = newVoice.name;
}

const updateExistingVoice = () => {
   if (!selectedVoiceId.value) return;
   
   const voiceIdx = savedVoices.value.findIndex(v => v.id === selectedVoiceId.value);
   if (voiceIdx !== -1) {
      savedVoices.value[voiceIdx].paths = [...referencePaths.value];
      savedVoices.value[voiceIdx].refCount = referencePaths.value.length;
      localStorage.setItem(STORAGE_KEY, JSON.stringify(savedVoices.value));
      alert(`✅ Đã cập nhật file mẫu cho giọng "${selectedVoiceName.value}"!`);
   }
}

const saveGeminiKey = () => {
  localStorage.setItem('gemini_api_key', geminiApiKey.value);
}

const saveLibrary = () => {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(savedVoices.value));
}

// File Helpers
const getFileName = (path) => path.split(/[/\\]/).pop()

const addReferenceFile = async () => {
  const selected = await open({
    multiple: true,
    filters: [{ name: 'Audio', extensions: ['wav', 'mp3'] }]
  })
  if (selected) {
    const paths = Array.isArray(selected) ? selected : [selected];
    referencePaths.value.push(...paths.map(s => s.path || s));
  }
}

const removeReference = (index) => {
  referencePaths.value.splice(index, 1);
}

const clearReference = () => {
  referencePaths.value = []
}

// Recording Logic
const toggleRecording = async () => {
  if (isRecording.value) {
    if (mediaRecorder) mediaRecorder.stop()
    isRecording.value = false
  } else {
    try {
      const stream = await navigator.mediaDevices.getUserMedia({ audio: true })
      mediaRecorder = new MediaRecorder(stream)
      audioChunks = []
      mediaRecorder.ondataavailable = e => audioChunks.push(e.data)
      mediaRecorder.onstop = async () => {
        const audioBlob = new Blob(audioChunks, { type: 'audio/wav' })
        const arrayBuffer = await audioBlob.arrayBuffer()
        const uint8Array = new Uint8Array(arrayBuffer)
        const savedPath = await invoke('save_temp_audio', { audioData: Array.from(uint8Array) })
        referencePaths.value.push(savedPath)
      }
      mediaRecorder.start()
      isRecording.value = true
    } catch (err) {
      alert("Lỗi Micro: " + err)
    }
  }
}

// 🧠 CHUNKING LOGIC: Chia văn bản thành các đoạn nhỏ
const splitTextWithGemini = async (fullText) => {
  const activeKey = geminiApiKey.value || GEMINI_SYSTEM_KEY;
  if (!activeKey) return null;

  try {
    statusMsg.value = "Gemini đang phân tích ngữ cảnh để chia đoạn...";
    const response = await fetch(`https://generativelanguage.googleapis.com/v1/models/gemini-1.5-flash:generateContent?key=${activeKey}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        contents: [{
          parts: [{
            text: `Bạn là chuyên gia xử lý ngôn ngữ Nhật Bản.
Hãy chia đoạn văn bản tiếng Nhật dưới đây thành một mảng JSON các chuỗi.

Yêu cầu NGHIÊM NGẶT:
1. Mỗi chuỗi dài 100-150 ký tự tiếng Nhật (bao gồm cả Kanji, Hiragana, Katakana).
2. PHẢI ngắt ở vị trí tự nhiên: dấu chấm (。), xuống dòng, hoặc hết một ý.
3. Tuyệt đối KHÔNG thay đổi, dịch, hoặc thêm bớt nội dung gốc.
4. Trả về DUY NHẤT một mảng JSON. Ví dụ: ["đoạn 1", "đoạn 2", "đoạn 3"]
5. KHÔNG viết giải thích, KHÔNG thêm markdown code block.

Văn bản cần chia:
${fullText}`
          }]
        }]
      })
    });

    const data = await response.json();
    console.log('Gemini response:', data);
    
    if (data.error) {
      console.error('Gemini API error:', data.error);
      throw new Error(data.error.message);
    }
    
    if (!data.candidates || !data.candidates[0]) {
      console.error('No candidates in response');
      return null;
    }
    
    const rawText = data.candidates[0].content.parts[0].text;
    console.log('Raw Gemini output:', rawText);
    
    // Làm sạch kết quả
    let cleanText = rawText.replace(/```json/g, '').replace(/```/g, '').trim();
    
    // Nếu Gemini trả về text thay vì JSON, thử parse
    const result = JSON.parse(cleanText);
    
    if (Array.isArray(result) && result.length > 0) {
      console.log(`Gemini chia thành ${result.length} đoạn`);
      return result;
    }
    
    return null;
  } catch (err) {
    console.error("Lỗi Gemini Splitter:", err);
    return null; 
  }
}

const handleManualSplit = async () => {
  if (!text.value.trim()) return;
  isProcessing.value = true;
  try {
    const result = await splitTextWithGemini(text.value);
    if (result) {
      textChunks.value = result;
      statusMsg.value = "Đã chia đoạn thành công!";
      setTimeout(() => { isProcessing.value = false; statusMsg.value = ""; }, 1000);
    } else {
      textChunks.value = splitTextIntoChunks(text.value);
      isProcessing.value = false;
    }
  } catch (e) {
    textChunks.value = splitTextIntoChunks(text.value);
    isProcessing.value = false;
  }
}

const splitTextIntoChunks = (fullText, maxChars = 120) => {
  // Chia theo ký tự cho tiếng Nhật
  const sentences = [];
  let current = '';
  
  for (const char of fullText) {
    current += char;
    if (char === '。' || char === '！' || char === '？' || char === '\n' || (char === '）' && current.length > 50)) {
      if (current.trim()) {
        sentences.push(current.trim());
      }
      current = '';
    }
  }
  
  if (current.trim()) {
    sentences.push(current.trim());
  }
  
  const chunks = [];
  let currentChunk = '';
  
  for (const sentence of sentences) {
    // Nếu câu quá dài, chia nhỏ hơn
    if (sentence.length > maxChars) {
      if (currentChunk) {
        chunks.push(currentChunk.trim());
        currentChunk = '';
      }
      
      const parts = sentence.split('、');
      for (const part of parts) {
        if ((currentChunk + part).length > maxChars && currentChunk) {
          chunks.push(currentChunk.trim());
          currentChunk = part;
        } else {
          currentChunk += currentChunk ? '、' + part : part;
        }
      }
      continue;
    }
    
    if ((currentChunk + sentence).length > maxChars && currentChunk) {
      chunks.push(currentChunk.trim());
      currentChunk = sentence;
    } else {
      const prefix = (sentence.startsWith('１') || sentence.startsWith('２') || sentence.startsWith('３')) ? '\n' : '';
      currentChunk += prefix + sentence;
    }
  }
  
  if (currentChunk.trim()) {
    chunks.push(currentChunk.trim());
  }
  
  // Nếu không chia được, chia cứng
  if (chunks.length === 0) {
    const result = [];
    for (let i = 0; i < fullText.length; i += maxChars) {
      result.push(fullText.substring(i, i + maxChars));
    }
    return result;
  }
  
  return chunks;
}

// Clone Logic
const isValid = computed(() => text.value.trim().length > 0 && referencePaths.value.length > 0)

const startCloning = async () => {
  if (!isValid.value) return
  
  isProcessing.value = true
  shouldCancel.value = false
  outputUrl.value = null
  outputPathRaw.value = null
  processedChunks.value = 0
  
  try {
    // 1. Kiểm tra chunks
    if (textChunks.value.length === 0) {
       statusMsg.value = "Đang chia nhỏ văn bản...";
       textChunks.value = await splitTextWithGemini(text.value) || splitTextIntoChunks(text.value);
    }
    
    let chunks = textChunks.value;
    totalChunks.value = chunks.length;
    const chunkFiles = [];

    // Tham chiếu (nối các file bằng dấu phẩy cho python script)
    const refParam = referencePaths.value.join(",");

    for (let i = 0; i < chunks.length; i++) {
      // Kiểm tra nếu người dùng ấn Stop
      if (shouldCancel.value) {
        statusMsg.value = "Đã dừng xử lý.";
        return;
      }
      
      statusMsg.value = `Đang xử lý AI phần ${i+1}/${chunks.length}...`;
      const tempOut = `/tmp/chunk_${Date.now()}_${i}.wav`;
      
      const resultPart = await invoke('clone_voice_metavoice', {
        text: chunks[i],
        referenceAudioPath: refParam,
        outputPath: tempOut,
        lang: selectedLang.value,
        speed: readingSpeed.value
      });
      
      chunkFiles.push(resultPart);
      processedChunks.value++;
    }

    // Kiểm tra lần cuối trước khi merge
    if (shouldCancel.value) {
      statusMsg.value = "Đã dừng xử lý.";
      return;
    }

    // Nối tất cả tệp lại thành file cuối
    statusMsg.value = "Đang hợp nhất các đoạn âm thanh...";
    const finalOutput = `/tmp/echose_final_${Date.now()}.wav`;
    const mergedPath = await invoke('merge_audio_files', {
      inputPaths: chunkFiles,
      outputPath: finalOutput
    });

    outputPathRaw.value = mergedPath;
    outputUrl.value = convertFileSrc(mergedPath);
    statusMsg.value = "Hoàn tất!";
    
  } catch (err) {
    if (!shouldCancel.value) {
      alert("Lỗi: " + String(err))
    }
  } finally {
    isProcessing.value = false
    shouldCancel.value = false
  }
}

const stopCloning = () => {
  shouldCancel.value = true;
  statusMsg.value = "Đang dừng xử lý...";
}

// 💾 LIBRARY LOGIC
const openSaveVoiceDialog = async () => {
  console.log("🛠️ Open Save Dialog called");
  // Tự lấy tên file mẫu làm tên gợi ý
  let defaultName = "Giọng mới";
  if (referencePaths.value.length > 0) {
    defaultName = referencePaths.value[0].split(/[/\\]/).pop().split('.')[0];
  }

  const name = prompt("Nhập tên cho giọng nói này để lưu vào thư viện:", defaultName);
  if (!name) return;

  try {
    const voiceId = 'custom-' + Date.now();
    console.log("🛠️ Creating new voice object:", voiceId);
    
    const newVoice = {
      id: voiceId,
      name: name,
      voice: 'xtts-v2-local',
      desc: 'Giọng nhân bản (XTTS v2)',
      icon: '🎙️',
      paths: [...referencePaths.value],
      referencePath: referencePaths.value[0],
      lang: selectedLang.value,
      speed: readingSpeed.value,
      refCount: referencePaths.value.length,
      isCustom: true
    };
    
    // Cập nhật và lưu ngay lập tức
    console.log("🛠️ Saving to localStorage...");
    const currentLibrary = JSON.parse(localStorage.getItem('custom_cloned_voices') || '[]');
    const updatedLibrary = [...currentLibrary, newVoice];
    
    savedVoices.value = updatedLibrary;
    localStorage.setItem('custom_cloned_voices', JSON.stringify(updatedLibrary));
    console.log("✅ Saved to localStorage success");
    
    alert("✅ Đã lưu giọng '" + name + "' vào thư viện thành công!");
    
    // Chuyển sang trang Library
    console.log("🚀 Emitting navigate event...");
    emit('navigate', 'voice-library');
    console.log("✅ Emit called");
  } catch (e) {
    console.error("❌ Save Error:", e);
    alert("❌ Lỗi khi lưu vào thư viện: " + e);
  }
}

const useVoiceFromLibrary = (voice) => {
  if (voice.paths && voice.paths.length > 0) {
    referencePaths.value = [...voice.paths];
  } else if (voice.referencePath) {
    referencePaths.value = [voice.referencePath];
  }
  selectedLang.value = voice.lang || 'en';
  showLibrary.value = false;
  alert(`Đã nạp giọng "${voice.name}"!`);
}

const deleteVoice = async (id) => {
  const confirmed = await ask("Bạn có chắc chắn muốn xóa giọng này khỏi thư viện?", "Xóa giọng");
  if (confirmed) {
    savedVoices.value = savedVoices.value.filter(v => v.id !== id);
    saveLibrary();
    // Nếu giọng đang xóa là giọng đang chọn, đưa về màn hình chào
    if (selectedVoiceId.value === id) {
      selectedVoiceId.value = null;
    }
  }
}

// 📥 DOWNLOAD LOGIC (Fixing)
const downloadAudio = async () => {
  if (!outputPathRaw.value) {
    alert('Không tìm thấy file kết quả để tải!');
    return;
  }

  try {
    const savePath = await save({
      filters: [{ name: 'Audio', extensions: ['wav'] }],
      defaultPath: 'cloned_voice.wav'
    });

    if (savePath) {
      console.log('Đang lưu từ:', outputPathRaw.value, 'vào:', savePath);
      await invoke('copy_external_file', {
        src: outputPathRaw.value,
        dest: savePath
      });
      alert('✅ Đã lưu file thành công tại: ' + savePath);
    }
  } catch (err) {
    alert('❌ Lỗi khi lưu file: ' + String(err));
    console.error('Download error:', err);
  }
}

// 💾 NEW: SAVE RESULT TO LIBRARY
const saveResultToLibrary = async () => {
  if (!outputPathRaw.value) {
    alert("Chưa có kết quả để lưu!");
    return;
  }
  
  const name = prompt("Đặt tên cho file ghi âm này:", "Audio_" + new Date().toLocaleTimeString());
  if (!name) return;
  
  try {
     const docDir = await documentDir();
     // Path: Documents/Balocco/echose/saved_audio
     const saveDir = await join(docDir, 'Balocco', 'echose', 'saved_audio');
     
     // 1. Tạo thư mục nếu chưa có
     // check exists? plugin-fs doesn't have existSync easily, use mkdir recursive to be safe
     // Actually mkdir with recursive:true won't limit if exists
     await mkdir('Balocco/echose/saved_audio', { baseDir: BaseDirectory.Document, recursive: true });
     
     const fileName = `saved_${Date.now()}.wav`;
     const destPath = await join(saveDir, fileName);
     
     // 2. Copy file
     await invoke('copy_external_file', {
        src: outputPathRaw.value,
        dest: destPath
     });
     
     // 3. Save metadata
     const newResult = {
        id: 'res-' + Date.now(),
        name: name,
        path: destPath,
        date: new Date().toLocaleDateString() + ' ' + new Date().toLocaleTimeString(),
        duration: audioPlayer.value ? Math.round(audioPlayer.value.duration) : 0,
        textPreview: text.value.substring(0, 50) + '...',
        speed: readingSpeed.value
     };
     
     const updated = [newResult, ...savedResults.value];
     savedResults.value = updated;
     localStorage.setItem('saved_generated_audios', JSON.stringify(updated));
     
     alert(`✅ Đã lưu audio "${name}" thành công!`);
     showLibrary.value = true; // Hiển thị library view tại chỗ
     currentTab.value = 'results'; // Chuyển sang tab kết quả
     
  } catch(e) {
     console.error(e);
     alert("Lỗi khi lưu: " + e);
  }
}

const deleteSavedResult = async (id) => {
   const confirmed = await ask("Bạn có chắc chắn muốn xóa file này?", "Xóa Audio");
   if (confirmed) {
      savedResults.value = savedResults.value.filter(r => r.id !== id);
      localStorage.setItem('saved_generated_audios', JSON.stringify(savedResults.value));
   }
}

const playSavedResult = (res) => {
   // Chơi file đã lưu
   const url = convertFileSrc(res.path);
   outputUrl.value = url;
   outputPathRaw.value = res.path; // Update current path so user can download/re-save if needed
   showLibrary.value = false; // Close library
   // Auto play
   setTimeout(() => {
     if (audioPlayer.value) audioPlayer.value.play();
   }, 500);
}
</script>

<style scoped>
.voice-theme {
  --primary: #10b981;
  --bg: #0f172a;
  --panel: rgba(255, 255, 255, 0.03);
  --glass: rgba(255, 255, 255, 0.05);
}

.page-container {
  padding: 1.5rem 2.5rem;
  height: 100vh;
  background: var(--bg);
  color: white;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 2rem;
}

.btn-secondary {
  padding: 0.75rem 1.5rem;
  background: var(--glass);
  border: 1px solid rgba(255,255,255,0.1);
  border-radius: 0.75rem;
  color: white;
  cursor: pointer;
  transition: 0.3s;
}
.btn-secondary:hover { background: rgba(255,255,255,0.1); }

.content-grid-voice {
  display: grid;
  grid-template-columns: 480px 1fr;
  gap: 2rem;
  flex: 1;
  min-height: 0;
}

.glass-panel {
  background: var(--panel);
  border: 1px solid rgba(255, 255, 255, 0.05);
  border-radius: 2rem;
  padding: 2rem;
  backdrop-filter: blur(20px);
  display: flex;
  flex-direction: column;
}

.full-width { width: 100%; }

.section-block { margin-bottom: 2rem; }
.section-title { font-size: 0.9rem; color: var(--primary); margin-bottom: 0.5rem; font-weight: 700; text-transform: uppercase; }
.section-desc { font-size: 0.8rem; color: #94a3b8; margin-bottom: 1rem; }

/* Upload Area */
.file-picker-area { display: flex; flex-direction: column; gap: 0.75rem; }
.upload-controls { display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; }

.btn-upload, .btn-record {
  padding: 0.85rem; border: 2px dashed #334155; background: rgba(0,0,0,0.2); 
  border-radius: 0.75rem; color: #94a3b8; cursor: pointer; display: flex; 
  align-items: center; justify-content: center; gap: 0.5rem; transition: 0.3s; font-weight: 600; font-size: 0.85rem;
}
.btn-upload:hover { border-color: var(--primary); color: white; background: rgba(16, 185, 129, 0.1); }
.btn-record { border-style: solid; background: rgba(255, 255, 255, 0.05); }
.btn-record:hover { border-color: #ef4444; color: white; background: rgba(239, 68, 68, 0.1); }
.btn-record.recording { background: rgba(239, 68, 68, 0.2); border-color: #ef4444; animation: pulse-red 2s infinite; }

.selected-file-display {
  padding: 0.75rem 1rem; background: rgba(16, 185, 129, 0.1); border: 1px solid var(--primary);
  border-radius: 0.75rem; display: flex; align-items: center; gap: 0.75rem;
}
.file-name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 0.8rem; }
.btn-remove { background: none; border: none; color: #ef4444; cursor: pointer; }

/* Form Elements */
.lang-selector-group { display: flex; align-items: center; gap: 1rem; margin-bottom: 1rem; }
.glow-select { flex: 1; padding: 0.6rem; background: rgba(0,0,0,0.3); border: 1px solid #334155; border-radius: 0.75rem; color: white; }
.glow-textarea { width: 100%; height: 150px; padding: 1rem; background: rgba(0,0,0,0.3); border: 1px solid #334155; border-radius: 1rem; color: white; resize: none; font-family: inherit; }
.glow-textarea:focus { border-color: var(--primary); outline: none; box-shadow: 0 0 15px rgba(16, 185, 129, 0.2); }

/* Processing & Progress */
.processing-state { text-align: center; margin-top: 3rem; }
.status-msg { margin-bottom: 1.5rem; font-weight: 600; color: var(--primary); }
.chunk-progress-container { width: 80%; margin: 0 auto; }
.progress-bar-bg { height: 8px; background: rgba(255,255,255,0.1); border-radius: 4px; overflow: hidden; margin-bottom: 0.5rem; }
.progress-bar-fill { height: 100%; background: var(--primary); transition: width 0.3s ease; }
.chunks-info { font-size: 0.75rem; color: #94a3b8; }

/* Audio Result */
.audio-result-container { display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100%; gap: 1.5rem; }
.audio-visual { width: 140px; height: 140px; border-radius: 50%; background: rgba(16, 185, 129, 0.05); border: 2px solid var(--primary); display: flex; align-items: center; justify-content: center; position: relative; }
.visual-circle { width: 70%; height: 70%; background: var(--primary); border-radius: 50%; box-shadow: 0 0 30px var(--primary); transition: transform 0.3s; }
.visual-circle.playing { animation: pulse-green 1.5s infinite; }
.main-audio { width: 100%; max-width: 450px; }

.result-actions { display: flex; gap: 1rem; }
.btn-action { padding: 0.75rem 1.5rem; border: none; border-radius: 0.75rem; color: white; font-weight: 700; cursor: pointer; transition: 0.2s; }
.btn-download { background: #334155; }
.btn-download:hover { background: #475569; }
.btn-save-lib { background: var(--primary); }
.btn-save-lib:hover { opacity: 0.9; }

/* Library Grid */
.voice-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: 1.5rem; margin-top: 1.5rem; }
.voice-card { background: rgba(255,255,255,0.05); border: 1px solid rgba(255,255,255,0.1); border-radius: 1.25rem; padding: 1.25rem; display: flex; align-items: center; gap: 1rem; transition: 0.3s; }
.voice-card:hover { transform: translateY(-5px); background: rgba(255,255,255,0.08); border-color: var(--primary); }
.voice-card-icon { font-size: 2rem; }
.voice-card-info { flex: 1; }
.voice-name { font-weight: 700; margin-bottom: 0.25rem; }
.voice-meta { font-size: 0.7rem; color: #94a3b8; text-transform: uppercase; }
.voice-card-actions { display: flex; flex-direction: column; gap: 0.5rem; }
.btn-card-use { background: var(--primary); border: none; padding: 0.4rem 0.8rem; border-radius: 0.5rem; color: white; font-weight: bold; cursor: pointer; }
.btn-card-delete { background: none; border: none; color: #ef4444; cursor: pointer; font-size: 1.1rem; }

/* Animations */
.animation-fade-in { animation: fadeIn 0.5s ease; }
.animation-slide-up { animation: slideUp 0.5s ease; }
@keyframes fadeIn { from { opacity: 0; } to { opacity: 1; } }
@keyframes slideUp { from { transform: translateY(20px); opacity: 0; } to { transform: translateY(0); opacity: 1; } }
@keyframes pulse-red { 0% { box-shadow: 0 0 0 0 rgba(239, 68, 68, 0.4); } 70% { box-shadow: 0 0 0 10px rgba(239, 68, 68, 0); } 100% { box-shadow: 0 0 0 0 rgba(239, 68, 68, 0); } }
@keyframes pulse-green { 0% { transform: scale(1); box-shadow: 0 0 20px var(--primary); } 50% { transform: scale(1.1); box-shadow: 0 0 40px var(--primary); } 100% { transform: scale(1); box-shadow: 0 0 20px var(--primary); } }

.btn-clone {
  width: 100%; padding: 1.25rem; background: linear-gradient(135deg, #10b981 0%, #059669 100%);
  border: none; border-radius: 1rem; color: white; font-weight: 800; cursor: pointer;
  transition: 0.3s; box-shadow: 0 10px 25px rgba(16, 185, 129, 0.3);
}
.btn-clone:disabled { opacity: 0.5; cursor: not-allowed; }

.btn-stop {
  width: 100%; padding: 1.25rem; background: linear-gradient(135deg, #ef4444 0%, #dc2626 100%);
  border: none; border-radius: 1rem; color: white; font-weight: 800; cursor: pointer;
  transition: 0.3s; box-shadow: 0 10px 25px rgba(239, 68, 68, 0.3);
  animation: pulse-red-btn 2s infinite;
}
.btn-stop:hover {
  transform: translateY(-2px);
  box-shadow: 0 15px 35px rgba(239, 68, 68, 0.5);
}

@keyframes pulse-red-btn {
  0% { box-shadow: 0 10px 25px rgba(239, 68, 68, 0.3); }
  50% { box-shadow: 0 10px 35px rgba(239, 68, 68, 0.6); }
  100% { box-shadow: 0 10px 25px rgba(239, 68, 68, 0.3); }
}

.btn-split {
  padding: 0.5rem 1rem;
  background: var(--glass);
  border: 1px solid var(--primary);
  border-radius: 0.5rem;
  color: var(--primary);
  font-size: 0.75rem;
  font-weight: 700;
  cursor: pointer;
  white-space: nowrap;
  transition: 0.2s;
}
.btn-split:hover:not(:disabled) {
  background: var(--primary);
  color: white;
}
.btn-split:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.pro-tips-panel { margin-top: 1.5rem; padding: 1rem; background: rgba(16, 185, 129, 0.05); border: 1px dashed rgba(16, 185, 129, 0.2); border-radius: 1rem; }
.tips-title { font-size: 0.8rem; color: var(--primary); margin-bottom: 0.4rem; font-weight: 800; }
.tips-list { list-style: none; padding: 0; font-size: 0.75rem; color: #94a3b8; line-height: 1.4; }
.tips-list li::before { content: "• "; color: var(--primary); }

.empty-state, .empty-library { text-align: center; color: #94a3b8; padding: 3rem; }
.empty-icon { font-size: 3rem; margin-bottom: 1rem; opacity: 0.3; }

.loader-voice { width: 24px; height: 24px; border: 3px solid rgba(255,255,255,0.3); border-top-color: white; border-radius: 50%; animation: spin 1s linear infinite; margin: 0 auto; }
@keyframes spin { to { transform: rotate(360deg); } }

/* Gemini Config Styles */
.gemini-config {
  margin-top: 1rem;
  background: rgba(255, 255, 255, 0.02);
  padding: 0.75rem;
  border-radius: 0.75rem;
  border: 1px solid rgba(255,255,255,0.05);
}
.flex-between {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.btn-text-link {
  background: none;
  border: none;
  color: var(--primary);
  font-size: 0.75rem;
  text-decoration: underline;
  cursor: pointer;
}
.api-key-input-wrapper {
  margin-top: 0.75rem;
}
.glow-input {
  width: 100%;
  background: rgba(0,0,0,0.3);
  border: 1px solid #334155;
  border-radius: 0.5rem;
  padding: 0.5rem;
  color: white;
  font-size: 0.8rem;
}
.glow-input:focus { border-color: var(--primary); outline: none; }
.input-hint {
  font-size: 0.65rem;
  color: #64748b;
  margin-top: 0.3rem;
}

/* Chunks Review Styles */
.text-chunks-review {
  margin-top: 1.5rem;
  border-top: 1px solid rgba(255,255,255,0.05);
  padding-top: 1rem;
}
.sub-section-title {
  font-size: 0.75rem;
  color: var(--primary);
  margin-bottom: 1rem;
  text-transform: uppercase;
  letter-spacing: 1px;
}
.chunks-list {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  max-height: 400px;
  overflow-y: auto;
  padding-right: 0.5rem;
}
.chunk-item {
  background: rgba(255,255,255,0.02);
  border: 1px solid rgba(255,255,255,0.05);
  border-radius: 0.75rem;
  padding: 0.75rem;
}
.chunk-header {
  font-size: 0.65rem;
  color: #94a3b8;
  margin-bottom: 0.5rem;
  font-weight: 800;
}
.chunk-textarea {
  width: 100%;
  height: 80px;
  background: rgba(0,0,0,0.2);
  border: 1px solid #334155;
  border-radius: 0.5rem;
  color: white;
  font-size: 0.8rem;
  padding: 0.5rem;
  resize: none;
}
.chunk-textarea:focus { border-color: var(--primary); outline: none; }
.speed-config {
  margin-top: 1rem;
  padding: 1rem;
}
.speed-hint {
  font-size: 0.7rem;
  color: #64748b;
}
.speed-slider {
  width: 100%;
  margin: 1rem 0 0.5rem;
  accent-color: var(--primary);
  cursor: pointer;
}
/* 🚀 NEW LAYOUT STYLES */
.voice-management-layout {
  display: grid;
  grid-template-columns: 320px 1fr;
  gap: 1.5rem;
  flex: 1;
  min-height: 0;
  margin-top: 1rem;
}

.voice-sidebar {
  display: flex;
  flex-direction: column;
  padding: 1.5rem !important;
  border-radius: 1.5rem !important;
}

.sidebar-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1.5rem;
}
.sidebar-header h3 { font-size: 1rem; color: var(--primary); }

.btn-add-new {
  background: var(--primary);
  border: none; color: white; padding: 0.4rem 0.8rem;
  border-radius: 0.5rem; font-size: 0.75rem; font-weight: 700; cursor: pointer;
}

.voice-list-scroll {
  flex: 1; overflow-y: auto; display: flex; flex-direction: column; gap: 0.75rem;
}

.voice-item-compact {
  background: rgba(255,255,255,0.03);
  border: 1px solid rgba(255,255,255,0.05);
  border-radius: 1rem; padding: 1rem;
  display: flex; align-items: center; gap: 1rem;
  cursor: pointer; transition: 0.3s; position: relative;
}
.voice-item-compact:hover { background: rgba(255,255,255,0.07); }
.voice-item-compact.active { border-color: var(--primary); background: rgba(16, 185, 129, 0.1); }

.v-icon { font-size: 1.5rem; }
.v-info .v-name { font-weight: 700; font-size: 0.9rem; }
.v-info .v-meta { font-size: 0.7rem; color: #64748b; }

.v-delete {
  position: absolute; right: 0.75rem; top: 50%; transform: translateY(-50%);
  background: rgba(239, 68, 68, 0.1); border: 1px solid rgba(239, 68, 68, 0.2); 
  color: #ef4444; width: 24px; height: 24px; border-radius: 50%;
  display: flex; align-items: center; justify-content: center;
  font-size: 0.7rem; cursor: pointer; opacity: 0.4; transition: 0.3s;
}
.voice-item-compact:hover .v-delete { opacity: 1; background: rgba(239, 68, 68, 0.2); }
.v-delete:hover { background: #ef4444 !important; color: white !important; }

.voice-workspace {
  background: var(--panel);
  border-radius: 1.5rem !important;
  padding: 2rem !important;
  display: flex; flex-direction: column;
}

.welcome-workspace {
  flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center;
  text-align: center; color: #64748b;
}
.welcome-icon { font-size: 4rem; margin-bottom: 2rem; }
.btn-primary-large {
  margin-top: 2rem; background: var(--primary); border: none; color: white;
  padding: 1rem 2.5rem; border-radius: 1rem; font-weight: 800; cursor: pointer;
}

.workspace-header { margin-bottom: 2rem; border-bottom: 1px solid rgba(255,255,255,0.05); padding-bottom: 1.5rem; display: flex; justify-content: space-between; align-items: flex-end; }
.workspace-header h2 { font-size: 1.5rem; }

.sub-title { font-size: 0.8rem; color: var(--primary); text-transform: uppercase; margin-bottom: 1rem; }

.ref-files-grid {
  display: flex; flex-wrap: wrap; gap: 0.75rem; margin-bottom: 1.5rem;
}
.ref-file-tag {
  background: rgba(16, 185, 129, 0.1); border: 1px solid var(--primary);
  padding: 0.5rem 0.75rem; border-radius: 0.5rem; font-size: 0.75rem; display: flex; align-items: center; gap: 0.5rem;
}
.ref-file-tag button { background: none; border: none; color: #ef4444; cursor: pointer; }

.btn-sm-action {
  background: rgba(255,255,255,0.05); border: 1px solid rgba(255,255,255,0.1);
  color: white; padding: 0.5rem 1rem; border-radius: 0.5rem; font-size: 0.8rem; cursor: pointer; margin-right: 0.5rem;
}
.btn-sm-action.recording { background: #ef4444; }

.new-voice-name-box { margin-bottom: 2rem; }
.new-voice-name-box label { display: block; font-size: 0.8rem; margin-bottom: 0.5rem; color: #94a3b8; }
.glow-input-large {
  width: 100%; padding: 1rem; background: rgba(0,0,0,0.3); border: 1px solid #334155; border-radius: 1rem; color: white; font-size: 1.1rem;
}

.speed-config-mini { margin-bottom: 2rem; }
.speed-config-mini label { display: block; font-size: 0.8rem; margin-bottom: 0.8rem; }
.speed-presets { display: flex; gap: 0.5rem; }
.speed-presets button {
  background: rgba(255,255,255,0.05); border: 1px solid rgba(255,255,255,0.1); color: #94a3b8;
  padding: 0.3rem 0.8rem; border-radius: 0.5rem; font-size: 0.75rem; cursor: pointer;
}
.speed-presets button.active { background: var(--primary); color: white; border-color: var(--primary); }

.workspace-footer { display: flex; justify-content: space-between; align-items: center; margin-top: 2rem; }
.btn-save-voice, .btn-update-voice {
  background: #eab308; color: black; border: none; padding: 0.75rem 1.5rem; border-radius: 0.75rem; font-weight: 700; cursor: pointer;
}

.main-action-buttons { flex: 1; display: flex; justify-content: flex-end; }
.btn-clone-main {
  background: linear-gradient(135deg, #10b981 0%, #059669 100%);
  padding: 1rem 2.5rem; border: none; border-radius: 1rem; color: white; font-weight: 800; cursor: pointer;
}

.result-box-inline {
  margin-top: 2rem; padding: 1.5rem; border: 1px solid rgba(255,255,255,0.1); border-radius: 1rem; background: rgba(0,0,0,0.2);
}
.audio-actions { display: flex; gap: 1rem; margin-top: 1rem; }
.audio-actions button {
  background: #334155; border: none; color: white; padding: 0.5rem 1rem; border-radius: 0.5rem; cursor: pointer; font-size: 0.8rem;
}

.glow-select-small {
  padding: 0.4rem; background: rgba(0,0,0,0.3); border: 1px solid #334155; border-radius: 0.5rem; color: white; font-size: 0.8rem;
}

.w-full { width: 100%; }


.speed-presets {
  display: flex;
  gap: 0.5rem;
}
.btn-speed-preset {
  background: rgba(255,255,255,0.05);
  border: 1px solid rgba(255,255,255,0.1);
  color: #94a3b8;
  padding: 0.2rem 0.6rem;
  border-radius: 0.4rem;
  font-size: 0.7rem;
  cursor: pointer;
  transition: 0.2s;
}
.btn-speed-preset:hover, .btn-speed-preset.active {
  background: var(--primary);
  color: white;
  border-color: var(--primary);
}

.result-actions { display: flex; gap: 1rem; flex-wrap: wrap; }
.btn-save-result { background: #eab308; color: black; } 
.btn-save-result:hover { background: #facc15; }

/* Tabs */
.panel-header-tabs {
  display: flex;
  justify-content: center;
  margin-bottom: 2rem;
  border-bottom: 1px solid rgba(255,255,255,0.1);
  padding-bottom: 1rem;
}
.tab-group {
    background: rgba(0,0,0,0.3);
    padding: 0.3rem;
    border-radius: 0.8rem;
    display: flex;
    gap: 0.5rem;
}
.tab-btn {
    background: none;
    border: none;
    color: #94a3b8;
    padding: 0.5rem 1.5rem;
    border-radius: 0.6rem;
    cursor: pointer;
    font-weight: 600;
    transition: 0.3s;
}
.tab-btn.active {
    background: var(--primary);
    color: white;
    box-shadow: 0 4px 12px rgba(16, 185, 129, 0.2);
}
.text-truncate {
  white-space: nowrap; 
  overflow: hidden; 
  text-overflow: ellipsis; 
  max-width: 200px;
}
</style>
