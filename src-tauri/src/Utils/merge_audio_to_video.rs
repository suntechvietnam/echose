use crate::file_audio::get_audio_duration;
use crate::utils::find_ffmpeg_by_os::run_ffmpeg;
use std::path::PathBuf;

/// Tìm đường dẫn đến file caption
fn find_caption_file() -> Result<String, String> {
    let caption_filename = "audio_01_caption.ass";
    
    // Nếu là đường dẫn tuyệt đối và tồn tại, dùng luôn
    let path = PathBuf::from(caption_filename);
    if path.is_absolute() && path.exists() {
        return Ok(caption_filename.to_string());
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
                        resources_dir.join(caption_filename),
                        resources_dir.join("src").join(caption_filename),
                    ];
                    
                    for p in paths {
                        if p.exists() {
                            return Ok(p.to_string_lossy().to_string());
                        }
                    }
                }
            }
            
            // Thử các đường dẫn tương đối từ thư mục exe
            let paths = vec![
                app_dir.join(caption_filename),
                app_dir.join("src").join(caption_filename),
            ];
            
            for p in paths {
                if p.exists() {
                    return Ok(p.to_string_lossy().to_string());
                }
            }
        }
    }
    
    // Thử đường dẫn tương đối từ working directory
    let paths = vec![
        PathBuf::from(caption_filename),
        PathBuf::from("src").join(caption_filename),
        PathBuf::from("src-tauri").join("src").join(caption_filename),
    ];
    
    for p in paths {
        if p.exists() {
            return Ok(p.to_string_lossy().to_string());
        }
    }
    
    Err(format!(
        "Không tìm thấy file caption: {}\n\
        Đã tìm trong: app directory, src/, src-tauri/src/, và working directory.\n\
        Vui lòng đảm bảo file {} tồn tại.",
        caption_filename, caption_filename
    ))
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
 * Sử dụng file caption cố định: audio_01_caption.ass
 */
pub async fn merge_audio_with_caption(
    video_path: &str,
    audio_path: &str,
    output_path: &str,
) -> Result<(), String> {
    // Tìm đường dẫn đến file caption
    let caption_path = find_caption_file()?;
    
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
    Ok(())
}
