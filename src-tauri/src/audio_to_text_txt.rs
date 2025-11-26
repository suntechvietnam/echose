use whisper_rs::{WhisperContext, WhisperContextParameters, FullParams, SamplingStrategy};
use std::fs::File;
use std::io::Write;
use anyhow::Result;
use crate::utils::find_ffmpeg_by_os::run_ffmpeg_sync;

/// Segment chứa thông tin timestamp và text từ Whisper
#[derive(Debug, Clone)]
pub struct TranscriptSegment {
    pub start: f64,  // seconds
    pub end: f64,    // seconds
    pub text: String,
}

/// Cấu hình cho việc chia nhỏ segments
#[derive(Debug, Clone)]
pub struct SplitConfig {
    pub max_words_per_segment: usize,  // Số từ tối đa mỗi segment
    pub min_segment_duration: f64,     // Thời gian tối thiểu mỗi segment (giây)
    pub enable_splitting: bool,        // Bật/tắt tính năng chia segment
}

impl Default for SplitConfig {
    fn default() -> Self {
        Self {
            max_words_per_segment: 6,  // 6 từ per segment
            min_segment_duration: 1.0, // Ít nhất 1 giây
            enable_splitting: true,    // Mặc định bật
        }
    }
}

/// Chia segment thành các segment nhỏ hơn theo số từ
fn split_segment_by_words(segment: &TranscriptSegment, config: &SplitConfig) -> Vec<TranscriptSegment> {
    if !config.enable_splitting {
        return vec![segment.clone()];
    }
    
    let words: Vec<&str> = segment.text.split_whitespace().collect();
    
    // Nếu số từ <= max, không cần chia
    if words.len() <= config.max_words_per_segment {
        return vec![segment.clone()];
    }
    
    let mut result = Vec::new();
    let total_duration = segment.end - segment.start;
    let word_count = words.len();
    
    // Chia đều thời gian cho mỗi từ
    let duration_per_word = total_duration / word_count as f64;
    
    let mut current_start = segment.start;
    
    for chunk in words.chunks(config.max_words_per_segment) {
        let chunk_word_count = chunk.len();
        let chunk_duration = (chunk_word_count as f64 * duration_per_word)
            .max(config.min_segment_duration); // Đảm bảo thời gian tối thiểu
        
        let chunk_end = (current_start + chunk_duration).min(segment.end);
        let chunk_text = chunk.join(" ");
        
        result.push(TranscriptSegment {
            start: current_start,
            end: chunk_end,
            text: chunk_text,
        });
        
        current_start = chunk_end;
        
        // Nếu đã đến cuối segment gốc, dừng
        if current_start >= segment.end {
            break;
        }
    }
    
    // Điều chỉnh segment cuối để kết thúc đúng lúc
    if let Some(last_segment) = result.last_mut() {
        last_segment.end = segment.end;
    }
    
    result
}

/// Áp dụng splitting cho tất cả segments
fn apply_segment_splitting(segments: &[TranscriptSegment], config: &SplitConfig) -> Vec<TranscriptSegment> {
    let mut result = Vec::new();
    
    for segment in segments {
        let split_segments = split_segment_by_words(segment, config);
        result.extend(split_segments);
    }
    
    result
}

/// Tìm đường dẫn đến Whisper model
fn find_whisper_model(model_name: &str) -> Option<String> {
    use std::path::PathBuf;
    
    // Nếu là đường dẫn tuyệt đối và tồn tại, dùng luôn
    let path = PathBuf::from(model_name);
    if path.is_absolute() && path.exists() {
        return Some(model_name.to_string());
    }
    
    println!("🔍 Searching for model: {}", model_name);
    
    // Tìm trong các vị trí khác nhau
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(app_dir) = exe_path.parent() {
            println!("📁 App directory: {}", app_dir.display());
            
            #[cfg(target_os = "macos")]
            {
                // macOS: Tìm trong app bundle Resources/
                if let Some(contents_dir) = app_dir.parent() {
                    let resources_dir = contents_dir.join("Resources");
                    println!("📁 Checking macOS Resources: {}", resources_dir.display());
                    
                    let paths = vec![
                        resources_dir.join("models").join(model_name),
                        resources_dir.join(model_name),
                    ];
                    
                    for p in paths {
                        println!("   Trying: {}", p.display());
                        if p.exists() {
                            println!("✅ Found model at: {}", p.display());
                            return Some(p.to_string_lossy().to_string());
                        }
                    }
                }
            }
            
            #[cfg(target_os = "windows")]
            {
                // Windows: Tìm trong cùng thư mục exe và bundled resources
                println!("📁 Checking Windows app directory and bundled resources");
                
                let paths = vec![
                    // Bundled resources (từ tauri.conf.json)
                    app_dir.join("models").join(model_name),
                    app_dir.join(model_name),
                    // Backup locations  
                    app_dir.join("resources").join("models").join(model_name),
                    app_dir.join("_up_").join("models").join(model_name), // Some bundlers use _up_
                ];
                
                for p in paths {
                    println!("   Trying: {}", p.display());
                    if p.exists() {
                        println!("✅ Found model at: {}", p.display());
                        return Some(p.to_string_lossy().to_string());
                    }
                }
            }
            
            #[cfg(not(any(target_os = "macos", target_os = "windows")))]
            {
                // Linux và các platform khác
                let paths = vec![
                    app_dir.join("models").join(model_name),
                    app_dir.join(model_name),
                ];
                
                for p in paths {
                    println!("   Trying: {}", p.display());
                    if p.exists() {
                        println!("✅ Found model at: {}", p.display());
                        return Some(p.to_string_lossy().to_string());
                    }
                }
            }
        }
    }
    
    // Thử đường dẫn tương đối từ working directory (development mode)
    println!("📁 Checking development paths from working directory");
    let paths = vec![
        PathBuf::from("models").join(model_name),
        PathBuf::from("src-tauri").join("models").join(model_name),
        PathBuf::from(model_name),
    ];
    
    for p in paths {
        println!("   Trying: {}", p.display());
        if p.exists() {
            println!("✅ Found model at: {}", p.display());
            return Some(p.to_string_lossy().to_string());
        }
    }
    
    println!("❌ Model '{}' not found in any location", model_name);
    None
}

/// Chuyển file audio/video → file .txt với nội dung văn bản thuần túy
/// Sử dụng whisper-rs (bindings cho whisper.cpp)
pub fn audio_to_txt(
    input_path: &str,
    output_txt_path: Option<&str>,
    model_path: Option<&str>,
    language: Option<&str>,
) -> Result<String> {
    // Đường dẫn mặc định cho model (whisper.cpp format .bin)
    let model_name = model_path.unwrap_or("ggml-base.bin");
    
    println!("🎤 Đang tìm mô hình Whisper...");
    
    // Tìm model base
    let model = find_whisper_model(model_name).ok_or_else(|| {
        anyhow::anyhow!(
            "Không tìm thấy file model: {}\n\
            Đã tìm trong: models/, src-tauri/models/, và thư mục app.\n\
            Vui lòng tải model từ: https://huggingface.co/ggerganov/whisper.cpp/tree/main\n\
            Ví dụ: ggml-base.bin, ggml-small.bin, ggml-medium.bin",
            model_name
        )
    })?;
    
    println!("📁 Model found: {}", model);
    
    // Tạo WhisperContext
    let ctx = WhisperContext::new_with_params(
        &model,
        WhisperContextParameters::default()
    ).map_err(|e| anyhow::anyhow!("Không thể tải model Whisper: {}", e))?;

    // Chuyển đổi audio sang WAV 16kHz mono nếu cần
    let wav_path = ensure_wav_format(input_path)?;
    
    println!("🔊 Đang đọc audio: {}", wav_path);
    
    // Đọc WAV file thành f32 samples
    let audio_data = read_wav_to_f32(&wav_path)?;
    
    println!("⏱️ Audio length: {:.2}s ({} samples)", 
        audio_data.len() as f64 / 16000.0, 
        audio_data.len()
    );
    
    // Thiết lập parameters
    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    
    // Cấu hình ngôn ngữ - auto detect nếu không được chỉ định
    if language.is_some() {
        let lang = language.unwrap();
        params.set_language(Some(lang));
        println!("🌐 Sử dụng ngôn ngữ: {}", lang);
    } else {
        params.set_language(None); // Auto-detect language
        println!("🔍 Auto-detecting language...");
    }
    params.set_translate(false); // Không dịch, giữ nguyên ngôn ngữ gốc
    params.set_print_special(false);
    params.set_print_progress(true);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);
    
    // Tạo state và chạy transcription
    if language.is_some() {
        println!("🚀 Đang transcribe (ngôn ngữ: {})...", language.unwrap());
    } else {
        println!("🚀 Đang transcribe (auto-detect language)...");
    }
    let mut state = ctx.create_state()
        .map_err(|e| anyhow::anyhow!("Không thể tạo Whisper state: {}", e))?;
    
    state.full(params, &audio_data)
        .map_err(|e| anyhow::anyhow!("Lỗi khi transcribe: {}", e))?;
    
    // Lấy kết quả với timestamps - whisper-rs 0.15 API
    let num_segments = state.full_n_segments(); // Returns i32 directly
    
    println!("✅ Transcription hoàn tất! {} segments", num_segments);
    
    let mut segments: Vec<TranscriptSegment> = Vec::new();
    
    for i in 0..num_segments {
        // whisper-rs 0.15 API: get_segment returns Option<WhisperSegment>
        if let Some(segment) = state.get_segment(i) {
            // Get text (lossy for non-UTF8 compatibility)
            let text = segment.to_str_lossy()
                .map(|s| s.to_string())
                .unwrap_or_default();
            
            // Timestamps in centiseconds (10ms units)
            let start_cs = segment.start_timestamp();
            let end_cs = segment.end_timestamp();
            
            // Convert từ centiseconds sang seconds
            let start = start_cs as f64 * 0.01;
            let end = end_cs as f64 * 0.01;
            
            segments.push(TranscriptSegment {
                start,
                end,
                text: text.trim().to_string(),
            });
        }
    }
    
    // Tạo đường dẫn output .txt
    let final_output = output_txt_path.map(|s| s.to_string()).unwrap_or_else(|| {
        input_path.rsplit_once('.').map(|(name, _)| format!("{}_transcript.txt", name))
            .unwrap_or_else(|| "audio_transcript.txt".to_string())
    });

    // Áp dụng segment splitting (chia nhỏ theo số từ)
    let split_config = SplitConfig::default();
    let final_segments = apply_segment_splitting(&segments, &split_config);
    
    println!("🔀 Segment splitting: {} -> {} segments", segments.len(), final_segments.len());
    
    // Xuất TXT với nội dung văn bản thuần túy
    export_segments_to_txt_file(&final_segments, &final_output)?;

    println!("🎉 HOÀN TẤT! File TXT đã tạo:");
    println!("→ {}", final_output);

    // Xóa file WAV tạm nếu đã tạo
    if wav_path != input_path {
        let _ = std::fs::remove_file(&wav_path);
    }

    Ok(final_output)
}

/// Đọc WAV file và convert sang f32 samples (16kHz mono)
fn read_wav_to_f32(wav_path: &str) -> Result<Vec<f32>> {
    let reader = hound::WavReader::open(wav_path)
        .map_err(|e| anyhow::anyhow!("Không thể đọc WAV file: {}", e))?;
    
    let spec = reader.spec();
    
    // Whisper yêu cầu 16kHz mono
    if spec.sample_rate != 16000 {
        return Err(anyhow::anyhow!(
            "WAV file phải có sample rate 16000Hz, hiện tại: {}Hz", 
            spec.sample_rate
        ));
    }
    
    let samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Int => {
            let max_value = (1 << (spec.bits_per_sample - 1)) as f32;
            reader.into_samples::<i32>()
                .filter_map(|s| s.ok())
                .map(|s| s as f32 / max_value)
                .collect()
        },
        hound::SampleFormat::Float => {
            reader.into_samples::<f32>()
                .filter_map(|s| s.ok())
                .collect()
        },
    };
    
    // Nếu stereo, convert sang mono bằng cách lấy trung bình
    if spec.channels == 2 {
        let mono: Vec<f32> = samples.chunks(2)
            .map(|chunk| (chunk[0] + chunk.get(1).unwrap_or(&0.0)) / 2.0)
            .collect();
        Ok(mono)
    } else {
        Ok(samples)
    }
}

/// Đảm bảo file audio ở định dạng WAV 16kHz mono
fn ensure_wav_format(input_path: &str) -> Result<String> {
    // Luôn convert để đảm bảo format đúng (16kHz, mono, 16-bit)
    let temp_wav = format!("{}.whisper.wav", input_path);
    
    println!("🔄 Converting audio to WAV 16kHz mono...");
    
    // Sử dụng run_ffmpeg_sync() từ utils
    let output = run_ffmpeg_sync()
        .map_err(|e| anyhow::anyhow!("{}", e))?
        .arg("-i")
        .arg(input_path)
        .arg("-ar")
        .arg("16000")      // 16kHz sample rate
        .arg("-ac")
        .arg("1")          // mono
        .arg("-c:a")
        .arg("pcm_s16le")  // 16-bit PCM
        .arg("-y")         // Overwrite output file
        .arg(&temp_wav)
        .output()
        .map_err(|e| anyhow::anyhow!("Không thể chạy ffmpeg: {}", e))?;
    
    if output.status.success() {
        println!("✅ Audio converted successfully");
        Ok(temp_wav)
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        Err(anyhow::anyhow!("FFmpeg conversion failed: {}", error))
    }
}

/// Chuyển đổi segments thành nội dung TXT file (văn bản thuần túy)
pub fn segments_to_txt_content(segments: &[TranscriptSegment]) -> String {
    let mut content = String::new();
    
    for (i, segment) in segments.iter().enumerate() {
        if segment.text.trim().is_empty() {
            continue;
        }
        
        // Thêm text của segment
        content.push_str(&segment.text);
        
        // Thêm xuống dòng giữa các segments (trừ segment cuối)
        if i < segments.len() - 1 {
            content.push('\n');
        }
    }
    
    content
}

/// Xuất TXT file với nội dung văn bản thuần túy
pub fn export_segments_to_txt_file(
    segments: &[TranscriptSegment],
    output_path: &str,
) -> Result<()> {
    let content = segments_to_txt_content(segments);
    
    let mut file = File::create(output_path)?;
    file.write_all(content.as_bytes())?;
    
    Ok(())
}

/// Tauri command: Chuyển file audio/video → file .txt
#[tauri::command]
pub async fn convert_audio_to_txt(
    input_path: String,
    output_txt_path: Option<String>,
    model_path: Option<String>,
    language: Option<String>,
) -> Result<String, String> {
    // Chạy trong blocking thread vì whisper processing nặng
    tokio::task::spawn_blocking(move || {
        audio_to_txt(
            &input_path, 
            output_txt_path.as_deref(), 
            model_path.as_deref(),
            language.as_deref(),
        )
    })
    .await
    .map_err(|e| format!("Task error: {}", e))?
    .map_err(|e| format!("Lỗi khi chuyển đổi: {}", e))
}

/// Tauri command: Chuyển segments thành nội dung TXT string (không lưu file)
/// Hữu ích cho preview hoặc return content trực tiếp
#[tauri::command]
pub async fn segments_to_txt_string(
    segments: Vec<(f64, f64, String)>, // (start, end, text) tuples
) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        // Convert tuples to TranscriptSegment
        let transcript_segments: Vec<TranscriptSegment> = segments
            .into_iter()
            .map(|(start, end, text)| TranscriptSegment { start, end, text })
            .collect();

        // Tạo nội dung TXT
        Ok(segments_to_txt_content(&transcript_segments))
    })
    .await
    .map_err(|e| format!("Task error: {}", e))?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_segments_to_txt_content() {
        let segments = vec![
            TranscriptSegment {
                start: 0.0,
                end: 2.0,
                text: "Hello world".to_string(),
            },
            TranscriptSegment {
                start: 2.0,
                end: 4.0,
                text: "This is a test".to_string(),
            },
        ];
        
        let content = segments_to_txt_content(&segments);
        assert_eq!(content, "Hello world\nThis is a test");
    }

    #[test]
    fn test_split_segment_by_words() {
        let segment = TranscriptSegment {
            start: 0.0,
            end: 10.0,
            text: "This is a very long sentence with many words".to_string(),
        };
        
        let config = SplitConfig::default();
        let result = split_segment_by_words(&segment, &config);
        
        // Với 8 từ và max 6 từ per segment, sẽ có 2 segments
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].text, "This is a very long sentence");
        assert_eq!(result[1].text, "with many words");
    }
}