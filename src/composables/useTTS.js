import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { convertFileSrc } from '@tauri-apps/api/core'

export function useTTS() {
    const isGenerating = ref(false)
    const lastGeneratedPath = ref(null)
    const lastAudioUrl = ref(null)

    const generateAudio = async (text, voiceConfig, options = {}) => {
        const {
            pitch = 0,
            rate = 0,
            volume = 0,
            bass = 0,
            treble = 0,
            outputFolder = null,
            provider = 'edge',
            apiKey = ''
        } = options

        if (!text || !voiceConfig) {
            throw new Error('Thiếu thông tin (văn bản hoặc giọng nói)')
        }

        isGenerating.value = true
        try {
            // Nếu là giọng nhân bản (Custom)
            if (voiceConfig.isCustom) {
                console.log('🎙️ Using custom cloned voice:', voiceConfig.name);

                // Chuẩn bị đường dẫn đầu ra
                const filename = `cloned_${Date.now()}.wav`;
                const finalOutputPath = outputFolder
                    ? `${outputFolder}/${filename}`
                    : `/tmp/${filename}`;

                const filePath = await invoke('clone_voice_metavoice', {
                    text,
                    referenceAudioPath: voiceConfig.paths ? voiceConfig.paths.join(',') : voiceConfig.referencePath,
                    outputPath: finalOutputPath,
                    lang: options.lang || voiceConfig.lang || 'en',
                    speed: options.readingSpeed || 0.9
                });

                lastGeneratedPath.value = filePath;
                lastAudioUrl.value = convertFileSrc(filePath);
                return { filePath, audioUrl: lastAudioUrl.value };
            }

            // Nếu là giọng chuẩn (Edge, OpenAI)
            const pitchStr = pitch >= 0 ? `+${pitch}Hz` : `${pitch}Hz`
            const rateStr = rate >= 0 ? `+${rate}%` : `${rate}%`
            const volumeStr = volume >= 0 ? `+${volume}%` : `${volume}%`

            const filePath = await invoke('generate_tts', {
                provider,
                apiKey,
                text,
                voice: voiceConfig.voice,
                pitch: pitchStr,
                rate: rateStr,
                volume: volumeStr,
                bass: parseInt(bass),
                treble: parseInt(treble),
                outputFolder,
                referenceAudioPath: voiceConfig.referencePath || null
            })

            lastGeneratedPath.value = filePath
            lastAudioUrl.value = filePath.startsWith('http') ? filePath : convertFileSrc(filePath)
            return { filePath, audioUrl: lastAudioUrl.value }
        } catch (err) {
            console.error('useTTS Error:', err)
            throw err
        } finally {
            isGenerating.value = false
        }
    }

    return {
        isGenerating,
        lastGeneratedPath,
        lastAudioUrl,
        generateAudio
    }
}
