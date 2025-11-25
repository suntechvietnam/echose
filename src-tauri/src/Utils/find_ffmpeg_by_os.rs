use std::path::PathBuf;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// Tạo và trả về một tokio::process::Command đã được setup với ffmpeg_path
/// Tự động tìm ffmpeg path và trả về lỗi nếu không tìm thấy
/// Trên Windows, tự động thêm CREATE_NO_WINDOW flag để ẩn console window
/// # Returns
/// * `Ok(tokio::process::Command)` - Command đã được setup với ffmpeg path
/// * `Err(String)` - Thông báo lỗi phù hợp theo từng OS
pub fn run_ffmpeg() -> Result<tokio::process::Command, String> {
    let ffmpeg_path = find_ffmpeg_or_error()?;
    
    #[cfg(target_os = "windows")]
    {
        Ok(tokio::process::Command::new(&ffmpeg_path).creation_flags(CREATE_NO_WINDOW))
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        Ok(tokio::process::Command::new(&ffmpeg_path))
    }
}

/// Tìm đường dẫn đến FFmpeg executable theo từng OS
pub fn find_ffmpeg() -> Option<String> {
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(app_dir) = exe_path.parent() {
            
            #[cfg(target_os = "windows")]
            {
                // Windows: Tìm ffmpeg.exe trong resources/window/
                let bundled_paths = vec![
                    app_dir.join("resources").join("window").join("ffmpeg.exe"),
                    app_dir.join("ffmpeg.exe"), // Fallback: cùng thư mục exe
                ];
                
                for path in bundled_paths {
                    if path.exists() && path.is_file() {
                        return Some(path.to_string_lossy().to_string());
                    }
                }
            }
            
            #[cfg(target_os = "macos")]
            {
                // macOS: Tìm trong app bundle Resources/resources/mac/
                if let Some(contents_dir) = app_dir.parent() {
                    let resources_dir = contents_dir.join("Resources");
                    
                    let bundled_paths = vec![
                        resources_dir.join("resources").join("mac").join("ffmpeg"),
                        resources_dir.join("ffmpeg"), // Fallback: Resources/ffmpeg
                    ];
                    
                    for path in bundled_paths {
                        if path.exists() && path.is_file() {
                            return Some(path.to_string_lossy().to_string());
                        }
                    }
                }
                
                // macOS fallback: System paths
                let system_paths = vec![
                    "/usr/local/bin/ffmpeg",
                    "/opt/homebrew/bin/ffmpeg",
                    "/usr/bin/ffmpeg",
                ];
                
                for path in system_paths {
                    if PathBuf::from(path).exists() {
                        return Some(path.to_string());
                    }
                }
            }
            
        }
    }
    
    // Cuối cùng: Thử FFmpeg trong system PATH
    if let Ok(output) = std::process::Command::new("ffmpeg").arg("-version").output() {
        if output.status.success() {
            return Some("ffmpeg".to_string());
        }
    }
    
    None
}

/// Tìm đường dẫn đến FFmpeg executable hoặc trả về lỗi với thông báo phù hợp theo OS
/// # Returns
/// * `Ok(String)` - Đường dẫn đến FFmpeg executable
/// * `Err(String)` - Thông báo lỗi phù hợp theo từng OS
pub fn find_ffmpeg_or_error() -> Result<String, String> {
    find_ffmpeg().ok_or_else(|| {
        #[cfg(target_os = "windows")]
        {
            "Lỗi: Không tìm thấy ffmpeg. Vui lòng tải lại ứng dụng hoặc liên hệ hỗ trợ.".to_string()
        }
        #[cfg(target_os = "macos")]
        {
            "Lỗi: Không tìm thấy ffmpeg. Vui lòng cài đặt: brew install ffmpeg".to_string()
        }
        #[cfg(target_os = "linux")]
        {
            "Lỗi: Không tìm thấy ffmpeg. Vui lòng cài đặt: sudo apt install ffmpeg".to_string()
        }
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            "Lỗi: Không tìm thấy ffmpeg. Vui lòng cài đặt ffmpeg.".to_string()
        }
    })
}
