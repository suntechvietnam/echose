use crate::models::FileInfo;
use std::path::PathBuf;

#[tauri::command]
pub async fn read_directory(path: String) -> Result<Vec<FileInfo>, String> {
    let dir_path = PathBuf::from(&path);
    
    if !dir_path.exists() {
        return Err(format!("Path không tồn tại: {}", path));
    }
    
    if !dir_path.is_dir() {
        return Err(format!("Path không phải là thư mục: {}", path));
    }
    
    let mut files = Vec::new();
    
    match std::fs::read_dir(&dir_path) {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    let file_path = entry.path();
                    let metadata = entry.metadata().ok();
                    let size = metadata.as_ref().and_then(|m| {
                        if m.is_file() {
                            Some(m.len())
                        } else {
                            None
                        }
                    });
                    
                    files.push(FileInfo {
                        name: file_path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("")
                            .to_string(),
                        path: file_path.to_string_lossy().to_string(),
                        is_dir: file_path.is_dir(),
                        size,
                    });
                }
            }
            Ok(files)
        }
        Err(e) => Err(format!("Lỗi đọc thư mục: {}", e)),
    }
}

#[tauri::command]
pub async fn check_file_exists(path: String) -> Result<bool, String> {
    Ok(PathBuf::from(&path).exists())
}

#[tauri::command]
pub async fn open_folder(path: String) -> Result<(), String> {
    use std::process::Command;
    
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Lỗi khi mở thư mục: {}", e))?;
        Ok(())
    }
    
    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Lỗi khi mở thư mục: {}", e))?;
        Ok(())
    }
    
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Lỗi khi mở thư mục: {}", e))?;
        Ok(())
    }
    
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        Err("Platform không được hỗ trợ".to_string())
    }
}

#[tauri::command]
pub async fn get_file_size(path: String) -> Result<u64, String> {
    let file_path = PathBuf::from(&path);
    
    if !file_path.exists() {
        return Err("File không tồn tại".to_string());
    }
    
    match std::fs::metadata(&file_path) {
        Ok(metadata) => Ok(metadata.len()),
        Err(e) => Err(format!("Lỗi đọc metadata: {}", e)),
    }
}

#[tauri::command]
pub async fn get_home_dir() -> Result<String, String> {
    match dirs::home_dir() {
        Some(path) => Ok(path.to_string_lossy().to_string()),
        None => Err("Không tìm thấy home directory".to_string()),
    }
}

#[tauri::command]
pub async fn get_download_dir() -> Result<String, String> {
    match dirs::download_dir() {
        Some(path) => Ok(path.to_string_lossy().to_string()),
        None => Err("Không tìm thấy download directory".to_string()),
    }
}

#[tauri::command]
pub async fn save_temp_file(file_name: String, file_data: Vec<u8>) -> Result<String, String> {
    use std::fs;
    use std::io::Write;
    
    let temp_dir = std::env::temp_dir();
    let temp_file_path = temp_dir.join(&file_name);
    
    // Write file data to temp file
    let mut file = fs::File::create(&temp_file_path)
        .map_err(|e| format!("Lỗi khi tạo file tạm: {}", e))?;
    
    file.write_all(&file_data)
        .map_err(|e| format!("Lỗi khi ghi file tạm: {}", e))?;
    
    Ok(temp_file_path.to_string_lossy().to_string())
}

