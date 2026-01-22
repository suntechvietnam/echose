use std::path::PathBuf;
use std::process::Command;

/// Tìm đường dẫn đến FFprobe executable
pub fn find_ffprobe() -> Option<String> {
    // Xác định tên file FFprobe theo platform
    #[cfg(target_os = "windows")]
    let ffprobe_name = "ffprobe.exe";
    #[cfg(not(target_os = "windows"))]
    let ffprobe_name = "ffprobe";
    
    // Ưu tiên tìm trong app bundle trước (khi được bundle vào app)
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(app_dir) = exe_path.parent() {
            #[cfg(target_os = "macos")]
            {
                // Trên macOS: App.app/Contents/Resources/resources/ffprobe
                if let Some(contents_dir) = app_dir.parent() {
                    let resources_dir = contents_dir.join("Resources");
                    
                    // Thử tìm trong resources/ffprobe (khi được bundle)
                    let bundled_ffprobe1 = resources_dir.join("resources").join(ffprobe_name);
                    if bundled_ffprobe1.exists() {
                        return Some(bundled_ffprobe1.to_string_lossy().to_string());
                    }
                    
                    // Thử tìm trực tiếp trong Resources/ffprobe
                    let bundled_ffprobe2 = resources_dir.join(ffprobe_name);
                    if bundled_ffprobe2.exists() {
                        return Some(bundled_ffprobe2.to_string_lossy().to_string());
                    }
                }
            }
            
            #[cfg(target_os = "windows")]
            {
                // Trên Windows: tìm trong cùng thư mục với .exe hoặc trong resources/
                let local_ffprobe = app_dir.join(ffprobe_name);
                if local_ffprobe.exists() {
                    return Some(local_ffprobe.to_string_lossy().to_string());
                }
                
                // Tìm trong thư mục resources/ (nếu có)
                let resources_ffprobe = app_dir.join("resources").join(ffprobe_name);
                if resources_ffprobe.exists() {
                    return Some(resources_ffprobe.to_string_lossy().to_string());
                }
            }
            
            // Fallback: tìm trong cùng thư mục với executable
            let local_ffprobe = app_dir.join(ffprobe_name);
            if local_ffprobe.exists() {
                return Some(local_ffprobe.to_string_lossy().to_string());
            }
        }
    }
    
    // Tìm trong system paths
    #[cfg(target_os = "windows")]
    {
        if let Ok(_) = std::process::Command::new("ffprobe").arg("-version").output() {
            return Some("ffprobe".to_string());
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        let possible_paths = vec![
            "/usr/local/bin/ffprobe",
            "/opt/homebrew/bin/ffprobe",
            "/usr/bin/ffprobe",
            "ffprobe",
        ];
        
        for path in possible_paths {
            if path == "ffprobe" {
                return Some(path.to_string());
            }
            if PathBuf::from(path).exists() {
                if std::fs::metadata(path).map(|m| m.is_file()).unwrap_or(false) {
                    return Some(path.to_string());
                }
            }
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        let possible_paths = vec![
            "/usr/local/bin/ffprobe",
            "/usr/bin/ffprobe",
            "ffprobe",
        ];
        
        for path in possible_paths {
            if path == "ffprobe" {
                return Some(path.to_string());
            }
            if PathBuf::from(path).exists() {
                if std::fs::metadata(path).map(|m| m.is_file()).unwrap_or(false) {
                    return Some(path.to_string());
                }
            }
        }
    }
    
    None
}

/// Đọc duration của file audio bằng ffprobe (nhanh hơn JavaScript)
/// Sử dụng ffprobe để đọc metadata mà không cần decode toàn bộ file,
/// nhanh hơn nhiều so với đọc file vào memory trong JavaScript.
/// # Arguments
/// * `file_path` - Đường dẫn đến file audio (MP3, WAV, M4A, AAC, OGG, etc.)
/// # Returns
/// * `Ok(f64)` - Duration tính bằng giây
/// * `Err(String)` - Lỗi nếu không thể đọc duration
#[tauri::command]
pub async fn get_audio_duration(file_path: String) -> Result<f64, String> {
    let ffprobe_path = find_ffprobe().ok_or_else(|| {
        "Không tìm thấy ffprobe. Vui lòng cài đặt ffmpeg (bao gồm ffprobe)".to_string()
    })?;
    
    // Kiểm tra file tồn tại
    let path = PathBuf::from(&file_path);
    if !path.exists() {
        return Err(format!("File không tồn tại: {}", file_path));
    }
    
    // Sử dụng ffprobe để đọc duration (chỉ đọc metadata, không decode file)
    // ffprobe -v error -show_entries format=duration -of default=noprint_wrappers=1:nokey=1 file.mp3
    let output = Command::new(&ffprobe_path)
        .arg("-v")
        .arg("error")
        .arg("-show_entries")
        .arg("format=duration")
        .arg("-of")
        .arg("default=noprint_wrappers=1:nokey=1")
        .arg(&file_path)
        .output()
        .map_err(|e| format!("Lỗi khi chạy ffprobe: {}", e))?;
    
    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("ffprobe error: {}", error_msg));
    }
    
    // Parse output (duration là số thực, tính bằng giây)
    let duration_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    
    if duration_str.is_empty() {
        return Err("Không thể đọc duration từ file".to_string());
    }
    
    let duration: f64 = duration_str.parse()
        .map_err(|_| format!("Không thể parse duration: {}", duration_str))?;
    
    if duration.is_nan() || duration <= 0.0 || !duration.is_finite() {
        return Err(format!("Duration không hợp lệ: {}", duration));
    }
    
    Ok(duration)
}

/// Kiểm tra xem file video có chứa stream audio hay không
async fn has_audio_stream(file_path: &str) -> Result<bool, String> {
    let ffprobe_path = find_ffprobe().ok_or_else(|| {
        "Không tìm thấy ffprobe".to_string()
    })?;
    
    let output = std::process::Command::new(&ffprobe_path)
        .arg("-v").arg("error")
        .arg("-select_streams").arg("a")
        .arg("-show_entries").arg("stream=index")
        .arg("-of").arg("csv=p=0")
        .arg(file_path)
        .output()
        .map_err(|e| format!("Lỗi khi chạy ffprobe: {}", e))?;
    
    let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(!result.is_empty())
}

use crate::process::ProcessStore;
use crate::utils::find_ffmpeg_by_os::run_ffmpeg;
use std::sync::{Arc, Mutex};

/// Trích xuất audio từ video file
#[tauri::command]
pub async fn extract_audio_from_video(
    video_path: String,
    output_path: String,
    format: String, // "mp3" hoặc "m4a"
    process_id: String,
    processes: tauri::State<'_, ProcessStore>,
) -> Result<String, String> {
    // 1. Kiểm tra đầu vào
    let video_p = std::path::Path::new(&video_path);
    if !video_p.exists() {
        return Err(format!("File video không tồn tại tại: {}", video_path));
    }

    // Kiểm tra xem video có audio không
    if !has_audio_stream(&video_path).await? {
        return Err("Video này không có âm thanh (Audio stream) để trích xuất.".to_string());
    }

    let output_p = std::path::Path::new(&output_path);
    if let Some(parent) = output_p.parent() {
        if !parent.exists() {
            return Err(format!("Thư mục lưu không tồn tại: {:?}", parent));
        }
    }

    let mut cmd = run_ffmpeg()?;
    
    cmd.arg("-i").arg(&video_path)
       .arg("-vn"); // Không lấy video
    
    // 2. Cấu hình Audio Codec
    if format == "mp3" {
        // Thử dùng libmp3lame, nếu lỗi FFmpeg thường sẽ báo thiếu encoder
        cmd.arg("-acodec").arg("libmp3lame")
           .arg("-q:a").arg("2"); 
    } else {
        // M4A/AAC
        cmd.arg("-acodec").arg("aac")
           .arg("-b:a").arg("192k");
    }
    
    cmd.arg("-y").arg(&output_path);

    // Debug: In command ra console của Tauri (Terminal) để xem
    println!("Running FFmpeg: {:?}", cmd);
    
    let child = cmd.spawn().map_err(|e| format!("Lỗi khi khởi chạy FFmpeg: {}", e))?;
    let child_arc = Arc::new(tokio::sync::Mutex::new(Some(child)));
    
    // Đăng ký process
    {
        let mut procs = processes.lock().unwrap();
        procs.entry(process_id.to_string()).or_insert_with(Vec::new).push(child_arc.clone());
    }
    
    // Đợi process hoàn thành và lấy output để debug nếu lỗi
    let output = {
        let mut child_lock = child_arc.lock().await;
        if let Some(child) = child_lock.take() {
            child.wait_with_output().await.map_err(|e| format!("Lỗi khi đợi ffmpeg: {}", e))?
        } else {
            return Err("Tiến trình đã bị dừng".to_string());
        }
    };
    
    // Cleanup
    {
        let mut procs = processes.lock().unwrap();
        if let Some(vec) = procs.get_mut(&process_id) {
            vec.retain(|p| !Arc::ptr_eq(p, &child_arc));
        }
    }
    
    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("FFmpeg Error: {}", error_msg));
    }
    
    Ok(format!("Trích xuất audio thành công: {}", output_path))
}
