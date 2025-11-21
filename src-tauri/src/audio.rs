use crate::process::ProcessStore;
use std::path::PathBuf;
use rand::seq::SliceRandom;
use rand::thread_rng;

pub fn find_ffmpeg() -> Option<String> {
    // Ưu tiên tìm trong app bundle trước (khi được bundle vào app)
    // Trên macOS: App.app/Contents/Resources/resources/ffmpeg hoặc App.app/Contents/Resources/ffmpeg
    if let Ok(exe_path) = std::env::current_exe() {
        // Tìm trong app bundle: ../Resources/resources/ffmpeg hoặc ../Resources/ffmpeg
        if let Some(app_dir) = exe_path.parent() {
            // Trong app bundle: Contents/MacOS/app -> Contents/Resources/
            if let Some(contents_dir) = app_dir.parent() {
                let resources_dir = contents_dir.join("Resources");
                
                // Thử tìm trong resources/ffmpeg (khi được bundle)
                let bundled_ffmpeg1 = resources_dir.join("resources").join("ffmpeg");
                if bundled_ffmpeg1.exists() {
                    // Không cần canonicalize, dùng đường dẫn trực tiếp
                    return Some(bundled_ffmpeg1.to_string_lossy().to_string());
                }
                
                // Thử tìm trực tiếp trong Resources/ffmpeg
                let bundled_ffmpeg2 = resources_dir.join("ffmpeg");
                if bundled_ffmpeg2.exists() {
                    // Không cần canonicalize, dùng đường dẫn trực tiếp
                    return Some(bundled_ffmpeg2.to_string_lossy().to_string());
                }
            }
            
            // Fallback: tìm trong cùng thư mục với executable (cho dev mode)
            let local_ffmpeg = app_dir.join("ffmpeg");
            if local_ffmpeg.exists() {
                return Some(local_ffmpeg.to_string_lossy().to_string());
            }
        }
    }
    
    // Tìm trong system paths
    let possible_paths = vec![
        "/usr/local/bin/ffmpeg",
        "/opt/homebrew/bin/ffmpeg",
        "/usr/bin/ffmpeg",
        "ffmpeg", // In PATH
    ];
    
    for path in possible_paths {
        if path == "ffmpeg" {
            return Some(path.to_string());
        }
        if PathBuf::from(path).exists() {
            if std::fs::metadata(path).map(|m| m.is_file()).unwrap_or(false) {
                return Some(path.to_string());
            }
        }
    }
    
    None
}

pub fn find_ffprobe() -> Option<String> {
    // Ưu tiên tìm trong app bundle trước (khi được bundle vào app)
    // Trên macOS: App.app/Contents/Resources/resources/ffprobe hoặc App.app/Contents/Resources/ffprobe
    if let Ok(exe_path) = std::env::current_exe() {
        // Tìm trong app bundle: ../Resources/resources/ffprobe hoặc ../Resources/ffprobe
        if let Some(app_dir) = exe_path.parent() {
            // Trong app bundle: Contents/MacOS/app -> Contents/Resources/
            if let Some(contents_dir) = app_dir.parent() {
                let resources_dir = contents_dir.join("Resources");
                
                // Thử tìm trong resources/ffprobe (khi được bundle)
                let bundled_ffprobe1 = resources_dir.join("resources").join("ffprobe");
                if bundled_ffprobe1.exists() {
                    // Không cần canonicalize, dùng đường dẫn trực tiếp
                    return Some(bundled_ffprobe1.to_string_lossy().to_string());
                }
                
                // Thử tìm trực tiếp trong Resources/ffprobe
                let bundled_ffprobe2 = resources_dir.join("ffprobe");
                if bundled_ffprobe2.exists() {
                    // Không cần canonicalize, dùng đường dẫn trực tiếp
                    return Some(bundled_ffprobe2.to_string_lossy().to_string());
                }
            }
            
            // Fallback: tìm trong cùng thư mục với executable (cho dev mode)
            let local_ffprobe = app_dir.join("ffprobe");
            if local_ffprobe.exists() {
                return Some(local_ffprobe.to_string_lossy().to_string());
            }
        }
    }
    
    // Tìm trong system paths
    let possible_paths = vec![
        "/usr/local/bin/ffprobe",
        "/opt/homebrew/bin/ffprobe",
        "/usr/bin/ffprobe",
        "ffprobe", // In PATH
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
    
    None
}

/// Tạo danh sách bài nhạc được xáo trộn thông minh
/// Đảm bảo không có bài nào lặp lại liền nhau
/// Và đảm bảo bài cuối và bài đầu không trùng nhau (để khi loop không bị lặp)
fn create_shuffled_playlist(
    audio_files: &[String],
    target_count: usize,
) -> Vec<String> {
    if audio_files.is_empty() {
        return Vec::new();
    }
    
    if audio_files.len() == 1 {
        // Nếu chỉ có 1 bài, không thể tránh lặp lại
        return vec![audio_files[0].clone(); target_count];
    }
    
    let mut playlist = Vec::new();
    let mut rng = thread_rng();
    
    // Tạo danh sách ban đầu với shuffle
    let mut available_files: Vec<String> = audio_files.to_vec();
    
    // Lặp lại cho đến khi đủ số lượng
    while playlist.len() < target_count {
        // Shuffle danh sách
        available_files.shuffle(&mut rng);
        
        // Thêm từng bài vào playlist, kiểm tra không lặp lại liền nhau
        for file in &available_files {
            if playlist.len() >= target_count {
                break;
            }
            
            // Kiểm tra bài cuối cùng trong playlist có trùng với bài hiện tại không
            let can_add = if let Some(last_file) = playlist.last() {
                last_file != file
            } else {
                true
            };
            
            if can_add {
                playlist.push(file.clone());
            } else if available_files.len() > 1 {
                // Nếu bài cuối cùng trùng, bỏ qua và tiếp tục với bài khác
                continue;
            }
        }
        
        // Nếu đã hết bài để thêm nhưng chưa đủ số lượng và chỉ có 2 bài
        if playlist.len() < target_count && available_files.len() == 2 {
            // Đảm bảo bài cuối và bài đầu khác nhau
            if let (Some(first), Some(last)) = (playlist.first(), playlist.last()) {
                if first == last && playlist.len() > 1 {
                    // Swap bài cuối với bài khác
                    let len = playlist.len();
                    if len >= 2 {
                        playlist.swap(len - 1, len - 2);
                    }
                }
            }
        }
    }
    
    // Đảm bảo không có bài nào lặp lại liền nhau (double check)
    for i in 1..playlist.len() {
        if playlist[i] == playlist[i - 1] {
            // Tìm bài khác để swap
            for j in (i + 1)..playlist.len() {
                if playlist[j] != playlist[i - 1] {
                    playlist.swap(i, j);
                    break;
                }
            }
            // Nếu không tìm thấy ở sau, tìm ở trước
            if playlist[i] == playlist[i - 1] {
                for j in 0..(i - 1) {
                    if playlist[j] != playlist[i - 1] && (j == 0 || playlist[j - 1] != playlist[i]) {
                        playlist.swap(i, j);
                        break;
                    }
                }
            }
        }
    }
    
    // Đảm bảo bài cuối và bài đầu không trùng nhau (quan trọng khi loop)
    if playlist.len() >= 2 {
        let len = playlist.len();
        if playlist[0] == playlist[len - 1] {
            // Tìm bài khác để swap với bài cuối
            for i in (1..len - 1).rev() {
                if playlist[i] != playlist[0] && playlist[i] != playlist[len - 2] {
                    playlist.swap(len - 1, i);
                    break;
                }
            }
        }
    }
    
    playlist.truncate(target_count);
    playlist
}


#[tauri::command]
pub async fn create_audio_mix(
    audio_files: Vec<String>,
    random_count: usize,
    duration_seconds: u64,
    output_folder: String,
    output_filename: String,
    processes: tauri::State<'_, ProcessStore>,
) -> Result<String, String> {
    if audio_files.is_empty() {
        return Err("Cần ít nhất một file nhạc".to_string());
    }
    
    if random_count == 0 {
        return Err("Số bài nhạc phải lớn hơn 0".to_string());
    }
    
    if duration_seconds == 0 {
        return Err("Thời gian phải lớn hơn 0".to_string());
    }
    
    // Find ffmpeg
    let ffmpeg_path = find_ffmpeg().ok_or_else(|| {
        "Không tìm thấy ffmpeg. Vui lòng cài đặt: brew install ffmpeg".to_string()
    })?;
    
    // Validate output folder
    let output_path = PathBuf::from(&output_folder);
    if !output_path.exists() || !output_path.is_dir() {
        return Err("Thư mục output không hợp lệ".to_string());
    }
    
    let output_file = output_path.join(&output_filename);
    let output_file_str = output_file.to_string_lossy().to_string();
    
    // Tạo danh sách bài nhạc được xáo trộn thông minh
    // Ước tính số lượng file cần thiết: giả sử mỗi bài trung bình 3-4 phút (180-240 giây)
    // Để đảm bảo đủ thời gian, tạo playlist dài hơn khoảng 2-3 lần thời gian mong muốn
    let estimated_files_needed = (duration_seconds / 180).max(10) as usize; // Ít nhất 10 bài
    let shuffled_playlist = create_shuffled_playlist(&audio_files, estimated_files_needed.max(random_count));
    
    // Generate timestamp trước để dùng cho cả process_id và concat file
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    // Tạo file list cho concat demuxer với playlist đủ dài
    let temp_dir = std::env::temp_dir();
    let concat_list_file = temp_dir.join(format!("concat_list_{}.txt", timestamp));
    
    // Tạo nội dung file list cho concat demuxer
    // Lặp lại playlist nhiều lần để đảm bảo đủ thời gian
    let mut concat_content = String::new();
    let repeat_times = (duration_seconds / 180).max(3) as usize; // Lặp lại ít nhất 3 lần
    
    for _ in 0..repeat_times {
        for file in &shuffled_playlist {
            // Escape single quotes và backslashes cho file path
            let escaped_path = file.replace('\\', "\\\\").replace('\'', "'\\''");
            concat_content.push_str(&format!("file '{}'\n", escaped_path));
        }
    }
    
    // Ghi file list
    std::fs::write(&concat_list_file, concat_content)
        .map_err(|e| format!("Lỗi khi tạo file concat list: {}", e))?;
    
    // Build ffmpeg command với concat demuxer
    // Decode về PCM trước (qua filter) để reset timestamp, sau đó encode lại
    // Format: ffmpeg -f concat -safe 0 -i concat_list.txt -af "aresample=async=1" -t duration -c:a libmp3lame -b:a 192k output.mp3
    let mut cmd = tokio::process::Command::new(&ffmpeg_path);
    
    cmd.arg("-f")
        .arg("concat")
        .arg("-safe")
        .arg("0") // Cho phép absolute paths
        .arg("-i")
        .arg(concat_list_file.to_string_lossy().as_ref())
        .arg("-af")
        .arg("aresample=async=1") // Resample để reset timestamp và sync audio
        .arg("-t")
        .arg(duration_seconds.to_string()) // Thời gian output (giây) - sẽ tự động cắt khi đủ
        .arg("-c:a")
        .arg("libmp3lame") // Encode với MP3 codec
        .arg("-b:a")
        .arg("192k") // Bitrate 192kbps cho chất lượng tốt
        .arg("-y") // Overwrite output file
        .arg(&output_file_str);
    
    // Generate process ID với timestamp đã tạo ở trên
    let process_id = format!("audio_{}", timestamp);
    
    // Spawn process
    let child = cmd
        .spawn()
        .map_err(|e| format!("Lỗi khi chạy ffmpeg: {}", e))?;
    
    // Store process handle
    {
        let mut procs = processes.lock().unwrap();
        procs.insert(process_id.clone(), child);
    }
    
    // Note: Concat list file sẽ được cleanup trong wait_for_audio_creation
    
    Ok(process_id)
}

#[tauri::command]
pub async fn wait_for_audio_creation(
    process_id: String,
    processes: tauri::State<'_, ProcessStore>,
) -> Result<String, String> {
    let child_opt = {
        let mut procs = processes.lock().unwrap();
        procs.remove(&process_id)
    };
    
    if let Some(child) = child_opt {
        let output = child.wait_with_output().await
            .map_err(|e| format!("Lỗi khi chờ process: {}", e))?;
        
        // Cleanup concat list file nếu có
        let temp_dir = std::env::temp_dir();
        if let Some(timestamp_str) = process_id.strip_prefix("audio_") {
            let concat_file = temp_dir.join(format!("concat_list_{}.txt", timestamp_str));
            let _ = std::fs::remove_file(&concat_file);
        }
        
        if output.status.success() {
            Ok("✅ File nhạc đã được tạo thành công!".to_string())
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            Err(format!("Lỗi khi tạo file nhạc: {}", error_msg))
        }
    } else {
        Err("Process không tồn tại hoặc đã bị hủy".to_string())
    }
}

#[tauri::command]
pub async fn stop_audio_creation(
    process_id: String,
    processes: tauri::State<'_, ProcessStore>,
) -> Result<String, String> {
    let child_opt = {
        let mut procs = processes.lock().unwrap();
        procs.remove(&process_id)
    };
    
    if let Some(mut child) = child_opt {
        // Kill the process
        child.kill().await
            .map_err(|e| format!("Lỗi khi dừng process: {}", e))?;
        
        Ok("Đã dừng quá trình tạo file nhạc".to_string())
    } else {
        Err("Process không tồn tại".to_string())
    }
}

#[tauri::command]
pub async fn get_audio_duration(file_path: String) -> Result<f64, String> {
    let file = PathBuf::from(&file_path);
    
    if !file.exists() {
        return Err("File không tồn tại".to_string());
    }
    
    let ffprobe_path = find_ffprobe().ok_or_else(|| {
        "Không tìm thấy ffprobe. Vui lòng cài đặt: brew install ffmpeg".to_string()
    })?;
    
    // Sử dụng ffprobe để lấy duration
    // Command: ffprobe -v error -show_entries format=duration -of default=noprint_wrappers=1:nokey=1 file.mp3
    let output = tokio::process::Command::new(&ffprobe_path)
        .arg("-v")
        .arg("error")
        .arg("-show_entries")
        .arg("format=duration")
        .arg("-of")
        .arg("default=noprint_wrappers=1:nokey=1")
        .arg(&file_path)
        .output()
        .await
        .map_err(|e| format!("Lỗi khi chạy ffprobe: {}", e))?;
    
    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Lỗi khi lấy duration: {}", error_msg));
    }
    
    let duration_str = String::from_utf8_lossy(&output.stdout)
        .trim()
        .to_string();
    
    duration_str.parse::<f64>()
        .map_err(|e| format!("Lỗi khi parse duration: {}", e))
}

