use crate::file_audio::get_audio_duration;
use crate::utils::find_ffmpeg_by_os::run_ffmpeg;
use crate::audio_to_text_ass::audio_to_ass;

/// Tạo file caption từ audio path sử dụng Whisper
async fn generate_caption_from_audio(audio_path: &str, video_aspect_ratio: &str) -> Result<String, String> {
    // Tạo đường dẫn output cho caption file
    let caption_output_path = format!("{}_caption.ass", 
        audio_path.rsplit_once('.').map(|(name, _)| name).unwrap_or(audio_path)
    );
    
    // Xác định vị trí caption và video format dựa trên aspect ratio
    let (position, video_format) = match video_aspect_ratio {
        "16:9" => (Some("centerbottom"), Some("landscape")),
        "9:16" => (Some("center"), Some("portrait")),
        _ => (Some("center"), Some("landscape")), // Mặc định
    };
    
    // Gọi audio_to_ass để tạo caption từ audio
    match audio_to_ass(
        audio_path,
        Some(&caption_output_path),
        None, // Sử dụng model mặc định
        None, // Ngôn ngữ mặc định (en)
        position, // Vị trí tùy theo aspect ratio
        video_format, // Video format tùy theo aspect ratio
        None, // Text color mặc định
        None, // Border color mặc định
        None, // Highlight color mặc định
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
    video_aspect_ratio: &str,
) -> Result<(), String> {
    // Tạo caption từ audio path
    let caption_path = generate_caption_from_audio(audio_path, video_aspect_ratio).await?;
    
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
