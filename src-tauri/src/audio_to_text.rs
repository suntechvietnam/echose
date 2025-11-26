use whisper_rs::{WhisperContext, WhisperContextParameters, FullParams, SamplingStrategy};
use std::fs::File;
use std::io::Write;
use anyhow::Result;
use crate::utils::find_ffmpeg_by_os::run_ffmpeg_sync;

/// Segment chứa thông tin timestamp và text
#[derive(Debug, Clone)]
pub struct TranscriptSegment {
    pub start: f64,  // seconds
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
            
            // Thử các đường dẫn tương đối từ thư mục exe
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
    
    // Thử đường dẫn tương đối từ working directory
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

/// Vị trí caption trên video
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CaptionPosition {
    Top,
    Center,
    Bottom,
}

impl CaptionPosition {
    /// Parse từ string, mặc định là Center
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "top" => CaptionPosition::Top,
            "bottom" => CaptionPosition::Bottom,
            _ => CaptionPosition::Center,
        }
    }
    
    /// Trả về ASS Alignment value
    /// Top: 8, Center: 5, Bottom: 2
    pub fn to_alignment(&self) -> u8 {
        match self {
            CaptionPosition::Top => 8,
            CaptionPosition::Center => 5,
            CaptionPosition::Bottom => 2,
        }
    }
}

/// Định dạng video (tỉ lệ khung hình)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VideoFormat {
    Landscape,  // 16:9 ngang (1920x1080) - YouTube, Facebook
    Portrait,   // 9:16 dọc (1080x1920) - TikTok, Reels, Shorts
}

impl VideoFormat {
    /// Parse từ string, mặc định là Landscape
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "short" | "shorts" | "portrait" | "vertical" | "tiktok" | "reels" => VideoFormat::Portrait,
            _ => VideoFormat::Landscape,
        }
    }
    
    /// Trả về resolution (width, height)
    pub fn resolution(&self) -> (u32, u32) {
        match self {
            VideoFormat::Landscape => (1920, 1080),
            VideoFormat::Portrait => (1080, 1920),
        }
    }
    
    /// Trả về font size phù hợp
    pub fn font_size(&self) -> u32 {
        match self {
            VideoFormat::Landscape => 58,
            VideoFormat::Portrait => 42,  // Nhỏ hơn vì màn hẹp
        }
    }
    
    /// Trả về margin left/right
    pub fn margin_lr(&self) -> u32 {
        match self {
            VideoFormat::Landscape => 10,
            VideoFormat::Portrait => 40,  // Margin rộng hơn cho màn dọc
        }
    }
    
    /// Trả về outline thickness
    pub fn outline(&self) -> f32 {
        match self {
            VideoFormat::Landscape => 3.0,
            VideoFormat::Portrait => 2.5,
        }
    }
}

/// Chuyển file audio/video → file .ass với timestamps chính xác
/// Sử dụng whisper-rs (bindings cho whisper.cpp)
/// - position: "top", "center", "bottom" (mặc định: "center")
/// - video_format: "landscape" (16:9) hoặc "short/portrait" (9:16)
pub fn audio_to_ass(
    input_path: &str,
    output_ass_path: Option<&str>,
    model_path: Option<&str>,
    language: Option<&str>,
    position: Option<&str>,
    video_format: Option<&str>,
) -> Result<String> {
    // Đường dẫn mặc định cho model (whisper.cpp format .bin)
    let model_name = model_path.unwrap_or("ggml-base.bin");
    let lang = language.unwrap_or("en"); // Mặc định tiếng Anh
    
    println!("🎤 Đang tìm mô hình Whisper...");
    
    // Tìm model
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
    
    // Cấu hình ngôn ngữ
    params.set_language(Some(lang));
    params.set_translate(false); // Không dịch, giữ nguyên ngôn ngữ gốc
    params.set_print_special(false);
    params.set_print_progress(true);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);
    
    // Tạo state và chạy transcription
    println!("🚀 Đang transcribe (ngôn ngữ: {})...", lang);
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
    
    // Tạo đường dẫn output .ass
    let final_output = output_ass_path.map(|s| s.to_string()).unwrap_or_else(|| {
        input_path.rsplit_once('.').map(|(name, _)| format!("{}_caption.ass", name))
            .unwrap_or_else(|| "audio_caption.ass".to_string())
    });

    // Parse position và video format
    let caption_pos = CaptionPosition::from_str(position.unwrap_or("center"));
    let vid_format = VideoFormat::from_str(video_format.unwrap_or("landscape"));
    
    // Xuất ASS với timestamps chính xác
    export_to_ass(&segments, lang, &final_output, caption_pos, vid_format)?;

    println!("🎉 HOÀN TẤT! File ASS đã tạo:");
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

/// Xuất file ASS với timestamps chính xác từ segments
/// - Sử dụng 1 style: chữ trắng viền đen
/// - Hỗ trợ karaoke highlight từng word
/// - Vị trí caption có thể tùy chỉnh (top, center, bottom)
/// - Hỗ trợ cả video ngang (16:9) và video dọc/short (9:16)
fn export_to_ass(
    segments: &[TranscriptSegment], 
    language: &str, 
    path: &str,
    position: CaptionPosition,
    video_format: VideoFormat,
) -> Result<()> {
    let mut file = File::create(path)?;
    
    // Lấy các thông số từ video format
    let (res_x, res_y) = video_format.resolution();
    let font_size = video_format.font_size();
    let margin_lr = video_format.margin_lr();
    let outline = video_format.outline();
    
    let alignment = position.to_alignment();
    
    // MarginV tùy theo vị trí và format
    let margin_v = match (&position, &video_format) {
        (CaptionPosition::Top, VideoFormat::Landscape) => 50,
        (CaptionPosition::Top, VideoFormat::Portrait) => 80,
        (CaptionPosition::Center, _) => 10,
        (CaptionPosition::Bottom, VideoFormat::Landscape) => 80,
        (CaptionPosition::Bottom, VideoFormat::Portrait) => 150,  // Cao hơn cho short
    };
    
    // WrapStyle: 0 = smart wrap, 2 = wrap theo margin (tốt hơn cho portrait)
    let wrap_style = match video_format {
        VideoFormat::Landscape => 0,
        VideoFormat::Portrait => 2,
    };

    // ASS Header với style động theo video format:
    // - Chữ trắng (FFFFFF), viền đen (000000)
    // - SecondaryColour vàng (00FFFF) cho karaoke highlight
    let ass_header = format!(
        r#"[Script Info]
Title: Caption - Generated by whisper-rs
ScriptType: v4.00+
Collisions: Normal
PlayResX: {}
PlayResY: {}
Language: {}
WrapStyle: {}

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,{},&H00FFFFFF,&H000000FF,&H00000000,&H80000000,-1,0,0,0,100,100,0,0,1,{},1.5,{},{},{},{},1
Style: Karaoke,Arial,{},&H00FFFFFF,&H0000FFFF,&H00000000,&H80000000,-1,0,0,0,100,100,0,0,1,{},1.5,{},{},{},{},1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
"#, 
        res_x, res_y, language, wrap_style,
        font_size, outline, alignment, margin_lr, margin_lr, margin_v,
        font_size, outline, alignment, margin_lr, margin_lr, margin_v
    );

    file.write_all(ass_header.as_bytes())?;

    for segment in segments.iter() {
        if segment.text.is_empty() {
            continue;
        }
        
        let start_time = seconds_to_ass_time(segment.start);
        let end_time = seconds_to_ass_time(segment.end);
        
        // Tạo karaoke effect cho từng word
        let karaoke_text = create_karaoke_text(&segment.text, segment.start, segment.end);
        
        let line = format!("Dialogue: 0,{},{},Karaoke,,0,0,0,,{}\n", 
            start_time, end_time, karaoke_text);
        file.write_all(line.as_bytes())?;
    }

    Ok(())
}

/// Tạo karaoke text với highlight effect
/// Mỗi word sẽ được highlight khi đang đọc, sau đó quay về màu trắng
fn create_karaoke_text(text: &str, start: f64, end: f64) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();
    
    if words.is_empty() {
        return escape_ass_text(text);
    }
    
    let duration = end - start;
    let word_count = words.len();
    
    // Chia đều thời gian cho mỗi word (tính bằng centiseconds cho \k tag)
    let duration_per_word_cs = ((duration / word_count as f64) * 100.0) as i64;
    
    let mut result = String::new();
    
    for (i, word) in words.iter().enumerate() {
        let escaped_word = escape_ass_text(word);
        
        // \k = karaoke fill effect (word được highlight khi đến lượt)
        // \kf = karaoke fill with smooth transition
        // Duration tính bằng centiseconds
        if i == 0 {
            // Word đầu tiên: bắt đầu highlight
            result.push_str(&format!("{{\\kf{}}}{}", duration_per_word_cs, escaped_word));
        } else {
            // Các word tiếp theo: thêm space và highlight
            result.push_str(&format!(" {{\\kf{}}}{}", duration_per_word_cs, escaped_word));
        }
    }
    
    result
}

/// Escape ký tự đặc biệt trong ASS
fn escape_ass_text(text: &str) -> String {
    text
        .replace('\\', r"\\")
        .replace('{', r"\{")
        .replace('}', r"\}")
        .replace('\n', r"\N")
}

/// Chuyển giây → định dạng ASS: 0:00:12.34
fn seconds_to_ass_time(seconds: f64) -> String {
    let total_centisec = (seconds * 100.0) as i64;
    let h = total_centisec / 360000;
    let m = (total_centisec % 360000) / 6000;
    let s = (total_centisec % 6000) / 100;
    let cs = total_centisec % 100;
    format!("{}:{:02}:{:02}.{:02}", h, m, s, cs)
}

/// Tauri command: Chuyển file audio/video → file .ass
/// - position: "top", "center", "bottom" (mặc định: "center")
#[tauri::command]
pub async fn convert_audio_to_ass(
    input_path: String,
    output_ass_path: Option<String>,
    model_path: Option<String>,
    language: Option<String>,
    position: Option<String>,
    video_format: Option<String>,
) -> Result<String, String> {
    // Chạy trong blocking thread vì whisper processing nặng
    tokio::task::spawn_blocking(move || {
        audio_to_ass(
            &input_path, 
            output_ass_path.as_deref(), 
            model_path.as_deref(),
            language.as_deref(),
            position.as_deref(),
            video_format.as_deref()
        )
    })
    .await
    .map_err(|e| format!("Task error: {}", e))?
    .map_err(|e| format!("Lỗi khi chuyển đổi: {}", e))
}

/// Tauri command: Lấy danh sách models có sẵn
#[tauri::command]
pub async fn list_whisper_models(models_dir: Option<String>) -> Result<Vec<String>, String> {
    let dir = models_dir.unwrap_or_else(|| "models".to_string());
    
    let mut models = Vec::new();
    
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "bin").unwrap_or(false) {
                if let Some(name) = path.file_name() {
                    models.push(name.to_string_lossy().to_string());
                }
            }
        }
    }
    
    Ok(models)
}

/// Tauri command: Copy file ASS đến vị trí người dùng chọn
#[tauri::command]
pub async fn save_ass_file(source_path: String, destination_path: String) -> Result<String, String> {
    std::fs::copy(&source_path, &destination_path)
        .map_err(|e| format!("Không thể lưu file: {}", e))?;
    
    Ok(destination_path)
}

/// Tauri command: Đọc nội dung file ASS (để hiển thị hoặc download)
#[tauri::command]
pub async fn read_ass_file(file_path: String) -> Result<String, String> {
    std::fs::read_to_string(&file_path)
        .map_err(|e| format!("Không thể đọc file: {}", e))
}

/// Tauri command: Xóa file ASS tạm
#[tauri::command]
pub async fn delete_temp_ass_file(file_path: String) -> Result<(), String> {
    std::fs::remove_file(&file_path)
        .map_err(|e| format!("Không thể xóa file: {}", e))
}
