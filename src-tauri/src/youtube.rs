use std::path::PathBuf;

pub fn find_yt_dlp() -> Option<String> {
    // Common paths for yt-dlp
    let possible_paths = vec![
        "/usr/local/bin/yt-dlp",
        "/opt/homebrew/bin/yt-dlp",
        "/Library/Frameworks/Python.framework/Versions/3.10/bin/yt-dlp",
        "/usr/bin/yt-dlp",
        "yt-dlp", // In PATH
    ];
    
    for path in possible_paths {
        if PathBuf::from(path).exists() || path == "yt-dlp" {
            // Check if it's executable
            if path == "yt-dlp" {
                return Some(path.to_string());
            }
            if std::fs::metadata(path).map(|m| m.is_file()).unwrap_or(false) {
                return Some(path.to_string());
            }
        }
    }
    
    None
}

#[tauri::command]
pub async fn download_youtube_video(url: String, save_path: String) -> Result<String, String> {
    // Validate YouTube URL
    if !url.contains("youtube.com") && !url.contains("youtu.be") {
        return Err("URL không hợp lệ. Vui lòng nhập link YouTube.".to_string());
    }
    
    // Validate save path
    let path = PathBuf::from(&save_path);
    if !path.exists() || !path.is_dir() {
        return Err("Thư mục lưu không hợp lệ".to_string());
    }
    
    // Find yt-dlp executable
    let yt_dlp_path = find_yt_dlp().ok_or_else(|| {
        "Không tìm thấy yt-dlp. Vui lòng cài đặt: brew install yt-dlp".to_string()
    })?;
    
    // Build command - Download MP4 Full HD (1080p) or best available MP4
    // Format: best video+audio combination, prefer MP4, max 1080p
    let output = tokio::process::Command::new(&yt_dlp_path)
        .arg("-f")
        .arg("bestvideo[ext=mp4][height<=1080]+bestaudio[ext=m4a]/bestvideo[ext=mp4]+bestaudio[ext=m4a]/best[ext=mp4][height<=1080]/best[ext=mp4]/bestvideo+bestaudio/best") // Full HD MP4
        .arg("--merge-output-format")
        .arg("mp4") // Merge to MP4 format
        .arg("-S")
        .arg("res:1080") // Prefer 1080p resolution
        .arg("-o")
        .arg(format!("{}/%(title)s.%(ext)s", save_path))
        .arg("--no-playlist") // Only download single video
        .arg("--progress") // Show progress
        .arg("--newline") // Newline for progress updates
        .arg(&url)
        .output()
        .await
        .map_err(|e| format!("Lỗi khi chạy yt-dlp: {}", e))?;
    
    if output.status.success() {
        // Try to extract filename from output
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        // Look for "Destination:" or "Downloading" in output
        let message = if stdout.contains("Destination:") || stdout.contains("Downloading") {
            format!("✅ Download thành công!\n{}", stdout)
        } else if stderr.contains("Destination:") || stderr.contains("Downloading") {
            format!("✅ Download thành công!\n{}", stderr)
        } else {
            format!("✅ Download thành công! Video đã được lưu tại: {}", save_path)
        };
        
        Ok(message)
    } else {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        Err(format!("Lỗi khi download: {}", error_msg))
    }
}

