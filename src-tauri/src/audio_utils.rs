use std::process::Command;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

#[tauri::command]
pub async fn merge_audio_files(
    input_paths: Vec<String>,
    output_path: String,
) -> Result<String, String> {
    if input_paths.is_empty() {
        return Err("Danh sách tệp đầu vào trống".to_string());
    }

    if input_paths.len() == 1 {
        std::fs::copy(&input_paths[0], &output_path)
            .map_err(|e| format!("Lỗi khi copy file duy nhất: {}", e))?;
        return Ok(output_path);
    }

    let temp_dir = std::env::temp_dir();
    let list_file_path = temp_dir.join(format!("concat_list_{}.txt", uuid::Uuid::new_v4()));
    
    {
        let mut list_file = File::create(&list_file_path)
            .map_err(|e| format!("Lỗi tạo tệp danh sách nối: {}", e))?;

        for path in input_paths {
            // FFmpeg concat demuxer yêu cầu format: file 'path'
            let escaped_path = path.replace("'", "'\\''");
            writeln!(list_file, "file '{}'", escaped_path)
                .map_err(|e| format!("Lỗi ghi vào tệp danh sách: {}", e))?;
        }
        // File auto-flushes and closes here when block ends
    }

    let status = Command::new("ffmpeg")
        .arg("-f")
        .arg("concat")
        .arg("-safe")
        .arg("0")
        .arg("-i")
        .arg(&list_file_path)
        .arg("-c:a")
        .arg("libmp3lame")
        .arg("-y")
        .arg(&output_path)
        .status()
        .map_err(|e| format!("Lỗi thực thi FFmpeg: {}", e))?;

    if !status.success() {
        return Err("FFmpeg thất bại khi nối âm thanh".to_string());
    }

    // Xóa file list tạm
    let _ = std::fs::remove_file(list_file_path);

    Ok(output_path)
}

#[tauri::command]
pub async fn copy_external_file(
    src: String,
    dest: String,
) -> Result<String, String> {
    let src_path = std::path::Path::new(&src);
    if !src_path.exists() {
        return Err(format!("Tệp nguồn không tồn tại: {}", src));
    }
    
    std::fs::copy(&src, &dest)
        .map_err(|e| format!("Lỗi copy file từ {} sang {}: {}", src, dest, e))?;
    Ok(dest)
}
