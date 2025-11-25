use rusty_whisper::Whisper;
use std::fs::File;
use std::io::Write;
use anyhow::Result;
use std::path::PathBuf;
use crate::utils::find_ffmpeg_by_os::run_ffmpeg;

/// Chuyển file audio/video → file .ass đẹp như kênh triệu sub
/// Lưu ý: rusty-whisper 0.1.3 chỉ trả về text, không có timestamps.
/// Để có timestamps chính xác, cần sử dụng crate khác hoặc fork có hỗ trợ segments.
pub fn audio_to_ass(
    input_path: &str,
    output_ass_path: Option<&str>,
    weights_dir: Option<&str>,
) -> Result<String> {
    // Đường dẫn mặc định cho weights (có thể được bundle vào app)
    let weights_base = weights_dir.unwrap_or("weights");
    
    let encoder_path = format!("{}/encoder.onnx", weights_base);
    let decoder_path = format!("{}/decoder.onnx", weights_base);
    let tokenizer_path = format!("{}/multilingual.tiktoken", weights_base);
    let pos_emb_path = format!("{}/positional_embedding.npz", weights_base);
    let mel_filters_path = format!("{}/mel_filters.npz", weights_base);

    println!("Đang tải mô hình Whisper...");
    println!("Encoder: {}", encoder_path);
    println!("Decoder: {}", decoder_path);
    
    // Kiểm tra các file model tồn tại
    for (name, path) in [
        ("encoder", &encoder_path),
        ("decoder", &decoder_path),
        ("tokenizer", &tokenizer_path),
        ("positional embedding", &pos_emb_path),
        ("mel filters", &mel_filters_path),
    ] {
        if !std::path::Path::new(path).exists() {
            return Err(anyhow::anyhow!(
                "Không tìm thấy file {}: {}\nVui lòng tải các file model từ: https://www.dropbox.com/scl/fi/r92pn94756qtxiiu2md0s/weights.zip",
                name, path
            ));
        }
    }
    
    // Khởi tạo Whisper với các file model
    // Lưu ý: Whisper::new() sẽ panic nếu có lỗi, nhưng chúng ta đã kiểm tra file tồn tại
    let whisper = Whisper::new(
        &encoder_path,
        &decoder_path,
        &tokenizer_path,
        &pos_emb_path,
        &mel_filters_path,
    );

    // Chuyển đổi audio sang WAV 16-bit nếu cần (rusty-whisper chỉ hỗ trợ WAV 16-bit)
    let wav_path = ensure_wav_format(input_path)?;

    println!("Đang xử lý: {}", input_path);
    
    // Transcribe audio (mặc định dùng tiếng Anh, có thể thay đổi sau)
    let text = whisper.recognize_from_audio(&wav_path, "en");
    
    println!("Transcription hoàn tất!");

    // Tạo đường dẫn output .ass
    let final_output = output_ass_path.map(|s| s.to_string()).unwrap_or_else(|| {
        input_path.rsplit_once('.').map(|(name, _)| format!("{}_caption.ass", name))
            .unwrap_or_else(|| "audio_01.ass".to_string())
    });

    // Lấy duration của audio để tạo timestamps ước tính
    // Sử dụng get_audio_duration từ file_audio module
    use crate::file_audio::get_audio_duration;
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| anyhow::anyhow!("Không thể tạo tokio runtime: {}", e))?;
    let duration = rt.block_on(get_audio_duration(wav_path.clone()))
        .unwrap_or(0.0);

    // Xuất ASS đẹp (với timestamps ước tính)
    export_to_ass_simple(&text, duration, "en", &final_output)?;

    println!("HOÀN TẤT! File ASS đã tạo:");
    println!("→ {}", final_output);

    // Xóa file WAV tạm nếu đã tạo
    if wav_path != input_path {
        let _ = std::fs::remove_file(&wav_path);
    }

    Ok(final_output)
}

/// Đảm bảo file audio ở định dạng WAV 16-bit (yêu cầu của rusty-whisper)
fn ensure_wav_format(input_path: &str) -> Result<String> {
    let path = PathBuf::from(input_path);
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
    
    // Nếu đã là WAV, kiểm tra xem có cần convert không
    if ext == "wav" {
        // TODO: Kiểm tra format thực tế của file WAV
        // Tạm thời giả sử file WAV đã đúng format
        return Ok(input_path.to_string());
    }
    
    // Nếu không phải WAV, cần convert sang WAV 16-bit
    // Tạo file tạm
    let temp_wav = format!("{}.temp.wav", input_path);
    
    // Sử dụng run_ffmpeg từ utils để tự động tìm ffmpeg path
    let mut cmd = run_ffmpeg()
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    
    cmd.arg("-i")
        .arg(input_path)
        .arg("-ar")
        .arg("16000")
        .arg("-ac")
        .arg("1")
        .arg("-c:a")
        .arg("pcm_s16le")
        .arg("-y") // Overwrite output file
        .arg(&temp_wav);
    
    // Chạy command (sử dụng tokio runtime để chạy async command trong sync context)
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| anyhow::anyhow!("Không thể tạo tokio runtime: {}", e))?;
    
    let output = rt.block_on(cmd.output())
        .map_err(|e| anyhow::anyhow!("Không thể chạy ffmpeg: {}", e))?;
    
    if output.status.success() {
        Ok(temp_wav)
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        Err(anyhow::anyhow!("FFmpeg conversion failed: {}", error))
    }
}

/// Xuất ASS đơn giản với text (không có timestamps chính xác từ segments)
/// Vì rusty-whisper 0.1.3 không hỗ trợ segments, chúng ta sẽ tạo một dialogue
/// với toàn bộ text và duration của audio
fn export_to_ass_simple(text: &str, duration: f64, language: &str, path: &str) -> Result<()> {
    let mut file = File::create(path)?;

    let ass_header = format!(
        r#"[Script Info]
Title: YouTube Caption - Generated by rusty-whisper
ScriptType: v4.00+
Collisions: Normal
PlayResX: 1920
PlayResY: 1080

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,56,&H00FFFFFF,&H000000FF,&H00111111,&H80111111,-1,0,0,0,100,100,0,0,1,3.5,2.5,2,10,10,80,1
Style: Speaker1,Arial,58,&H00E8F5FF,&H000000FF,&H00222222,&H80222222,-1,0,0,0,100,100,0,0,1,3.8,3,2,10,10,90,1
Style: Speaker2,Arial,58,&H00B8FFCC,&H000000FF,&H00222222,&H80222222,-1,0,0,0,100,100,0,0,1,3.8,3,2,10,10,90,1
Style: Highlight,Arial,60,&H0000FFFF,&H000000FF,&H00FF00FF,&H80000000,-1,0,0,0,100,100,0,0,1,4,3.5,2,10,10,70,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
"#
    );

    file.write_all(ass_header.as_bytes())?;

    // Tạo một dialogue với toàn bộ text và duration
    let start_time = seconds_to_ass_time(0.0);
    let end_time = seconds_to_ass_time(duration.max(1.0)); // Ít nhất 1 giây
    
    // Chia text thành các câu ngắn để hiển thị tốt hơn
    let sentences: Vec<&str> = text.split(|c: char| c == '.' || c == '!' || c == '?')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    
    if sentences.is_empty() {
        // Nếu không có câu nào, dùng toàn bộ text
        let escaped = escape_ass_text(text);
        let line = format!("Dialogue: 0,{},{},Default,,0,0,0,,{}\n", start_time, end_time, escaped);
        file.write_all(line.as_bytes())?;
    } else {
        // Chia duration cho số câu
        let duration_per_sentence = duration / sentences.len() as f64;
        
        for (i, sentence) in sentences.iter().enumerate() {
            let start = i as f64 * duration_per_sentence;
            let end = (i + 1) as f64 * duration_per_sentence;
            
            let start_time = seconds_to_ass_time(start);
            let end_time = seconds_to_ass_time(end);
            
            let style = if sentence.to_lowercase().contains("new") ||
                          sentence.to_lowercase().contains("sale") ||
                          sentence.to_lowercase().contains("free") ||
                          sentence.to_lowercase().contains("giảm giá") {
                "Highlight"
            } else if i % 2 == 0 {
                "Speaker1"
            } else {
                "Speaker2"
            };
            
            let escaped = escape_ass_text(sentence);
            let line = format!("Dialogue: 0,{},{},{},,0,0,0,,{}\n", start_time, end_time, style, escaped);
            file.write_all(line.as_bytes())?;
        }
    }

    Ok(())
}

/// Escape ký tự ASS
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
#[tauri::command]
pub async fn convert_audio_to_ass(
    input_path: String,
    output_ass_path: Option<String>,
    weights_dir: Option<String>,
) -> Result<String, String> {
    audio_to_ass(&input_path, output_ass_path.as_deref(), weights_dir.as_deref())
        .map_err(|e| format!("Lỗi khi chuyển đổi: {}", e))
}
