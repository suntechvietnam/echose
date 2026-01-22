use crate::file_audio::get_audio_duration;
use crate::utils::find_ffmpeg_by_os::run_ffmpeg;
use crate::audio_to_text_ass::audio_to_ass;

/// Tạo file caption từ audio path sử dụng Whisper
async fn generate_caption_from_audio(
    audio_path: &str,
    language: Option<&str>,
    font_name: Option<&str>,
    font_size: Option<i32>,
) -> Result<String, String> {
    // Tạo đường dẫn output cho caption file
    let caption_output_path = format!("{}_caption.ass", 
        audio_path.rsplit_once('.').map(|(name, _)| name).unwrap_or(audio_path)
    );
    
    // Gọi audio_to_ass để tạo caption từ audio
    match audio_to_ass(
        audio_path,
        Some(&caption_output_path),
        None, // Sử dụng model mặc định
        language,
        None, // Vị trí mặc định (center)
        None, // Video format mặc định (landscape)
        None, // Text color mặc định
        None, // Border color mặc định
        None, // Highlight color mặc định
        font_name,
        font_size,
    ) {
        Ok(ass_file_path) => Ok(ass_file_path),
        Err(e) => Err(format!("Lỗi khi tạo caption từ audio: {}", e))
    }
}

/**
 * Merge video với audio, đảm bảo video loop để match với audio duration
 * Sử dụng -c:v copy để copy video stream trực tiếp (không re-encode), nhanh hơn nhiều
 */
pub async fn merge_audio_without_caption(
    video_path: &str,
    audio_path: &str,
    output_path: &str,
) -> Result<(), String> {
    // Lấy duration của audio file
    let audio_duration = get_audio_duration(audio_path.to_string()).await
        .map_err(|e| format!("Lỗi khi đọc duration của audio: {}", e))?;
    
    let mut cmd = run_ffmpeg()?;

    cmd.arg("-stream_loop")
        .arg("-1") // Loop video vô hạn
        .arg("-i")
        .arg(video_path)
        .arg("-i")
        .arg(audio_path)
        .arg("-t")
        .arg(format!("{:.2}", audio_duration)) // Giới hạn output theo duration của audio
        .arg("-c:v")
        .arg("copy") // Copy video stream trực tiếp - không re-encode, nhanh hơn nhiều
        .arg("-c:a")
        .arg("aac") // Encode audio thành AAC
        .arg("-b:a")
        .arg("256k") // Bitrate audio 256kbps cho chất lượng cao nhất
        .arg("-movflags")
        .arg("+faststart") // Fast start để stream tốt hơn
        .arg("-y")
        .arg(output_path);
    
    // Chạy và đợi process hoàn thành
    let output = cmd.output().await
        .map_err(|e| format!("Lỗi khi merge video với audio: {}", e))?;
    
    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Lỗi khi merge video với audio: {}", error_msg));
    }
    Ok(())
}

/**
 * Merge video với audio và caption, đảm bảo video loop để match với audio duration
 * Tự động tạo caption từ file audio bằng Whisper
 */
pub async fn merge_audio_with_caption(
    video_path: &str,
    audio_path: &str,
    output_path: &str,
) -> Result<(), String> {
    // Tạo caption từ audio path (mặc định các settings font)
    let caption_path = generate_caption_from_audio(audio_path, None, None, None).await?;
    
    // Lấy duration của audio file
    let audio_duration = get_audio_duration(audio_path.to_string()).await
        .map_err(|e| format!("Lỗi khi đọc duration của audio: {}", e))?;
    
    let mut cmd = run_ffmpeg()?;

    cmd.arg("-stream_loop")
        .arg("-1") // Loop video vô hạn
        .arg("-i")
        .arg(video_path)
        .arg("-i")
        .arg(audio_path)
        .arg("-t")
        .arg(format!("{:.2}", audio_duration)) // Giới hạn output theo duration của audio
        .arg("-vf")
        .arg(format!("subtitles='{}'", caption_path.replace('\'', "'\\''"))) // Burn-in subtitle vào video với escape
        .arg("-c:a")
        .arg("aac") // Encode audio thành AAC
        .arg("-b:a")
        .arg("256k") // Bitrate audio 256kbps cho chất lượng cao nhất
        .arg("-c:v")
        .arg("libx264") // Re-encode video để burn-in subtitle
        .arg("-preset")
        .arg("medium") // Balance giữa tốc độ và chất lượng
        .arg("-crf")
        .arg("20") // Chất lượng video cao
        .arg("-pix_fmt")
        .arg("yuv420p") // Pixel format tương thích
        .arg("-movflags")
        .arg("+faststart") // Fast start để stream tốt hơn
        .arg("-y")
        .arg(output_path);
    
    // Chạy và đợi process hoàn thành
    let output = cmd.output().await
        .map_err(|e| format!("Lỗi khi merge video với audio và caption: {}", e))?;
    
    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Lỗi khi merge video với audio và caption: {}", error_msg));
    }

    // Xóa file caption tạm sau khi merge xong
    std::fs::remove_file(caption_path)
        .map_err(|e| format!("Lỗi khi xóa file caption tạm: {}", e))?;

    Ok(())
}

/**
 * MASTER MERGE: Ghép Video + Audio + Logo + Subtitle trong một lần duy nhất
 * Đảm bảo chất lượng tốt nhất và tiết kiệm thời gian (chỉ re-encode 1 lần)
 */
pub async fn merge_video_all_in_one(
    video_path: &str,
    audio_path: Option<&str>,
    logo_path: Option<&str>,
    logo_position: Option<&str>, // "top_left", "top_right", "bottom_left", "bottom_right"
    logo_margin_top: i32,
    logo_margin_right: i32,
    logo_margin_bottom: i32,
    logo_margin_left: i32,
    subtitle_path: Option<&str>,
    subtitle_margin_v: i32,
    subtitle_font_size: i32,
    subtitle_language: Option<&str>,
    subtitle_font_name: Option<&str>,
    has_auto_caption: bool,
    output_path: &str,
) -> Result<(), String> {
    let mut cmd = run_ffmpeg()?;
    
    // 1. Inputs
    // Input 0: Video
    if audio_path.is_some() {
        cmd.arg("-stream_loop").arg("-1"); // Loop video nếu có audio
    }
    cmd.arg("-i").arg(video_path);
    
    // Input 1: Audio (nếu có)
    if let Some(ap) = audio_path {
        cmd.arg("-i").arg(ap);
    }
    
    // Input 2: Logo (nếu có)
    if let Some(lp) = logo_path {
        cmd.arg("-i").arg(lp);
    }
    
    // 2. Duration & Audio Duration
    if let Some(ap) = audio_path {
        let audio_duration = get_audio_duration(ap.to_string()).await
            .map_err(|e| format!("Lỗi khi đọc duration audio: {}", e))?;
        cmd.arg("-t").arg(format!("{:.2}", audio_duration));
    }
    
    // 3. Filter Complex (Logo + Subtitles)
    let mut filters = Vec::new();
    let mut current_video_stream = "[0:v]".to_string();
    
    // A. Logo Overlay với Margin tùy chỉnh
    if let Some(_lp) = logo_path {
        let logo_input_index = if audio_path.is_some() { 2 } else { 1 };
        
        let pos = match logo_position.unwrap_or("top_left") {
            "top_right" => format!("W-w-{}:{}", logo_margin_right, logo_margin_top),
            "bottom_left" => format!("{}:H-h-{}", logo_margin_left, logo_margin_bottom),
            "bottom_right" => format!("W-w-{}:H-h-{}", logo_margin_right, logo_margin_bottom),
            _ => format!("{}:{}", logo_margin_left, logo_margin_top), // top_left default
        };
        
        filters.push(format!("{}[{}:v]overlay={}[v_logo]", current_video_stream, logo_input_index, pos));
        current_video_stream = "[v_logo]".to_string();
    }
    
    // B. Subtitles với Wrapping tự động
    // WrapStyle=0: Smart wrapping, Alignment=2: Bottom Center
    let force_style = format!(
        "WrapStyle=0,Overlap=1,Alignment=2,MarginV={},FontSize={},FontName={}", 
        subtitle_margin_v, 
        subtitle_font_size,
        subtitle_font_name.unwrap_or("Arial")
    );
    
    let mut temp_caption_path = None;
    if has_auto_caption {
        if let Some(ap) = audio_path {
             let cap_path = generate_caption_from_audio(ap, subtitle_language, subtitle_font_name, Some(subtitle_font_size)).await?;
             filters.push(format!("{}subtitles='{}':force_style='{}'[v_sub]", 
                current_video_stream, cap_path.replace('\'', "'\\''"), force_style));
             current_video_stream = "[v_sub]".to_string();
             temp_caption_path = Some(cap_path);
        }
    } else if let Some(sp) = subtitle_path {
        filters.push(format!("{}subtitles='{}':force_style='{}'[v_sub]", 
            current_video_stream, sp.replace('\'', "'\\''"), force_style));
        current_video_stream = "[v_sub]".to_string();
    }
    
    if !filters.is_empty() {
        cmd.arg("-filter_complex").arg(filters.join(";"));
        cmd.arg("-map").arg(current_video_stream);
    } else {
        // Nếu không có filter thì copy video stream cho nhanh nếu không có audio
        // Nhưng nếu có audio và loop thì thường vẫn nên re-encode để ổn định
        cmd.arg("-map").arg("0:v");
    }
    
    // 4. Audio Mapping
    if audio_path.is_some() {
        cmd.arg("-map").arg("1:a").arg("-c:a").arg("aac").arg("-b:a").arg("256k");
    } else {
        // Giữ audio gốc nếu có
        cmd.arg("-c:a").arg("copy");
    }
    
    // 5. Video Encoding
    // Nếu có logo hoặc sub thì BẮT BUỘC phải encode lại
    if logo_path.is_some() || subtitle_path.is_some() || has_auto_caption {
        cmd.arg("-c:v").arg("libx264")
           .arg("-preset").arg("medium")
           .arg("-crf").arg("21")
           .arg("-pix_fmt").arg("yuv420p");
    } else {
        cmd.arg("-c:v").arg("copy");
    }
    
    cmd.arg("-movflags").arg("+faststart").arg("-y").arg(output_path);
    
    println!("Master Merge Command: {:?}", cmd);
    
    let output = cmd.output().await.map_err(|e| e.to_string())?;
    
    // Cleanup
    if let Some(tcp) = temp_caption_path {
        let _ = std::fs::remove_file(tcp);
    }
    
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("FFmpeg Master Merge Error: {}", err));
    }
    
    Ok(())
}
