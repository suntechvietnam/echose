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

/// Vị trí caption trên video
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CaptionPosition {
    Top,
    Center,
    CenterBottom,  // Nằm giữa Center và Bottom
    Bottom,
}

impl CaptionPosition {
    /// Parse từ string, mặc định là Center
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "top" => CaptionPosition::Top,
            "centerbottom" | "center-bottom" | "center_bottom" => CaptionPosition::CenterBottom,
            "bottom" => CaptionPosition::Bottom,
            _ => CaptionPosition::Center,
        }
    }
    
    /// Trả về ASS Alignment value
    /// Top: 8, Center: 5, CenterBottom: 2, Bottom: 2
    pub fn to_alignment(&self) -> u8 {
        match self {
            CaptionPosition::Top => 8,
            CaptionPosition::Center => 5,
            CaptionPosition::CenterBottom => 2,  // Cùng alignment với Bottom
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
            VideoFormat::Landscape => 200,  // Margin lớn hơn để block caption nhỏ và căn giữa
            VideoFormat::Portrait => 120,   // Margin lớn hơn cho video dọc để tránh text tràn
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

/// Cấu hình màu sắc cho ASS subtitle
#[derive(Debug, Clone)]
pub struct AssColorConfig {
    pub text_color: String,      // Hex color cho text
    pub border_color: String,    // Hex color cho viền text
    pub highlight_color: String, // Hex color cho karaoke highlight
}

impl Default for AssColorConfig {
    fn default() -> Self {
        Self {
            text_color: "FFFFFF".to_string(),   // Trắng
            border_color: "000000".to_string(), // Đen
            highlight_color: "FFFF00".to_string(), // Vàng
        }
    }
}

/// Cấu hình hoàn chỉnh cho ASS export
#[derive(Debug, Clone)]
pub struct AssExportConfig {
    pub language: String,
    pub position: CaptionPosition,
    pub video_format: VideoFormat,
    pub colors: AssColorConfig,
    pub enable_karaoke: bool,     // Bật/tắt karaoke effects
    pub font_name: String,        // Font family
    pub custom_font_size: Option<u32>, // Override font size
}

impl Default for AssExportConfig {
    fn default() -> Self {
        Self {
            language: "en".to_string(),
            position: CaptionPosition::Center,
            video_format: VideoFormat::Landscape,
            colors: AssColorConfig::default(),
            enable_karaoke: true,
            font_name: "Arial".to_string(),
            custom_font_size: None,
        }
    }
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

impl AssExportConfig {
    /// Tạo config từ các tham số option strings
    pub fn from_options(
        language: Option<&str>,
        position: Option<&str>,
        video_format: Option<&str>,
        text_color: Option<&str>,
        border_color: Option<&str>,
        highlight_color: Option<&str>,
        font_name: Option<&str>,
        font_size: Option<i32>,
    ) -> Self {
        let mut config = Self::default();
        
        if let Some(lang) = language {
            config.language = lang.to_string();
        }
        
        if let Some(pos) = position {
            config.position = CaptionPosition::from_str(pos);
        }
        
        if let Some(format) = video_format {
            config.video_format = VideoFormat::from_str(format);
        }

        if let Some(font) = font_name {
            config.font_name = font.to_string();
        }

        if let Some(size) = font_size {
            config.custom_font_size = Some(size as u32);
        }
        
        if let Some(text_col) = text_color {
            config.colors.text_color = text_col.to_string();
        }
        
        if let Some(border_col) = border_color {
            config.colors.border_color = border_col.to_string();
        }
        
        if let Some(highlight_col) = highlight_color {
            config.colors.highlight_color = highlight_col.to_string();
        }
        
        config
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

/// Chuyển file audio/video → file .ass với timestamps chính xác
/// Sử dụng whisper-rs (bindings cho whisper.cpp)
/// - position: "top", "center", "bottom", "centerbottom" (mặc định: "center")
/// - video_format: "landscape" (16:9) hoặc "short/portrait" (9:16)
/// - text_color: hex color cho text (mặc định: "FFFFFF" - trắng)
/// - border_color: hex color cho viền text (mặc định: "000000" - đen)
/// - highlight_color: hex color cho karaoke highlight (mặc định: "FFFF00" - vàng)
pub fn audio_to_ass(
    input_path: &str,
    output_ass_path: Option<&str>,
    model_path: Option<&str>,
    language: Option<&str>,
    position: Option<&str>,
    video_format: Option<&str>,
    text_color: Option<&str>,
    border_color: Option<&str>,
    highlight_color: Option<&str>,
    font_name: Option<&str>,
    font_size: Option<i32>,
) -> Result<String> {
    // Đường dẫn mặc định cho model (whisper.cpp format .bin)
    let model_name = model_path.unwrap_or("ggml-base.bin");
    let lang = language.unwrap_or("en"); // Mặc định tiếng Anh
    
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
        println!("🚀 Đang transcribe (ngôn ngữ: {})...", lang);
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
    
    // Tạo đường dẫn output .ass
    let final_output = output_ass_path.map(|s| s.to_string()).unwrap_or_else(|| {
        input_path.rsplit_once('.').map(|(name, _)| format!("{}_caption.ass", name))
            .unwrap_or_else(|| "audio_caption.ass".to_string())
    });

    // Áp dụng segment splitting (chia nhỏ theo số từ)
    let split_config = SplitConfig::default();
    let final_segments = apply_segment_splitting(&segments, &split_config);
    
    println!("🔀 Segment splitting: {} -> {} segments", segments.len(), final_segments.len());
    
    // Tạo cấu hình ASS export
    let ass_config = AssExportConfig::from_options(
        Some(lang),
        position,
        video_format,
        text_color,
        border_color,
        highlight_color,
        font_name,
        font_size,
    );
    
    // Xuất ASS với timestamps chính xác
    export_segments_to_ass_file(&final_segments, &ass_config, &final_output)?;

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

/// Chuyển đổi segments thành nội dung ASS file hoàn chỉnh
pub fn segments_to_ass_content(
    segments: &[TranscriptSegment],
    config: &AssExportConfig,
) -> Result<String> {
    let mut content = String::new();
    
    // Tạo ASS header
    let header = generate_ass_header(config)?;
    content.push_str(&header);
    
    // Tạo events từ segments
    for segment in segments.iter() {
        if segment.text.trim().is_empty() {
            continue;
        }
        
        let start_time = seconds_to_ass_time(segment.start);
        let end_time = seconds_to_ass_time(segment.end);
        
        // Tạo text với karaoke effects nếu enabled
        let formatted_text = if config.enable_karaoke {
            create_karaoke_text(&segment.text, segment.start, segment.end)
        } else {
            escape_ass_text(&segment.text)
        };
        
        let style_name = if config.enable_karaoke { "Karaoke" } else { "Default" };
        let line = format!("Dialogue: 0,{},{},{},,0,0,0,,{}\n", 
            start_time, end_time, style_name, formatted_text);
        content.push_str(&line);
    }
    
    Ok(content)
}

/// Xuất ASS file với cấu hình tùy chỉnh
pub fn export_segments_to_ass_file(
    segments: &[TranscriptSegment],
    config: &AssExportConfig,
    output_path: &str,
) -> Result<()> {
    let content = segments_to_ass_content(segments, config)?;
    
    let mut file = File::create(output_path)?;
    file.write_all(content.as_bytes())?;
    
    Ok(())
}

/// Tạo ASS header với cấu hình tùy chỉnh
fn generate_ass_header(config: &AssExportConfig) -> Result<String> {
    // Lấy các thông số từ video format
    let (res_x, res_y) = config.video_format.resolution();
    let font_size = config.custom_font_size
        .unwrap_or_else(|| config.video_format.font_size());
    let margin_lr = config.video_format.margin_lr();
    let outline = config.video_format.outline();
    
    let alignment = config.position.to_alignment();
    
    // MarginV tùy theo vị trí và format
    let margin_v = match (&config.position, &config.video_format) {
        (CaptionPosition::Top, VideoFormat::Landscape) => 50,
        (CaptionPosition::Top, VideoFormat::Portrait) => 80,
        (CaptionPosition::Center, _) => 10,
        (CaptionPosition::CenterBottom, VideoFormat::Landscape) => 40,  // Nằm giữa Center(10) và Bottom(80)
        (CaptionPosition::CenterBottom, VideoFormat::Portrait) => 100,  // Nằm giữa Center(10) và Bottom(150)
        (CaptionPosition::Bottom, VideoFormat::Landscape) => 80,
        (CaptionPosition::Bottom, VideoFormat::Portrait) => 150,
    };
    
    // WrapStyle: 0 = smart wrap, 2 = wrap theo margin
    let wrap_style = match config.video_format {
        VideoFormat::Landscape => 0,
        VideoFormat::Portrait => 2,
    };

    // Chuyển đổi hex colors sang ASS format
    let ass_text_color = format!("&H00{}", hex_to_ass_bgr(&config.colors.text_color));
    let ass_border_color = format!("&H00{}", hex_to_ass_bgr(&config.colors.border_color));
    let ass_highlight_color = format!("&H00{}", hex_to_ass_bgr(&config.colors.highlight_color));
    
    let header = format!(
        r#"[Script Info]
Title: Caption - Generated by echose
ScriptType: v4.00+
Collisions: Normal
PlayResX: {}
PlayResY: {}
Language: {}
WrapStyle: {}

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,{},{},{},&H000000FF,{},&H80000000,-1,0,0,0,100,100,0,0,1,{},1.5,{},{},{},{},1
Style: Karaoke,{},{},{},{},{},&H80000000,-1,0,0,0,100,100,0,0,1,{},1.5,{},{},{},{},1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
"#, 
        res_x, res_y, config.language, wrap_style,
        config.font_name, font_size, ass_text_color, ass_border_color, outline, alignment, margin_lr, margin_lr, margin_v,
        config.font_name, font_size, ass_text_color, ass_highlight_color, ass_border_color, outline, alignment, margin_lr, margin_lr, margin_v
    );
    
    Ok(header)
}

/// Tạo karaoke text với highlight effect
fn create_karaoke_text(text: &str, start: f64, end: f64) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();
    
    if words.is_empty() {
        return escape_ass_text(text);
    }
    
    let duration = end - start;
    let word_count = words.len();
    
    // Chia đều thời gian cho mỗi word (tính bằng centiseconds)
    let duration_per_word_cs = ((duration / word_count as f64) * 100.0) as i64;
    
    let mut result = String::new();
    
    for (i, word) in words.iter().enumerate() {
        let escaped_word = escape_ass_text(word);
        
        if i == 0 {
            result.push_str(&format!("{{\\kf{}}}{}", duration_per_word_cs, escaped_word));
        } else {
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

/// Chuyển đổi hex color (RGB) sang ASS format (BGR)
fn hex_to_ass_bgr(hex: &str) -> String {
    let clean_hex = hex.trim_start_matches('#');
    
    let padded_hex = if clean_hex.len() == 3 {
        // "F0A" -> "FF00AA"
        format!("{}{}{}{}{}{}", 
            clean_hex.chars().nth(0).unwrap(), clean_hex.chars().nth(0).unwrap(),
            clean_hex.chars().nth(1).unwrap(), clean_hex.chars().nth(1).unwrap(), 
            clean_hex.chars().nth(2).unwrap(), clean_hex.chars().nth(2).unwrap())
    } else if clean_hex.len() >= 6 {
        clean_hex[0..6].to_uppercase()
    } else {
        "000000".to_string()
    };
    
    if padded_hex.len() == 6 {
        let r = &padded_hex[0..2];
        let g = &padded_hex[2..4]; 
        let b = &padded_hex[4..6];
        format!("{}{}{}", b, g, r)  // BGR
    } else {
        "000000".to_string()
    }
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
/// - position: "top", "center", "bottom", "centerbottom" (mặc định: "center")
/// - text_color: hex color cho text (mặc định: "FFFFFF")
/// - border_color: hex color cho viền text (mặc định: "000000")
/// - highlight_color: hex color cho karaoke highlight (mặc định: "FFFF00")
#[tauri::command]
pub async fn convert_audio_to_ass(
    input_path: String,
    output_ass_path: Option<String>,
    model_path: Option<String>,
    language: Option<String>,
    position: Option<String>,
    video_format: Option<String>,
    text_color: Option<String>,
    border_color: Option<String>,
    highlight_color: Option<String>,
    font_name: Option<String>,
    font_size: Option<i32>,
) -> Result<String, String> {
    // Chạy trong blocking thread vì whisper processing nặng
    tokio::task::spawn_blocking(move || {
        audio_to_ass(
            &input_path, 
            output_ass_path.as_deref(), 
            model_path.as_deref(),
            language.as_deref(),
            position.as_deref(),
            video_format.as_deref(),
            text_color.as_deref(),
            border_color.as_deref(),
            highlight_color.as_deref(),
            font_name.as_deref(),
            font_size
        )
    })
    .await
    .map_err(|e| format!("Task error: {}", e))?
    .map_err(|e| format!("Lỗi khi chuyển đổi: {}", e))
}

/// Tauri command: Chuyển segments thành nội dung ASS string (không lưu file)
/// Hữu ích cho preview hoặc return content trực tiếp
#[tauri::command]
pub async fn segments_to_ass_string(
    segments: Vec<(f64, f64, String)>, // (start, end, text) tuples
    language: Option<String>,
    position: Option<String>,
    video_format: Option<String>,
    text_color: Option<String>,
    border_color: Option<String>,
    highlight_color: Option<String>,
    font_name: Option<String>,
    font_size: Option<i32>,
) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        // Convert tuples to TranscriptSegment
        let transcript_segments: Vec<TranscriptSegment> = segments
            .into_iter()
            .map(|(start, end, text)| TranscriptSegment { start, end, text })
            .collect();

        // Tạo cấu hình ASS export
        let ass_config = AssExportConfig::from_options(
            language.as_deref(),
            position.as_deref(),
            video_format.as_deref(),
            text_color.as_deref(),
            border_color.as_deref(),
            highlight_color.as_deref(),
            font_name.as_deref(),
            font_size,
        );

        // Tạo nội dung ASS
        segments_to_ass_content(&transcript_segments, &ass_config)
            .map_err(|e| format!("Lỗi khi tạo ASS content: {}", e))
    })
    .await
    .map_err(|e| format!("Task error: {}", e))?
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_to_ass_bgr() {
        assert_eq!(hex_to_ass_bgr("FF0000"), "0000FF"); // Red RGB -> Red BGR
        assert_eq!(hex_to_ass_bgr("00FF00"), "00FF00"); // Green RGB -> Green BGR
        assert_eq!(hex_to_ass_bgr("0000FF"), "FF0000"); // Blue RGB -> Blue BGR
        assert_eq!(hex_to_ass_bgr("#FFFFFF"), "FFFFFF"); // White with #
        assert_eq!(hex_to_ass_bgr("F0A"), "A0A0F0F0"); // 3 char expansion
    }

    #[test]
    fn test_caption_position() {
        assert_eq!(CaptionPosition::from_str("top").to_alignment(), 8);
        assert_eq!(CaptionPosition::from_str("center").to_alignment(), 5);
        assert_eq!(CaptionPosition::from_str("centerbottom").to_alignment(), 2);
        assert_eq!(CaptionPosition::from_str("center-bottom").to_alignment(), 2);
        assert_eq!(CaptionPosition::from_str("bottom").to_alignment(), 2);
        assert_eq!(CaptionPosition::from_str("invalid").to_alignment(), 5); // Default center
    }

    #[test]
    fn test_video_format() {
        assert_eq!(VideoFormat::from_str("landscape").resolution(), (1920, 1080));
        assert_eq!(VideoFormat::from_str("portrait").resolution(), (1080, 1920));
        assert_eq!(VideoFormat::from_str("tiktok").resolution(), (1080, 1920));
        assert_eq!(VideoFormat::from_str("invalid").resolution(), (1920, 1080)); // Default landscape
    }

    #[test]
    fn test_ass_time_format() {
        assert_eq!(seconds_to_ass_time(0.0), "0:00:00.00");
        assert_eq!(seconds_to_ass_time(61.5), "0:01:01.50");
        assert_eq!(seconds_to_ass_time(3661.25), "1:01:01.25");
    }
}