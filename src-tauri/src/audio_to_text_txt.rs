use whisper_rs::{WhisperContext, WhisperContextParameters, FullParams, SamplingStrategy};
use std::fs::File;
use std::io::Write;
use anyhow::Result;
use crate::utils::find_ffmpeg_by_os::run_ffmpeg_sync;

/// Segment chứa thông tin timestamp và text từ Whisper
#[derive(Debug, Clone)]
pub struct TranscriptSegment {
    #[allow(dead_code)]
    pub start: f64,  // seconds
    #[allow(dead_code)]
    pub end: f64,    // seconds
    pub text: String,
}

/// Tìm đường dẫn đến Whisper model
fn find_whisper_model(model_name: &str) -> Option<String> {
    use std::path::PathBuf;
    
    // Nếu là đường dẫn tuyệt đối và tồn tại, dùng luôn
    let path = PathBuf::from(model_name);
    if path.is_absolute() && path.exists() {
        return Some(model_name.to_string());
    }
    
    // Tìm trong các vị trí khác nhau
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(app_dir) = exe_path.parent() {
            
            #[cfg(target_os = "macos")]
            {
                // macOS: Tìm trong app bundle Resources/
                if let Some(contents_dir) = app_dir.parent() {
                    let resources_dir = contents_dir.join("Resources");
                    
                    let paths = vec![
                        resources_dir.join("models").join(model_name),
                        resources_dir.join(model_name),
                    ];
                    
                    for p in paths {
                        if p.exists() {
                            return Some(p.to_string_lossy().to_string());
                        }
                    }
                }
            }
            
            #[cfg(target_os = "windows")]
            {
                // Windows: Tìm trong cùng thư mục exe và bundled resources
                let paths = vec![
                    // Bundled resources (từ tauri.conf.json)
                    app_dir.join("models").join(model_name),
                    app_dir.join(model_name),
                    // Backup locations  
                    app_dir.join("resources").join("models").join(model_name),
                    app_dir.join("_up_").join("models").join(model_name), // Some bundlers use _up_
                ];
                
                for p in paths {
                    if p.exists() {
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
                    if p.exists() {
                        return Some(p.to_string_lossy().to_string());
                    }
                }
            }
        }
    }
    
    // Thử đường dẫn tương đối từ working directory (development mode)
    let paths = vec![
        PathBuf::from("models").join(model_name),
        PathBuf::from("src-tauri").join("models").join(model_name),
        PathBuf::from(model_name),
    ];
    
    for p in paths {
        if p.exists() {
            return Some(p.to_string_lossy().to_string());
        }
    }
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
    
    // Tạo WhisperContext
    let ctx = WhisperContext::new_with_params(
        &model,
        WhisperContextParameters::default()
    ).map_err(|e| anyhow::anyhow!("Không thể tải model Whisper: {}", e))?;

    // Chuyển đổi audio sang WAV 16kHz mono nếu cần
    let wav_path = ensure_wav_format(input_path)?;
    
    // Đọc WAV file thành f32 samples
    let audio_data = read_wav_to_f32(&wav_path)?;
    
    // Thiết lập parameters
    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    
    // Cấu hình ngôn ngữ - auto detect nếu không được chỉ định
    if language.is_some() {
        let lang = language.unwrap();
        params.set_language(Some(lang));
    } else {
        params.set_language(None); // Auto-detect language
    }
    params.set_translate(false); // Không dịch, giữ nguyên ngôn ngữ gốc
    params.set_print_special(false);
    params.set_print_progress(true);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);
    
    // Tạo state và chạy transcription
    let mut state = ctx.create_state()
        .map_err(|e| anyhow::anyhow!("Không thể tạo Whisper state: {}", e))?;
    
    state.full(params, &audio_data)
        .map_err(|e| anyhow::anyhow!("Lỗi khi transcribe: {}", e))?;
    
    // Lấy kết quả với timestamps - whisper-rs 0.15 API
    let num_segments = state.full_n_segments(); // Returns i32 directly
    
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

    // Không áp dụng segment splitting cho TXT - giữ nguyên segments gốc từ Whisper
    // Xuất TXT với nội dung văn bản thuần túy (theo câu)
    export_segments_to_txt_file(&segments, &final_output)?;

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
        Ok(temp_wav)
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        Err(anyhow::anyhow!("FFmpeg conversion failed: {}", error))
    }
}

/// Combine tất cả segments thành văn bản liền mạch
fn combine_segments_to_text(segments: &[TranscriptSegment]) -> String {
    let mut combined_text = String::new();
    
    for segment in segments {
        let text = segment.text.trim();
        if !text.is_empty() {
            // Thêm khoảng trắng nếu không phải segment đầu tiên
            if !combined_text.is_empty() && !combined_text.ends_with(' ') {
                combined_text.push(' ');
            }
            combined_text.push_str(text);
        }
    }
    
    combined_text
}

/// Chia văn bản thành các câu dựa vào dấu chấm và format thành đoạn văn
fn format_text_into_sentences(text: &str) -> String {
    let mut formatted = String::new();
    
    // Chia theo dấu chấm (.)
    let sentences: Vec<&str> = text.split('.')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    
    for (i, sentence) in sentences.iter().enumerate() {
        // Thêm câu và dấu chấm
        formatted.push_str(sentence);
        if !sentence.ends_with(&['.', '!', '?'][..]) {
            formatted.push('.');
        }
        
        // Thêm khoảng trắng giữa các câu (trừ câu cuối)
        if i < sentences.len() - 1 {
            formatted.push(' ');
        }
    }
    
    formatted
}

/// Chuyển đổi segments thành nội dung TXT file (văn bản thuần túy theo câu)
pub fn segments_to_txt_content(segments: &[TranscriptSegment]) -> String {
    // Bước 1: Combine tất cả segments thành văn bản liền mạch
    let combined_text = combine_segments_to_text(segments);
    
    // Bước 2: Format thành các câu, mỗi câu một dòng
    format_text_into_sentences(&combined_text)
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
                text: "Hello world. This is a test".to_string(),
            },
            TranscriptSegment {
                start: 2.0,
                end: 4.0,
                text: "Another sentence here".to_string(),
            },
        ];
        
        let content = segments_to_txt_content(&segments);
        // Kỳ vọng: các câu cách nhau bằng khoảng trắng, dựa vào dấu chấm
        assert_eq!(content, "Hello world. This is a test. Another sentence here.");
    }

    #[test]
    fn test_format_text_into_sentences() {
        let text = "This is first sentence. This is second sentence. Final sentence";
        let result = format_text_into_sentences(text);
        assert_eq!(result, "This is first sentence. This is second sentence. Final sentence.");
    }
}