use crate::process::ProcessStore;
use std::path::PathBuf;
use std::fs;
use std::io::Write;

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

#[tauri::command]
pub async fn start_create_video(
    audio_files: Vec<String>,
    _image_files: Vec<String>, // Reserved for future use
    video_files: Vec<String>,
    output_folder: String,
    output_filename: String,
    quality: String,
    processes: tauri::State<'_, ProcessStore>,
) -> Result<String, String> {
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
    
    // For now, implement simple case: video + audio (like the example)
    // We'll use the first video and first audio file
    if video_files.is_empty() {
        return Err("Cần ít nhất một file video".to_string());
    }
    
    if audio_files.is_empty() {
        return Err("Cần ít nhất một file audio".to_string());
    }
    
    let video_file = &video_files[0];
    let audio_file = &audio_files[0];
    
    // Build ffmpeg command based on user's example
    // ffmpeg -stream_loop -1 -i tmp_03.mp4 -i mp3_150min.mp3 \
    //   -t 9000 \
    //   -filter_complex "[0:a]volume=0.3[vid_audio];[vid_audio][1:a]amix=inputs=2:duration=longest:dropout_transition=3" \
    //   -c:v copy -c:a aac -b:a 192k \
    //   output_video_4k_150min_mix.mp4
    
    let mut cmd = tokio::process::Command::new(&ffmpeg_path);
    
    cmd.arg("-stream_loop")
        .arg("-1") // Loop video infinitely
        .arg("-i")
        .arg(video_file)
        .arg("-i")
        .arg(audio_file);
    
    // Build filter_complex for audio mixing
    let filter_complex = "[0:a]volume=0.3[vid_audio];[vid_audio][1:a]amix=inputs=2:duration=longest:dropout_transition=3";
    
    // Determine if we need to scale (and thus re-encode video)
    let needs_scale = match quality.as_str() {
        "1080p" | "720p" | "480p" | "360p" => true,
        _ => false,
    };
    
    if needs_scale {
        // Need to re-encode video, so build video filter
        let scale_filter = match quality.as_str() {
            "1080p" => "scale=1920:1080",
            "720p" => "scale=1280:720",
            "480p" => "scale=854:480",
            "360p" => "scale=640:360",
            _ => "",
        };
        
        if !scale_filter.is_empty() {
            cmd.arg("-vf").arg(scale_filter);
        }
        
        // Re-encode video with h264 codec
        cmd.arg("-c:v")
            .arg("libx264")
            .arg("-preset")
            .arg("medium")
            .arg("-crf")
            .arg("23"); // Good quality
    } else {
        // No scaling, can copy video codec
        cmd.arg("-c:v").arg("copy");
    }
    
    // Add audio filter and encoding
    cmd.arg("-filter_complex")
        .arg(filter_complex)
        .arg("-c:a")
        .arg("aac")
        .arg("-b:a")
        .arg("192k")
        .arg("-y") // Overwrite output file
        .arg(&output_file_str);
    
    // Spawn process instead of waiting
    let child = cmd
        .spawn()
        .map_err(|e| format!("Lỗi khi chạy ffmpeg: {}", e))?;
    
    // Generate process ID
    let process_id = format!("video_{}", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs());
    
    // Store process handle
    {
        let mut procs = processes.lock().unwrap();
        procs.insert(process_id.clone(), child);
    }
    
    Ok(process_id)
}

#[tauri::command]
pub async fn wait_for_video_creation(
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
        
        if output.status.success() {
            // Nếu là process từ image_to_video, cleanup các file tạm và trả về message với tên file
            if process_id.starts_with("images_video_") {
                // Lấy output_file_path để trả về tên file
                let video_file_path = crate::image_to_video::get_output_file_path(&process_id);
                let video_file_name = video_file_path
                    .as_ref()
                    .and_then(|path| {
                        std::path::Path::new(path)
                            .file_name()
                            .and_then(|name| name.to_str())
                            .map(|s| s.to_string())
                    })
                    .unwrap_or_else(|| "video.mp4".to_string());
                
                // Cleanup output_folder và output_file từ HashMap
                crate::image_to_video::cleanup_output_folder_from_map(&process_id);
                crate::image_to_video::cleanup_output_file_from_map(&process_id);
                
                if let Err(e) = cleanup_images_workdir(&process_id).await {
                    eprintln!("Cảnh báo: Không thể cleanup workdir: {}", e);
                }
                
                return Ok(format!("Video đã được tạo thành công! {}", video_file_name));
            }
            
            Ok("Video đã được tạo thành công!".to_string())
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            Err(format!("Lỗi khi tạo video: {}", error_msg))
        }
    } else {
        Err("Process không tồn tại hoặc đã bị hủy".to_string())
    }
}

/// Cleanup work directory và các file tạm sau khi tạo video từ ảnh thành công
async fn cleanup_images_workdir(process_id: &str) -> Result<(), String> {
    use std::path::PathBuf;
    
    // Parse timestamp từ process_id: images_video_{timestamp}
    let timestamp = process_id.strip_prefix("images_video_")
        .ok_or_else(|| "Invalid process ID format".to_string())?;
    
    // Tìm work_dir bằng cách scan output_folder từ cleanup_info.txt
    // Tìm trong các thư mục phổ biến để tìm cleanup_info.txt
    let possible_dirs = vec![
        std::env::var("HOME").ok().map(|h| PathBuf::from(h).join("Downloads")),
        std::env::var("HOME").ok().map(|h| PathBuf::from(h).join("Desktop")),
        std::env::var("HOME").ok().map(|h| PathBuf::from(h)),
    ];
    
    for maybe_dir in possible_dirs {
        if let Some(base_dir) = maybe_dir {
            let work_dir = base_dir.join(format!("ytbflow_images_{}", timestamp));
            if work_dir.exists() && work_dir.is_dir() {
                // Đọc cleanup_info.txt để lấy output_folder
                let cleanup_info_file = work_dir.join("cleanup_info.txt");
                if cleanup_info_file.exists() {
                    if let Ok(content) = fs::read_to_string(&cleanup_info_file) {
                        let parts: Vec<&str> = content.trim().split('|').collect();
                        if parts.len() >= 1 {
                            // output_folder là phần đầu tiên
                            let output_folder = parts[0];
                            let output_path = PathBuf::from(output_folder);
                            
                            // Tìm lại work_dir từ output_folder chính xác
                            let correct_work_dir = output_path.join(format!("ytbflow_images_{}", timestamp));
                            
                            if correct_work_dir.exists() && correct_work_dir.is_dir() {
                                // Xóa toàn bộ folder work_dir (bao gồm tất cả file và subfolder)
                                if let Err(e) = fs::remove_dir_all(&correct_work_dir) {
                                    return Err(format!("Không thể xóa folder {}: {}", correct_work_dir.display(), e));
                                }
                                
                                return Ok(());
                            }
                        }
                    }
                }
                
                // Nếu không có cleanup_info.txt, xóa toàn bộ folder hiện tại
                if let Err(e) = fs::remove_dir_all(&work_dir) {
                    eprintln!("Không thể xóa folder {}: {}", work_dir.display(), e);
                } else {
                    return Ok(());
                }
            }
        }
    }
    
    Ok(())
}

#[tauri::command]
pub async fn stop_video_creation(
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
        
        // Đợi process kết thúc để giải phóng tài nguyên
        let _ = child.wait().await;
        
        Ok("Đã dừng quá trình tạo video".to_string())
    } else {
        Err("Process không tồn tại".to_string())
    }
}

/// Dừng tất cả các process video đang chạy và giải phóng bộ nhớ
#[tauri::command]
pub async fn stop_all_video_creation(
    processes: tauri::State<'_, ProcessStore>,
) -> Result<String, String> {
    let mut process_ids = Vec::new();
    
    // Lấy danh sách tất cả process ID liên quan đến video
    {
        let procs = processes.lock().unwrap();
        for (id, _) in procs.iter() {
            if id.starts_with("step1_mp3_") || 
               id.starts_with("step2_video_") || 
               id.starts_with("step3_mix_") ||
               id.starts_with("video_") {
                process_ids.push(id.clone());
            }
        }
    }
    
    // Dừng từng process
    let mut stopped_count = 0;
    let mut errors = Vec::new();
    
    for process_id in process_ids {
        let child_opt = {
            let mut procs = processes.lock().unwrap();
            procs.remove(&process_id)
        };
        
        if let Some(mut child) = child_opt {
            match child.kill().await {
                Ok(_) => {
                    // Đợi process kết thúc để giải phóng tài nguyên
                    let _ = child.wait().await;
                    stopped_count += 1;
                }
                Err(e) => {
                    errors.push(format!("{}: {}", process_id, e));
                }
            }
        }
    }
    
    if stopped_count > 0 {
        Ok(format!("Đã dừng {} process và giải phóng bộ nhớ", stopped_count))
    } else if !errors.is_empty() {
        Err(format!("Lỗi khi dừng: {}", errors.join(", ")))
    } else {
        Ok("Không có process nào đang chạy".to_string())
    }
}

/// Tạo file concat list cho ffmpeg
fn create_concat_file(files: &[String], temp_dir: &PathBuf) -> Result<PathBuf, String> {
    let concat_file = temp_dir.join("concat_list.txt");
    let mut file = fs::File::create(&concat_file)
        .map_err(|e| format!("Lỗi khi tạo file concat: {}", e))?;
    
    for file_path in files {
        // Escape single quotes và format cho concat demuxer
        let escaped_path = file_path.replace('\'', "'\\''");
        writeln!(file, "file '{}'", escaped_path)
            .map_err(|e| format!("Lỗi khi ghi file concat: {}", e))?;
    }
    
    Ok(concat_file)
}

// Step 1: Ghép MP3 files thành 1 file
#[tauri::command]
pub async fn create_merged_mp3_step1(
    audio_files: Vec<String>,
    output_folder: String,
    processes: tauri::State<'_, ProcessStore>,
) -> Result<String, String> {
    if audio_files.is_empty() {
        return Err("Cần ít nhất một file MP3".to_string());
    }
    
    let ffmpeg_path = find_ffmpeg().ok_or_else(|| {
        "Không tìm thấy ffmpeg. Vui lòng cài đặt: brew install ffmpeg".to_string()
    })?;
    
    // Validate output folder
    let output_path = PathBuf::from(&output_folder);
    if !output_path.exists() || !output_path.is_dir() {
        return Err("Thư mục output không hợp lệ".to_string());
    }
    
    // Tạo timestamp để tránh conflict
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    // Lưu vào output folder thay vì temp directory
    let work_dir = output_path.join(format!("ytbflow_step1_{}", timestamp));
    fs::create_dir_all(&work_dir)
        .map_err(|e| format!("Lỗi khi tạo work directory: {}", e))?;
    
    // Sử dụng concat filter thay vì concat demuxer để kiểm soát timestamp tốt hơn
    // Cách này decode từng file và concat lại, đảm bảo DTS tăng đơn điệu
    let merged_mp3 = work_dir.join("merged_audio.mp3");
    
    // Build filter_complex để concat tất cả audio files
    // Format: [0:a][1:a][2:a]...concat=n=N:v=0:a=1[outa]
    let mut filter_parts = Vec::new();
    let mut input_args = Vec::new();
    
    for (index, audio_file) in audio_files.iter().enumerate() {
        input_args.push("-i".to_string());
        input_args.push(audio_file.clone());
        filter_parts.push(format!("[{}:a]", index));
    }
    
    let n_inputs = audio_files.len();
    let filter_complex = format!("{}concat=n={}:v=0:a=1[outa]", 
        filter_parts.join(""), n_inputs);
    
    let mut cmd = tokio::process::Command::new(&ffmpeg_path);
    
    // Thêm tất cả input files
    for arg in input_args {
        cmd.arg(arg);
    }
    
    // Áp dụng concat filter
    cmd.arg("-filter_complex")
        .arg(&filter_complex)
        .arg("-map")
        .arg("[outa]")
        .arg("-c:a")
        .arg("libmp3lame")  // Re-encode MP3
        .arg("-b:a")
        .arg("192k")  // Bitrate 192kbps
        .arg("-ar")
        .arg("44100")  // Sample rate cố định
        .arg("-ac")
        .arg("2")  // Stereo
        .arg("-y")
        .arg(merged_mp3.to_string_lossy().to_string());
    
    // Spawn process
    let child = cmd.spawn()
        .map_err(|e| format!("Lỗi khi chạy ffmpeg: {}", e))?;
    
    // Generate process ID với timestamp và output folder
    let process_id = format!("step1_mp3_{}_{}", timestamp, 
        output_folder.replace('/', "_").replace('\\', "_"));
    
    // Store process handle
    {
        let mut procs = processes.lock().unwrap();
        procs.insert(process_id.clone(), child);
    }
    
    Ok(process_id)
}

// Step 2: Ép từng video về 2K, kéo dài thời gian, sau đó concat và xóa file trung gian
#[tauri::command]
pub async fn create_merged_video_step2(
    video_files: Vec<String>,
    mp3_duration: f64,
    output_folder: String,
    processes: tauri::State<'_, ProcessStore>,
) -> Result<String, String> {
    if video_files.is_empty() {
        return Err("Cần ít nhất một file MP4".to_string());
    }
    
    let ffmpeg_path = find_ffmpeg().ok_or_else(|| {
        "Không tìm thấy ffmpeg. Vui lòng cài đặt: brew install ffmpeg".to_string()
    })?;
    
    // Validate output folder
    let output_path = PathBuf::from(&output_folder);
    if !output_path.exists() || !output_path.is_dir() {
        return Err("Thư mục output không hợp lệ".to_string());
    }
    
    // Tính thời gian cho mỗi video (chia đều tổng thời lượng MP3)
    let duration_per_video = mp3_duration / video_files.len() as f64;
    
    // Tạo timestamp để tránh conflict
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    // Lưu vào output folder thay vì temp directory
    let work_dir = output_path.join(format!("ytbflow_step2_{}", timestamp));
    fs::create_dir_all(&work_dir)
        .map_err(|e| format!("Lỗi khi tạo work directory: {}", e))?;
    
    // Bước 1: Ép từng video về 2K và kéo dài thời gian
    let mut converted_videos = Vec::new();
    for (index, video_file) in video_files.iter().enumerate() {
        let output_2k = work_dir.join(format!("output{}_2k.mp4", index + 1));
        let output_2k_str = output_2k.to_string_lossy().to_string();
        
        // Ép video về 2K với scale và pad, kéo dài thời gian bằng cách loop
        let mut cmd = tokio::process::Command::new(&ffmpeg_path);
        cmd.arg("-stream_loop")
            .arg("-1") // Loop video vô hạn
            .arg("-i")
            .arg(video_file)
            .arg("-vf")
            .arg("scale=2048:1080:force_original_aspect_ratio=decrease,pad=2048:1080:(ow-iw)/2:(oh-ih)/2,fps=30")
            .arg("-t")
            .arg(format!("{:.2}", duration_per_video)) // Thời gian tính bằng giây (có thể là số thập phân)
            .arg("-c:v")
            .arg("libx264")
            .arg("-preset")
            .arg("veryfast")
            .arg("-crf")
            .arg("18")
            .arg("-c:a")
            .arg("aac")
            .arg("-b:a")
            .arg("192k")
            .arg("-y")
            .arg(&output_2k_str);
        
        // Chạy và đợi process hoàn thành
        let output = cmd.output().await
            .map_err(|e| format!("Lỗi khi chạy ffmpeg cho video {}: {}", index + 1, e))?;
        
        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Lỗi khi ép video {}: {}", index + 1, error_msg));
        }
        
        converted_videos.push(output_2k_str);
    }
    
    // Bước 2: Concat tất cả video đã ép
    let video_concat_file = create_concat_file(&converted_videos, &work_dir)?;
    let merged_video = work_dir.join("merged_video.mp4");
    
    // Concat video với concat demuxer (copy stream - nhanh vì đã cùng format)
    let mut cmd = tokio::process::Command::new(&ffmpeg_path);
    cmd.arg("-f")
        .arg("concat")
        .arg("-safe")
        .arg("0")
        .arg("-i")
        .arg(video_concat_file.to_string_lossy().to_string())
        .arg("-c:v")
        .arg("copy")  // Copy video stream - không re-encode, rất nhanh
        .arg("-c:a")
        .arg("copy")  // Copy audio stream
        .arg("-y")
        .arg(merged_video.to_string_lossy().to_string());
    
    // Spawn process cho concat
    let child = cmd.spawn()
        .map_err(|e| format!("Lỗi khi chạy ffmpeg concat: {}", e))?;
    
    // Generate process ID với timestamp và output folder
    let process_id = format!("step2_video_{}_{}", timestamp,
        output_folder.replace('/', "_").replace('\\', "_"));
    
    // Store process handle và thông tin để cleanup sau
    {
        let mut procs = processes.lock().unwrap();
        // Store converted videos paths để cleanup sau khi concat xong
        // Sử dụng một wrapper để lưu thông tin cleanup
        procs.insert(process_id.clone(), child);
    }
    
    // Lưu danh sách file cần xóa vào một file metadata
    let cleanup_file = work_dir.join("cleanup_files.txt");
    let mut cleanup_file_handle = fs::File::create(&cleanup_file)
        .map_err(|e| format!("Lỗi khi tạo cleanup file: {}", e))?;
    for converted_video in &converted_videos {
        writeln!(cleanup_file_handle, "{}", converted_video)
            .map_err(|e| format!("Lỗi khi ghi cleanup file: {}", e))?;
    }
    writeln!(cleanup_file_handle, "{}", video_concat_file.to_string_lossy())
        .map_err(|e| format!("Lỗi khi ghi cleanup file: {}", e))?;
    
    Ok(process_id)
}

// Step 3: Mix audio giữa MP3 và MP4 đã tạo
#[tauri::command]
pub async fn mix_video_audio_step3(
    merged_mp3_path: String,
    merged_video_path: String,
    output_folder: String,
    output_filename: String,
    processes: tauri::State<'_, ProcessStore>,
) -> Result<String, String> {
    let ffmpeg_path = find_ffmpeg().ok_or_else(|| {
        "Không tìm thấy ffmpeg. Vui lòng cài đặt: brew install ffmpeg".to_string()
    })?;
    
    let output_path = PathBuf::from(&output_folder);
    if !output_path.exists() || !output_path.is_dir() {
        return Err("Thư mục output không hợp lệ".to_string());
    }
    
    let output_file = output_path.join(&output_filename);
    let output_file_str = output_file.to_string_lossy().to_string();
    
    // Mix audio: video audio (30%) + MP3 (100%)
    let filter_complex = "[0:a]volume=0.3[vid_audio];[vid_audio][1:a]amix=inputs=2:duration=longest:dropout_transition=3[finala]";
    
    let mut cmd = tokio::process::Command::new(&ffmpeg_path);
    
    // Input 1: Merged video
    cmd.arg("-i")
        .arg(&merged_video_path);
    
    // Input 2: Merged MP3
    cmd.arg("-i")
        .arg(&merged_mp3_path);
    
    // Apply filter_complex để mix audio
    cmd.arg("-filter_complex")
        .arg(filter_complex);
    
    // Map outputs: video copy, audio mixed
    cmd.arg("-map")
        .arg("0:v")  // Video từ input 0 (merged video)
        .arg("-map")
        .arg("[finala]");  // Audio đã mix
    
    // Video codec: copy (không re-encode - nhanh)
    cmd.arg("-c:v")
        .arg("copy");
    
    // Audio codec
    cmd.arg("-c:a")
        .arg("aac")
        .arg("-b:a")
        .arg("192k");
    
    cmd.arg("-y")
        .arg(&output_file_str);
    
    // Spawn process
    let child = cmd.spawn()
        .map_err(|e| format!("Lỗi khi chạy ffmpeg: {}", e))?;
    
    // Generate process ID
    let process_id = format!("step3_mix_{}", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs());
    
    // Store process handle
    {
        let mut procs = processes.lock().unwrap();
        procs.insert(process_id.clone(), child);
    }
    
    Ok(process_id)
}

// Helper: Lấy file path từ step 1 process ID
#[tauri::command]
pub async fn get_step1_merged_mp3_path(process_id: String, output_folder: String) -> Result<String, String> {
    // Parse process ID: step1_mp3_{timestamp}_{output_folder}
    let parts: Vec<&str> = process_id.strip_prefix("step1_mp3_")
        .ok_or_else(|| "Invalid step1 process ID".to_string())?
        .splitn(2, '_')
        .collect();
    
    if parts.is_empty() {
        return Err("Invalid step1 process ID format".to_string());
    }
    
    let timestamp = parts[0];
    let output_path = PathBuf::from(&output_folder);
    let work_dir = output_path.join(format!("ytbflow_step1_{}", timestamp));
    let merged_mp3 = work_dir.join("merged_audio.mp3");
    
    if !merged_mp3.exists() {
        return Err("File MP3 chưa được tạo".to_string());
    }
    
    Ok(merged_mp3.to_string_lossy().to_string())
}

// Helper function: Cleanup các file trung gian sau khi concat xong
fn cleanup_step2_intermediate_files(work_dir: &PathBuf) -> Result<(), String> {
    let cleanup_file = work_dir.join("cleanup_files.txt");
    
    if !cleanup_file.exists() {
        // Không có cleanup file, không cần cleanup
        return Ok(());
    }
    
    // Đọc danh sách file cần xóa
    let cleanup_content = fs::read_to_string(&cleanup_file)
        .map_err(|e| format!("Lỗi khi đọc cleanup file: {}", e))?;
    
    // Xóa từng file
    for line in cleanup_content.lines() {
        let file_path = line.trim();
        if !file_path.is_empty() {
            let path = PathBuf::from(file_path);
            if path.exists() {
                if let Err(e) = fs::remove_file(&path) {
                    // Log lỗi nhưng không fail toàn bộ quá trình
                    eprintln!("Không thể xóa file {}: {}", file_path, e);
                }
            }
        }
    }
    
    // Xóa cleanup file
    if cleanup_file.exists() {
        let _ = fs::remove_file(&cleanup_file);
    }
    
    Ok(())
}

// Helper: Lấy file path từ step 2 process ID và cleanup các file trung gian
#[tauri::command]
pub async fn get_step2_merged_video_path(process_id: String, output_folder: String) -> Result<String, String> {
    // Parse process ID: step2_video_{timestamp}_{output_folder}
    let parts: Vec<&str> = process_id.strip_prefix("step2_video_")
        .ok_or_else(|| "Invalid step2 process ID".to_string())?
        .splitn(2, '_')
        .collect();
    
    if parts.is_empty() {
        return Err("Invalid step2 process ID format".to_string());
    }
    
    let timestamp = parts[0];
    let output_path = PathBuf::from(&output_folder);
    let work_dir = output_path.join(format!("ytbflow_step2_{}", timestamp));
    let merged_video = work_dir.join("merged_video.mp4");
    
    if !merged_video.exists() {
        return Err("File video chưa được tạo".to_string());
    }
    
    // Cleanup các file trung gian sau khi đã có merged video
    cleanup_step2_intermediate_files(&work_dir)
        .map_err(|e| format!("Cảnh báo khi cleanup: {}", e))?;
    
    Ok(merged_video.to_string_lossy().to_string())
}

// Wrapper command: Chỉ spawn step 1, frontend sẽ điều khiển các bước tiếp theo
#[tauri::command]
pub async fn create_video_with_audio_mix(
    audio_files: Vec<String>,
    _video_files: Vec<String>,  // Không dùng ở đây, frontend sẽ gọi step 2 riêng
    output_folder: String,
    _output_filename: String,
    processes: tauri::State<'_, ProcessStore>,
) -> Result<String, String> {
    // Step 1: Merge MP3 - spawn và return process ID
    // Frontend sẽ wait và gọi step 2 sau
    let step1_id = create_merged_mp3_step1(audio_files, output_folder, processes).await?;
    Ok(step1_id)
}


