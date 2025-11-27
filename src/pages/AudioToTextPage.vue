<template>
  <div class="page-content">
    <div class="page-header">
      <h2>Audio / Video to Text</h2>
    </div>
    <div class="page-body image-to-video-container">
      <!-- Layout 2 cột giống ImageToVideoPage -->
      <div class="image-to-video-layout">
        <!-- Cột trái: Chọn file và hiển thị kết quả -->
        <div class="image-to-video-left-column">
          <!-- File Selection Section -->
          <div class="file-group media-selection-group">
            <h3 class="form-label">Chọn file audio hoặc video</h3>
            <div class="file-selector">
              <button class="btn-select-file" @click="selectMediaFile">
                🎬 Chọn file audio/video
              </button>
              <button 
                v-if="mediaFile"
                class="btn-clear-all" 
                @click="clearMedia"
              >
                🗑️ Xóa File
              </button>
            </div>

            <!-- File Info -->
            <div v-if="mediaFile" class="file-info">
              <div class="file-item">
                <div class="file-item-info">
                  <span class="file-item-icon">
                    {{ getFileIcon(mediaFile) }}
                  </span>
                  <div class="file-item-details">
                    <span class="file-item-name">{{ getFileName(mediaFile) }}</span>
                    <span v-if="fileDuration" class="file-item-duration">
                      <span class="duration-icon">⏱️</span> {{ formatDuration(fileDuration, { showSeconds: true, roundUp: false }) }}
                    </span>
                    <span v-else class="file-item-duration loading">
                      Loading time...
                    </span>
                  </div>
                </div>
                <button class="file-item-remove" @click="clearMedia" title="Xóa">✕</button>
              </div>
            </div>
          </div>

          <!-- Result Section -->
          <div v-if="convertedText" class="file-group result-section">
            <div class="result-section-header">
              <h3 class="form-label">Kết quả</h3>
              <div class="result-actions">
                <button class="btn-select-file btn-copy" @click="copyToClipboard">
                  📋 Sao chép
                </button>
                <button class="btn-select-file btn-download" @click="downloadText">
                  💾 Tải xuống
                </button>
              </div>
            </div>
 
            <div class="form-group text-output-container">
              <div 
                class="text-output"
                :class="{ empty: !convertedText }"
              >
                <div v-if="!convertedText" class="text-placeholder">
                  Văn bản sẽ hiển thị ở đây...
                </div>
                <div v-else class="text-content">
                  {{ convertedText }}
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Cột phải: Convert button và Options -->
        <div class="image-to-video-right-column">
          <!-- Output Format Selection -->
          <div class="file-group">
            <div class="form-group">
              <label class="form-label">📄 Định dạng file output</label>
              <div class="output-format-list">
                <div 
                  v-for="format in outputFormats" 
                  :key="format.value"
                  :class="['format-option', { active: outputFormat === format.value }]"
                  @click="outputFormat = format.value"
                >
                  <span class="format-icon">{{ format.icon }}</span>
                  <span class="format-name">{{ format.name }}</span>
                  <span class="format-desc">{{ format.desc }}</span>
                  <span v-if="outputFormat === format.value" class="format-check">✓</span>
                </div>
              </div>
            </div>
          </div>

          <!-- Convert Button -->
          <div class="form-actions">
            <button 
              class="btn-create-video" 
              :disabled="!mediaFile || isConverting"
              @click="handleConvert"
            >
              {{ isConverting ? 'Đang chuyển đổi...' : `Chuyển đổi sang ${outputFormat.toUpperCase()}` }}
            </button>
          </div>

          <!-- Status message -->
          <div v-if="statusMessage" :class="['download-status', statusType]">
            <span>{{ statusMessage }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, watch, nextTick } from 'vue'
import { open, save } from '@tauri-apps/api/dialog'
import { invoke } from '@tauri-apps/api/tauri'
import { readTextFile, writeTextFile } from '@tauri-apps/api/fs'
import { useAudioDuration } from '@/composables/useAudioDuration'
import '@/assets/css/image-to-video.css'
import '@/assets/css/audio-to-text.css'

// Media file (single file only)
const mediaFile = ref(null)
const fileDuration = ref(null)

// Use composable for duration (works for both audio and video)
const { loadFileDuration } = useAudioDuration()

// Convert state
const isConverting = ref(false)
const convertedText = ref(`There are days when the smallest things feel strangely heavy, a single comment, a quick look from someone, an unexpected change in plans, nothing dramatic happens, and yet something inside tightens. A quiet unease you can't quite explain.
You go on with your day, but the mind keeps circling back.
Why did that bother me so much? Why did such a small thing feel so big? This is the part of life we rarely talk about, not the major crises, but the subtle moments that unsettle us for reasons we can't name.
A friend doesn't reply as quickly as usual.
Someone speaks in a tone that feels slightly off, a plan shifts, and suddenly your whole mood shifts with it.
These tiny disturbances seem random, but they aren't.
They reveal something deeper than the event itself.
They reveal the lens you're looking through.
Before the mind even understands what happened, reactions rise like waves, irritation, fear, defensiveness, insecurity.
It feels as if the world is pushing against you, but what if the pressure isn't coming from the world at all? What if it's coming from something inside? Something unseen.
The rudest wisdom calls this veil "Avija", not ignorance as in, not knowing facts, but a deeper blindness.
The inability to see things as they truly are.
Avija is subtle.
It doesn't announce itself.
It hides behind familiar thoughts and automatic emotions.
A person simply speaks and instantly, without any pause, the mind interprets.
They're judging me.
They don't care.
I said something wrong.
I need to defend myself.
But none of these reactions come from the moment itself.
They come from the stories we've been carrying, stories about who we think we are, what we fear, and what we're trying to protect.
The Buddha described this as living behind a veil, meeting life not as it is, but as we imagine it to be.
A word isn't just a word anymore.
It becomes a threat.
A change isn't just a change.
It becomes danger.
A feeling isn't just a feeling.
It becomes an identity.
And so we suffer.
Not because of what happens, but because of how we interpret it.
When you see clearly, that veil begins to thin.
And the world, instead of pressing in on you, becomes something you can meet with openness, steadiness, and quiet freedom.
When the Buddha spoke of seeing clearly, he wasn't talking about a special kind of mystical vision.
He was pointing to something profoundly simple, the ability to recognize three truths that quietly shape every moment of our lives.
These truths are not beliefs to memorize.
Their experiences to notice.
The first is anika impermanence.
We know intellectually that everything changes, but we don't live as if it does.
We expect people to stay the same.
We expect our emotions to stay steady.
We expect our routines, our roles, our sense of self to stay intact.
And when they shift, even by a little, we feel shaken.
We think something has gone wrong.
When in truth, things are simply following their nature.
Every joy, every frustration, every breath arises and dissolves.
When we don't see this, we cling.
And when we cling, change feels like loss.
The second is dukkha, the unsatisfactoriness that comes from holding on to what cannot stay.
It's like grasping a handful of water.
The harder you try to keep it, the faster it slips away.
We cling to praise, to relationships, to comfort, to ideas about who we should be.
And every time the world fails to match the picture in our minds, dissatisfaction appears.
Not because the world is unkind, but because the picture we hold is rigid, while life is fluid.
The third is anata, non-self.
This is the hardest to see, and the most liberating.
We move through life, protecting an imagined eye.
A fixed identity made of memories, opinions, fears and roles.
But when you look closely, you see that this eye is not solid.
It's a flowing stream of sensations, feelings, perceptions, thoughts, all changing, all moving.
Nothing you experience stays the same long enough to be called me.
These three distortions, impermanence, unsatisfactoriness and non-self are like filters shaping how we interpret reality.
When we forget them, even small moments can feel overwhelming.
But when we see them clearly, a quiet understanding dawns.
I don't need to fight life.
I only need to meet it as it is.
Most people believe emotions happen to them.
Something triggers us, and instantly we react, with anger, hurt, defensiveness, withdrawal.
It feels automatic, inevitable, but Buddhist wisdom invites us to look closer, to slow down the moment enough to see what's actually happening.
A word is spoken.
Before the meaning arrives, there is a sensation, a tightening in the chest, a heat rising in the face, a sudden shift in the breath.
Then a thought emerges.
They shouldn't have said that.
Then another, I need to protect myself.
Then an impulse, speak sharply, withdraw, shut down, and finally, the reaction.
We think it's one event.
It's actually a chain.
Clear seeing begins when you notice the space between each link in that chain.
A word, sensation, thought, impulse, choice.
That space is small at first, barely noticeable, like a thin thread of light.
But with awareness, it widens.
And inside that wideness is your freedom.
The freedom not to be swept away.
The freedom to respond instead of react.
The freedom to see that you are not your emotion.
Not your thought.
Not the story racing through your mind.
You are the one who is aware of them.
When you live from this awareness, nothing can truly overwhelm you.
Not because life stops being difficult, but because you are no longer lost in the storm.
You're standing in the open sky where storms come and go.
In the Salatha Sutta, the Buddha offered one of the simplest and most compassionate teachings on human suffering.
He said that when an ordinary person encounters pain, they are struck by two arrows.
The first arrow is unavoidable.
It is the raw pain of being human.
The ache of loss.
The sting of disappointment.
The physical or emotional discomfort that comes with living in a world that changes moment by moment.
This pain is clean.
Pure.
Real.
It hurts.
But it does not destroy.
The second arrow, however, is the one we fire into ourselves.
It's the story we build around the pain.
Why me? This shouldn't have happened.
I can't handle this.
I must have done something wrong.
We turn the event into an identity, the moment into a lifelong sentence.
And the second arrow, not the first, is what causes most of our suffering.
Think of a relationship ending.
The first arrow is grief, missing someone you once held close.
But the second arrow says, "I wasn't enough.
No one will love me again.
" I'm broken.
One is pain.
The other is interpretation.
And interpretation can feel infinite.
Clear seeing doesn't remove the first arrow.
It removes the second.
It allows you to say, "Yes, this hurts.
" But it doesn't have to become a story about who I am.
Yes, sadness is here.
But I don't have to fight it or build a prison around it.
When you meet pain without adding layers, something shifts.
The ache remains.
But the suffering dissolves.
You feel the wound.
But you also feel a quiet strength beneath it.
This is the beginning of freedom, not escaping pain, but no longer turning it into a self-inflicted wound.
In the Angutara Nikaia, the Buddha spoke of the eight worldly winds.
Gain and loss, praise and blame, pleasure and pain, fame and disrepute.
He said that most people are blown around by these winds without realizing it.
A compliment lifts us.
A criticism crushes us.
A success makes us confident.
A setback makes us doubt everything.
Why do these simple shifts have such power? Because they touch the thing we cling to most tightly.
Identity deep down we believe in a solid eye.
A fixed self that must be protected, polished, proven.
So when someone praises us, the identity expands.
I am worthy.
When someone criticizes us, the identity contracts.
I am failing.
But this identity is fragile because it isn't real.
It's a construction made of memories, fears, expectations and roles we've collected over years.
We suffer because we're trying to defend something that does not actually exist in the way we imagine.
Clear seeing dissolves this prison.
When praise arises, you notice there is a pleasant feeling and a thought saying, "I'm good.
" When blame arises, you notice there is discomfort and a thought saying, "I'm bad.
" The thoughts are weather, you are the sky.
When you recognize this, the winds no longer control you.
Gain and loss still happen.
Praise and blame still appear.
Pleasure and pain still move through your life, but they no longer define you.
You no longer cling to the highs or fear the lows.
You meet everything with steadiness because the eye you were trying to protect has softened, loosened, opened into something spacious and free.
This is the heart of Buddhist wisdom.
When the self is no longer rigid, the world can no longer shake you.
Most people believe that their thoughts are the truth.
If the mind says, "I'm not good enough," they take it as a fact.
If the mind says, "Everyone is judging me, they feel certain it must be real.
" If the mind says, "This will never get better," the future collapses into fear.
But Buddhist wisdom invites a different way of relating to the mind, not by trying to control it, silence it, or argue with it, but by seeing it.
Thoughts are not truths.
They are events.
In the satirpaterna teachings, the Buddha describes thoughts as phenomena that arise and pass away, just like clouds forming and dissolving in the sky.
They appear due to causes and conditions, mood, memory, habit, stress, and then they fade.
But because we identify with them, we mistake them for me.
A thought appears.
I'm failing.
We don't see it as a passing cloud.
We wear it like a nametag, an emotion arises, sadness, anger, anxiety.
We don't feel it as a sensation.
We interpret it as a story about who we are.
Clear seeing is the practice of meeting thoughts the way you'd observe weather.
With presence, without clinging, without fear.
When a negative thought appears instead of being swept into it, you notice, here is a thought.
When an emotion tightens the chest, instead of creating a narrative around it, you observe, here is a feeling.
The shift is small, but profound.
You move from being inside the thought to being aware of the thought.
You move from I am anxious to anxiety is present, from I am angry to heat is rising in the body.
And slowly you realize something extraordinary.
If you can observe a thought, it cannot be who you are.
If you can witness an emotion, it cannot define you.
You become the open sky, vast, spacious, untouched.
While thoughts and emotions come and go like passing weather.
This is not detachment, this is freedom.
Clear seeing is not something you only practice during meditation.
It shows itself in the quiet, ordinary moments.
The ones we often rush past without noticing, when walking, know you are walking.
The pressure of the feet, the shifting of weight, the breeze touching your skin.
This simple knowing dissolves the veil of autopilot, when eating, know you are eating.
Taste, texture, movement.
Instead of scrolling or thinking ahead, you return to the miracle of nourishment, something that has always been there, unnoticed.
When working, know you are working.
Notice the posture of the body, the rise of stress.
The softening that comes from one conscious breath.
Awareness dissolves overwhelm long before it becomes suffering.
In the satipata-nessuta, the Buddha described mindfulness of body, feelings, mind and phenomena, not as a ritual, but as a way of living fully awake.
When you are washing dishes, you are not getting chores done.
You are feeling warm water against your skin, hearing the soft clink of plates, watching the breath move in and out ordinary moments bloom with presence.
This is the quiet miracle.
Life doesn't need to change for clarity to arise.
You don't need a retreat, a perfect environment or hours of meditation.
You only need to pause long enough to notice what is already here.
Clear seeing grows each time you choose presence over hurry, awareness over habit.
And in those small moments, the world softens, your mind softens, your heart softens, the veil thins, not through effort, but through attention.
And the life you thought you were merely surviving becomes something you can finally inhabit, moment by moment with ease, gratitude and quiet joy.
There are moments in life when clarity feels impossible, when the ground beneath you cracks, when someone you love walks away, when plans collapse, when the future becomes a blur of uncertainty.
In these moments, the mind doesn't whisper, it spirals, it tells you that you are alone, that you have failed, that things will never be whole again.
Clear seeing doesn't deny the weight of these experiences, it doesn't ask you to pretend you are okay, it doesn't numb or erase the ache of being human.
Instead, it offers something gentler, the ability to stay open even when everything hurts, to feel grief without turning it into self-blame, to feel fear without becoming the fear, to feel sadness without collapsing into the story that something is wrong with you.
When the heart breaks, the first arrow is real, the loss, the shock, the emptiness, but the second arrow, I'm not enough, I'll never recover, my life is ruined, that one is optional.
Clear seeing removes that arrow gently without force, it allows you to hold heartbreak the way you'd hold something fragile, carefully, lovingly, without turning away.
It lets you say this hurts, and that's okay, it helps you recognize that pain is not a punishment, it's a sign that you care deeply.
When things fall apart, clarity doesn't close the wound, it keeps the wound from becoming your identity.
You feel the waves but you no longer drown in the stories about them, and slowly a quiet strength appears, not the strength of armour but the strength of openness.
If you've listened this far, then something in you is already shifting, already seeing.
Life will always move in waves, joy and sorrow, gain and loss, praise and blame.
You can't control the ocean, but you can learn to see clearly, and in that clarity, you discover something profound, you were never the waves, you were always the ocean.
The waves will rise, they will fall, some will feel gentle, others overwhelming, but the ocean remains vast, steady, unbroken, the space that holds everything without being harmed by anything.
When you remember this, nothing truly disturbs you, not because life becomes easier, but because you are no longer fighting its nature.
Thank you for being here, for listening, for opening your heart even a little.
If this wisdom touched you, consider subscribing for more Buddhist reflections, stories and reminders to return home to yourself.
May you see clearly, may you be free.`)
const statusMessage = ref('')
const statusType = ref('info')

// Output options
const outputFormat = ref('txt')
const outputFormats = [
  { value: 'txt', name: 'TXT', desc: 'Văn bản', icon: '📄' },
  { value: 'srt', name: 'SRT', desc: 'Caption đơn giản', icon: '🎬' },
  { value: 'ass', name: 'ASS', desc: 'Caption nâng cao', icon: '💬' },
]

const getFileName = (filePath) => {
  return filePath.split('/').pop() || filePath.split('\\').pop() || filePath
}

const getFileIcon = (filePath) => {
  const ext = filePath.split('.').pop()?.toLowerCase()
  if (['mp3', 'wav', 'm4a', 'aac', 'ogg', 'flac'].includes(ext)) {
    return '🎵'
  } else if (['mp4', 'avi', 'mov', 'mkv', 'webm', 'flv'].includes(ext)) {
    return '🎬'
  }
  return '📄'
}

/**
 * Format duration từ giây sang định dạng dễ đọc
 */
const formatDuration = (seconds, options = {}) => {
  const { showSeconds = true, roundUp = false } = options
  
  if (!seconds || isNaN(seconds)) {
    return showSeconds ? '0 phút 0 giây' : '0 phút'
  }
  
  const totalMins = Math.floor(seconds / 60)
  const secs = Math.floor(seconds % 60)
  
  // Xử lý >= 60 phút (hiển thị giờ)
  if (totalMins >= 60) {
    const hours = Math.floor(totalMins / 60)
    const mins = totalMins % 60
    
    if (mins === 0) {
      return `${hours} giờ`
    } else {
      return `${hours} giờ ${mins} phút`
    }
  }
  
  // Xử lý < 60 phút
  if (roundUp && secs > 0) {
    return `${totalMins + 1} phút`
  }
  
  if (secs === 0) {
    return `${totalMins} phút`
  } else {
    return showSeconds 
      ? `${totalMins} phút ${secs} giây`
      : `${totalMins} phút`
  }
}

const selectMediaFile = async () => {
  try {
    const selected = await open({
      multiple: false,
      filters: [{
        name: 'Media Files',
        extensions: ['mp3', 'wav', 'm4a', 'aac', 'ogg', 'flac', 'mp4', 'avi', 'mov', 'mkv', 'webm', 'flv']
      }]
    })
    
    if (selected) {
      const file = Array.isArray(selected) ? selected[0] : selected
      mediaFile.value = file
      fileDuration.value = null
      convertedText.value = ''
      
      // Load duration cho file
      try {
        const duration = await loadFileDuration(file)
        fileDuration.value = duration
      } catch (error) {
        console.error('Lỗi khi load duration:', error)
      }
      
      statusMessage.value = '✅ Đã chọn file'
      statusType.value = 'success'
    }
  } catch (error) {
    console.error('Lỗi khi chọn file:', error)
    statusMessage.value = '❌ Lỗi khi chọn file: ' + error
    statusType.value = 'error'
  }
}

const clearMedia = () => {
  mediaFile.value = null
  fileDuration.value = null
  convertedText.value = ''
  statusMessage.value = '✅ Đã xóa file'
  statusType.value = 'success'
}

const handleConvert = async () => {
  if (!mediaFile.value) return
  
  isConverting.value = true
  convertedText.value = ''
  statusMessage.value = `⏳ Đang chuyển đổi audio sang ${outputFormat.value.toUpperCase()}...`
  statusType.value = 'info'
  
  try {
    // Gọi Tauri command tương ứng với format
    let outputPath
    if (outputFormat.value === 'ass') {
      outputPath = await invoke('convert_audio_to_ass', {
        inputPath: mediaFile.value,
        outputAssPath: null // Để tự động tạo tên file
      })
    } else if (outputFormat.value === 'txt') {
      outputPath = await invoke('convert_audio_to_txt', {
        inputPath: mediaFile.value,
        outputTxtPath: null // Để tự động tạo tên file
      })
    } else {
      // TODO: Implement other formats (SRT, VTT, etc.)
      throw new Error(`Định dạng ${outputFormat.value.toUpperCase()} sẽ được implement trong phiên bản tiếp theo`)
    }
    
    // Đọc file đã tạo
    const content = await readTextFile(outputPath)
    convertedText.value = content
    
    statusMessage.value = `✅ Chuyển đổi hoàn tất! File ${outputFormat.value.toUpperCase()} đã được tạo tại: ${outputPath}`
    statusType.value = 'success'
  } catch (error) {
    console.error('Lỗi khi chuyển đổi:', error)
    statusMessage.value = `❌ Có lỗi xảy ra khi chuyển đổi: ${error}`
    statusType.value = 'error'
  } finally {
    isConverting.value = false
  }
}

const copyToClipboard = async () => {
  if (!convertedText.value) return
  
  try {
    await navigator.clipboard.writeText(convertedText.value)
    statusMessage.value = '✅ Đã sao chép vào clipboard!'
    statusType.value = 'success'
  } catch (error) {
    console.error('Lỗi khi sao chép:', error)
    statusMessage.value = '❌ Lỗi khi sao chép'
    statusType.value = 'error'
  }
}

const downloadText = async () => {
  if (!convertedText.value) return
  
  try {
    // Tạo tên file mặc định từ file input
    let defaultName = `caption.${outputFormat.value}`
    if (mediaFile.value) {
      const inputName = getFileName(mediaFile.value)
      const baseName = inputName.replace(/\.[^/.]+$/, '') // Bỏ extension
      defaultName = `${baseName}_caption.${outputFormat.value}`
    }
    
    // Tạo filter tương ứng với format
    const formatFilters = {
      txt: { name: 'Text File', extensions: ['txt'] },
      srt: { name: 'SRT Subtitle', extensions: ['srt'] },
      ass: { name: 'ASS Subtitle', extensions: ['ass'] },
      vtt: { name: 'WebVTT Subtitle', extensions: ['vtt'] },
      json: { name: 'JSON File', extensions: ['json'] }
    }
    
    // Mở dialog để chọn nơi lưu file
    const savePath = await save({
      defaultPath: defaultName,
      filters: [
        formatFilters[outputFormat.value] || { name: 'Text File', extensions: ['txt'] },
        { name: 'All Files', extensions: ['*'] }
      ]
    })
    
    if (savePath) {
      // Ghi nội dung vào file
      await writeTextFile(savePath, convertedText.value)
      
      statusMessage.value = `✅ Đã lưu file thành công: ${savePath}`
      statusType.value = 'success'
    }
  } catch (error) {
    console.error('Lỗi khi lưu file:', error)
    statusMessage.value = `❌ Lỗi khi lưu file: ${error}`
    statusType.value = 'error'
  }
}

</script>

