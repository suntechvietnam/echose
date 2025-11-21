use std::path::PathBuf;

#[tauri::command]
pub async fn select_files(file_types: Vec<String>, multiple: bool) -> Result<Vec<String>, String> {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        
        // Store file types for filtering later
        let file_types_lower: Vec<String> = file_types.iter()
            .map(|ext| ext.to_lowercase())
            .collect();
        
        // Don't filter in AppleScript - let user choose freely, then filter in Rust
        // This avoids issues with macOS file type recognition
        let prompt_msg = format!("Chọn file ({}): {}", 
            file_types_lower.join("/"),
            if multiple { "có thể chọn nhiều" } else { "" }
        );
        
        let script = if multiple {
            format!(
                r#"
                try
                    set theFiles to choose file with prompt "{}" with multiple selections allowed
                    set fileList to {{}}
                    repeat with aFile in theFiles
                        set end of fileList to POSIX path of aFile
                    end repeat
                    return fileList
                on error errMsg
                    return ""
                end try
                "#,
                prompt_msg
            )
        } else {
            format!(
                r#"
                try
                    set theFile to choose file with prompt "{}"
                    return POSIX path of theFile
                on error errMsg
                    return ""
                end try
                "#,
                prompt_msg
            )
        };
        
        let output = Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .output();
        
        match output {
            Ok(output) => {
                if output.status.success() {
                    let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    
                    if result.is_empty() {
                        return Err("Người dùng đã hủy chọn file".to_string());
                    }
                    
                    if multiple {
                        // Parse AppleScript list result
                        // Format: "/path1, /path2, /path3" or as a list
                        let mut files: Vec<String> = result
                            .split(", ")
                            .map(|s| {
                                s.trim()
                                    .trim_matches('"')
                                    .trim_matches('\'')
                                    .to_string()
                            })
                            .filter(|s| !s.is_empty() && s.starts_with('/'))
                            .collect();
                        
                        if files.is_empty() && !result.is_empty() {
                            // Try single file
                            files = vec![result];
                        }
                        
                        // Filter files by extension (case-insensitive)
                        let filtered_files: Vec<String> = files.into_iter()
                            .filter(|file| {
                                let path = PathBuf::from(file);
                                if let Some(ext) = path.extension() {
                                    let ext_str = ext.to_string_lossy().to_lowercase();
                                    file_types_lower.contains(&ext_str)
                                } else {
                                    false
                                }
                            })
                            .collect();
                        
                        if filtered_files.is_empty() {
                            Err("Không có file nào phù hợp với định dạng đã chọn".to_string())
                        } else {
                            Ok(filtered_files)
                        }
                    } else {
                        // Filter single file
                        let path = PathBuf::from(&result);
                        if let Some(ext) = path.extension() {
                            let ext_str = ext.to_string_lossy().to_lowercase();
                            if file_types_lower.contains(&ext_str) {
                                Ok(vec![result])
                            } else {
                                Err(format!("File không phù hợp. Cần file có đuôi: {}", file_types_lower.join(", ")))
                            }
                        } else {
                            Err("File không có phần mở rộng".to_string())
                        }
                    }
                } else {
                    let error_msg = String::from_utf8_lossy(&output.stderr);
                    if error_msg.contains("User canceled") || error_msg.is_empty() {
                        Err("Người dùng đã hủy chọn file".to_string())
                    } else {
                        Err(format!("Lỗi: {}", error_msg))
                    }
                }
            }
            Err(e) => Err(format!("Lỗi mở dialog: {}", e)),
        }
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        Err("Platform không được hỗ trợ".to_string())
    }
}

#[tauri::command]
pub async fn select_folder() -> Result<Option<String>, String> {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        
        // Use macOS native dialog via osascript
        let output = Command::new("osascript")
            .arg("-e")
            .arg("set theFolder to choose folder with prompt \"Chọn thư mục để lưu video\"")
            .arg("-e")
            .arg("return POSIX path of theFolder")
            .output();
        
        match output {
            Ok(output) => {
                if output.status.success() {
                    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !path.is_empty() {
                        return Ok(Some(path));
                    }
                }
            }
            Err(e) => {
                return Err(format!("Lỗi mở dialog: {}", e));
            }
        }
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        // For other platforms, return error to use frontend dialog
        return Err("Platform không được hỗ trợ. Vui lòng sử dụng dialog từ frontend.".to_string());
    }
    
    Ok(None)
}

