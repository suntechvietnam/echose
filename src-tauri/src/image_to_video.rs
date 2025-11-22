use crate::process::ProcessStore;
use crate::video::find_ffmpeg;
use std::path::PathBuf;
use std::fs;
use std::io::Write;
use std::sync::Arc;
use tokio::sync::Semaphore;
use std::collections::HashMap;
use std::sync::Mutex;

// Lưu output_folder theo process_id để có thể cleanup khi dừng
lazy_static::lazy_static! {
    static ref IMAGE_VIDEO_OUTPUT_FOLDERS: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));
    static ref IMAGE_VIDEO_OUTPUT_FILES: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));
}

/// Tạo một segment video từ một ảnh (được gọi song song)
async fn create_single_segment(
    ffmpeg_path: &str,
    work_dir: &PathBuf,
    image_file: &str,
    index: usize,
    image_duration: i32,
    width: i32,
    height: i32,
    total_frames: i32,
    effect_type: &str,
    preset: &str,
    crf: &str,
) -> Result<String, String> {
    let segment_file = work_dir.join(format!("segment_{:04}.mp4", index));
    let segment_path = segment_file.to_string_lossy().to_string();
    
    // Build filter dựa trên effect type
    let filter = build_image_filter(effect_type, width, height, index, total_frames);
    
    let mut cmd = tokio::process::Command::new(ffmpeg_path);
    cmd.arg("-loop")
        .arg("1")
        .arg("-framerate")
        .arg("25")  // Set framerate để tránh lỗi với PNG
        .arg("-i")
        .arg(image_file)
        .arg("-vf")
        .arg(&filter)
        .arg("-t")
        .arg(format!("{}", image_duration))
        .arg("-r")
        .arg("25")  // Output framerate
        .arg("-vsync")
        .arg("cfr")  // Constant framerate để đảm bảo video không bị đứng
        .arg("-c:v")
        .arg("libx264")
        .arg("-preset")
        .arg(preset)  // Tối ưu preset (medium cho 4K)
        .arg("-crf")
        .arg(crf)  // Tối ưu CRF (20 cho 4K)
        .arg("-pix_fmt")
        .arg("yuv420p")
        .arg("-color_range")
        .arg("1")  // TV/limited range (16-235) để tránh warning deprecated pixel format
        .arg("-g")
        .arg("50")  // GOP size = 2x framerate để mượt hơn
        .arg("-bf")
        .arg("2")  // B-frames để nén tốt hơn và mượt hơn
        .arg("-refs")
        .arg("4")  // Reference frames để chất lượng tốt hơn
        .arg("-y")
        .arg(&segment_path);
    
    // Chạy và đợi process hoàn thành
    let output = cmd.output().await
        .map_err(|e| format!("Lỗi khi chạy ffmpeg cho segment {}: {}", index + 1, e))?;
    
    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Lỗi khi tạo segment {}: {}", index + 1, error_msg));
    }
    
    Ok(segment_path)
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

/// Concat các video segments với xfade filter để tạo crossfade mượt
fn concat_with_xfade(
    cmd: &mut tokio::process::Command,
    segments: &[String],
    segment_duration: i32,
    _output_file: &str,
) -> Result<(), String> {
    if segments.is_empty() {
        return Err("Không có segment nào để concat".to_string());
    }
    
    if segments.len() == 1 {
        // Nếu chỉ có 1 segment, không cần xfade
        cmd.arg("-i").arg(&segments[0]);
        cmd.arg("-c:v").arg("copy");
        return Ok(());
    }
    
    // Thêm tất cả segments làm input
    for segment in segments {
        cmd.arg("-i").arg(segment);
    }
    
    // Tính toán timing cho xfade
    // Crossfade duration: 1 giây
    let crossfade_duration = 1.0;
    let segment_duration_f = segment_duration as f64;
    
    // Tính tổng duration của output video
    // Với n segments, mỗi segment dài d giây, crossfade 1s:
    // Total = d + (n-1) * (d - 1) = n*d - (n-1)
    let total_duration = (segments.len() as f64 * segment_duration_f) - ((segments.len() - 1) as f64 * crossfade_duration);
    
    // Build filter_complex với xfade
    // Format: [0:v][1:v]xfade=transition=fade:duration=1:offset=4[v1];
    //         [v1][2:v]xfade=transition=fade:duration=1:offset=9[v2];...
    let mut filter_parts = Vec::new();
    let mut current_input = String::from("[0:v]");
    
    for i in 1..segments.len() {
        let next_input = format!("[{}:v]", i);
        let output_label = if i == segments.len() - 1 {
            String::from("[vout]")
        } else {
            format!("[v{}]", i)
        };
        
        // Offset = thời điểm trong output video mà transition bắt đầu
        // Với segment i, transition bắt đầu tại: i * (segment_duration - crossfade_duration)
        let offset = (i as f64) * (segment_duration_f - crossfade_duration);
        
        // Format: [input1][input2]xfade=transition=fade:duration=1:offset=4[output]
        let xfade_filter = format!("{}{}xfade=transition=fade:duration={}:offset={}:{}",
            current_input, next_input, crossfade_duration, offset, output_label);
        
        filter_parts.push(xfade_filter);
        current_input = output_label.clone();
    }
    
    let filter_complex = filter_parts.join(";");
    
    cmd.arg("-filter_complex")
        .arg(&filter_complex)
        .arg("-map")
        .arg("[vout]")
        .arg("-t")
        .arg(format!("{:.2}", total_duration))  // Set duration để đảm bảo video đủ dài
        .arg("-c:v")
        .arg("libx264")
        .arg("-preset")
        .arg("medium")  // Medium preset cho concat - cân bằng tốc độ và chất lượng
        .arg("-crf")
        .arg("23")  // CRF 23 cho concat output - chất lượng tốt
        .arg("-pix_fmt")
        .arg("yuv420p")
        .arg("-color_range")
        .arg("1")  // TV/limited range (16-235) để tránh warning deprecated pixel format
        .arg("-threads")
        .arg("0");  // Sử dụng tất cả CPU cores có sẵn
    
    Ok(())
}

/// Tạo video từ danh sách ảnh với các hiệu ứng và chất lượng tùy chọn
#[tauri::command]
pub async fn create_video_from_images(
    image_files: Vec<String>,
    image_duration: i32,
    video_quality: String,
    effect_type: String,
    output_folder: String,
    processes: tauri::State<'_, ProcessStore>,
) -> Result<String, String> {
    if image_files.is_empty() {
        return Err("Cần ít nhất một ảnh".to_string());
    }
    
    if image_duration < 5 {
        return Err("Khoảng cách giữa các ảnh phải tối thiểu 5 giây".to_string());
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
    
    // Lưu vào output folder
    let work_dir = output_path.join(format!("ytbflow_images_{}", timestamp));
    fs::create_dir_all(&work_dir)
        .map_err(|e| format!("Lỗi khi tạo work directory: {}", e))?;
    
    // Xác định resolution dựa trên chất lượng
    let (width, height) = match video_quality.as_str() {
        "hd" => (1280, 720),      // HD 720p
        "fullhd" => (1920, 1080), // Full HD 1080p
        "2K" => (2048, 1080),     // 2K
        "4K" => (3840, 2160),     // 4K
        _ => (1920, 1080),        // Default Full HD
    };
    
    // Tính số frame dựa trên duration (25 fps)
    let total_frames = image_duration * 25;
    
    // Tối ưu preset và CRF dựa trên chất lượng video
    let (preset, crf) = match video_quality.as_str() {
        "hd" => ("medium", "23"),        // HD: CRF 23 cho chất lượng tốt
        "fullhd" => ("medium", "22"),    // Full HD: CRF 22 cho chất lượng tốt hơn
        "2K" => ("medium", "20"),        // 2K: CRF 20 cho chất lượng cao
        "4K" => ("medium", "20"),        // 4K: CRF 20 để cân bằng tốc độ và chất lượng
        _ => ("medium", "22"),           // Default Full HD
    };
    
    // Tạo video từng ảnh với hiệu ứng - PARALLEL PROCESSING
    // Giới hạn số lượng concurrent tasks để tránh quá tải hệ thống
    let max_concurrent = match video_quality.as_str() {
        "hd" => 4,        // HD: có thể nhiều concurrent hơn
        "fullhd" => 4,     // Full HD: vừa phải
        "2K" => 2,         // 2K: ít hơn một chút
        "4K" => 2,         // 4K: ít nhất để tránh quá tải
        _ => 4,            // Default Full HD
    };

    let semaphore = Arc::new(Semaphore::new(max_concurrent));
    
    // Sử dụng Arc để chia sẻ ffmpeg_path và work_dir
    let ffmpeg_path_arc = Arc::new(ffmpeg_path.clone());
    let work_dir_arc = Arc::new(work_dir.clone());
    let effect_type_arc = Arc::new(effect_type.clone());
    
    // Tạo tasks để chạy song song với giới hạn concurrent
    let mut tasks = Vec::new();
    for (index, image_file) in image_files.iter().enumerate() {
        let ffmpeg_path_clone = ffmpeg_path_arc.clone();
        let work_dir_clone = work_dir_arc.clone();
        let effect_type_clone = effect_type_arc.clone();
        let image_file_clone = image_file.clone();
        let permit = semaphore.clone();
        
        let task = tokio::spawn(async move {
            // Acquire permit để giới hạn số lượng concurrent tasks
            let _permit = permit.acquire().await.map_err(|e| {
                format!("Lỗi khi acquire semaphore: {}", e)
            })?;
            
            create_single_segment(
                &ffmpeg_path_clone,
                &work_dir_clone,
                &image_file_clone,
                index,
                image_duration,
                width,
                height,
                total_frames,
                &effect_type_clone,
                preset,
                crf,
            ).await
        });
        
        tasks.push((index, task));
    }
    
    // Đợi tất cả tasks hoàn thành và thu thập kết quả
    let mut video_segments = vec![String::new(); image_files.len()];
    for (index, task) in tasks {
        let segment_path = task.await
            .map_err(|e| format!("Lỗi khi chạy task cho segment {}: {}", index + 1, e))?
            .map_err(|e| format!("Lỗi khi tạo segment {}: {}", index + 1, e))?;
        video_segments[index] = segment_path;
    }
    
    // Concat tất cả segments thành video cuối cùng
    // Nếu effect là fade-only hoặc slide-fade, sử dụng xfade để tạo crossfade mượt
    let output_file = output_path.join(format!("video_from_images_{}.mp4", timestamp));
    let output_file_str = output_file.to_string_lossy().to_string();
    
    let mut cmd = tokio::process::Command::new(&ffmpeg_path);
    
    if effect_type == "fade-only" || effect_type == "slide-fade" {
        // Sử dụng xfade filter để tạo crossfade mượt giữa các segment
        concat_with_xfade(&mut cmd, &video_segments, image_duration, &output_file_str)?;
    } else {
        // Sử dụng concat demuxer cho các hiệu ứng khác
        let concat_file = create_concat_file(&video_segments, &work_dir)?;
        cmd.arg("-f")
            .arg("concat")
            .arg("-safe")
            .arg("0")
            .arg("-i")
            .arg(concat_file.to_string_lossy().to_string())
            .arg("-c:v")
            .arg("copy");
    }
    
    cmd.arg("-y")
        .arg(&output_file_str);
    
    // Spawn process cho concat
    let child = cmd.spawn()
        .map_err(|e| format!("Lỗi khi chạy ffmpeg concat: {}", e))?;
    
    // Generate process ID
    let process_id = format!("images_video_{}", timestamp);
    
    // Store process handle
    {
        let mut procs = processes.lock().unwrap();
        procs.insert(process_id.clone(), child);
    }
    
    // Lưu output_folder và output_file để có thể cleanup khi dừng và trả về message
    {
        let mut folders = IMAGE_VIDEO_OUTPUT_FOLDERS.lock().unwrap();
        folders.insert(process_id.clone(), output_folder.clone());
    }
    {
        let mut files = IMAGE_VIDEO_OUTPUT_FILES.lock().unwrap();
        files.insert(process_id.clone(), output_file_str.clone());
    }
    
    // Lưu thông tin để cleanup sau này
    // Format: output_folder|work_dir_path
    let cleanup_file = work_dir.join("cleanup_info.txt");
    let mut cleanup_file_handle = fs::File::create(&cleanup_file)
        .map_err(|e| format!("Lỗi khi tạo cleanup file: {}", e))?;
    writeln!(cleanup_file_handle, "{}|{}", 
        output_folder,
        work_dir.to_string_lossy())
        .map_err(|e| format!("Lỗi khi ghi cleanup file: {}", e))?;
    
    Ok(process_id)
}

/// Build filter string dựa trên effect type
fn build_image_filter(effect_type: &str, width: i32, height: i32, index: usize, total_frames: i32) -> String {
    let base_scale = format!("scale={}:{}:force_original_aspect_ratio=decrease,pad={}:{}:(ow-iw)/2:(oh-ih)/2", 
        width, height, width, height);
    
    match effect_type {
        "zoom-random" => apply_zoom_random_effect(&base_scale, width, height, index, total_frames),
        "zoom-ken-burns" => apply_ken_burns_effect(&base_scale, width, height, index, total_frames),
        "fade-only" => apply_fade_only_effect(&base_scale, width, height, index, total_frames),
        "slide-fade" => apply_slide_fade_effect(&base_scale),
        "none" => apply_no_effect(&base_scale),
        _ => apply_default_effect(&base_scale),
    }
}

/// Hiệu ứng zoom nhẹ random in/out
fn apply_zoom_random_effect(base_scale: &str, width: i32, height: i32, index: usize, total_frames: i32) -> String {
    // Zoom nhẹ random in/out - giảm tốc độ zoom để tránh rung
    // Zoom từ 1.0 đến 1.05 trong suốt duration
    let zoom_max = if index % 2 == 0 { "1.05" } else { "1.03" };
    // Giảm zoom rate để mượt hơn
    format!("{},zoompan=z='min(zoom+0.0003,{})':d={}:x='iw/2-(iw/zoom/2)':y='ih/2-(ih/zoom/2)':s={}x{},fps=25", 
        base_scale, zoom_max, total_frames, width, height)
}

/// Hiệu ứng Ken Burns (zoom + pan)
fn apply_ken_burns_effect(base_scale: &str, width: i32, height: i32, index: usize, total_frames: i32) -> String {
    // Ken Burns effect: zoom từ 1.0 đến 1.15 và pan mượt hơn
    // Giảm tốc độ zoom và pan để tránh rung
    let pan_x = if index % 2 == 0 { "iw*0.15" } else { "iw*0.25" };
    // Giảm zoom rate để mượt hơn
    format!("{},zoompan=z='min(zoom+0.0004,1.15)':d={}:x='{}':y='ih/2-(ih/zoom/2)':s={}x{},fps=25", 
        base_scale, total_frames, pan_x, width, height)
}

/// Hiệu ứng fade đơn giản với zoom nhẹ nhàng
/// Không fade out ở segment vì sẽ dùng xfade khi concat
/// Zoom từ từ từ 1.0 đến 1.06 để tạo chuyển động nhẹ nhàng
fn apply_fade_only_effect(base_scale: &str, width: i32, height: i32, _index: usize, total_frames: i32) -> String {
    // Chỉ zoom nhẹ nhàng từ 1.0 đến 1.06 trong suốt duration
    // Không fade trong segment vì xfade sẽ xử lý transition khi concat
    // Thứ tự: scale -> zoompan -> fps
    // Zoom rate được tính để đạt 1.06 sau total_frames frames
    // Với image_duration = 5s (125 frames): zoom_rate = 0.06 / 125 ≈ 0.00048
    // Để zoom nhẹ nhàng hơn, dùng zoom_rate nhỏ hơn một chút: 0.0004
    format!("{},zoompan=z='min(zoom+0.0004,1.06)':d={}:x='iw/2-(iw/zoom/2)':y='ih/2-(ih/zoom/2)':s={}x{},fps=25", 
        base_scale, total_frames, width, height)
}

/// Hiệu ứng slide + fade
/// Không fade out ở segment vì sẽ dùng xfade khi concat
fn apply_slide_fade_effect(base_scale: &str) -> String {
    // Chỉ fade in 1s đầu, không fade out (sẽ dùng xfade khi concat)
    format!("{},fade=t=in:st=0:d=1,fps=25", base_scale)
}

/// Không có hiệu ứng
fn apply_no_effect(base_scale: &str) -> String {
    // Không có hiệu ứng nhưng vẫn cần fps để video không bị đứng
    format!("{},fps=25", base_scale)
}

/// Hiệu ứng mặc định
fn apply_default_effect(base_scale: &str) -> String {
    // Default: slide left + fade (simplified)
    format!("{},fade=t=in:st=0:d=1,fade=t=out:st=4:d=1,fps=25", base_scale)
}

/// Dừng quá trình tạo video từ ảnh và cleanup toàn bộ file đã tạo
#[tauri::command]
pub async fn stop_image_video_creation(
    process_id: String,
    processes: tauri::State<'_, ProcessStore>,
) -> Result<String, String> {
    // Kiểm tra process_id có đúng format không
    if !process_id.starts_with("images_video_") {
        return Err("Process ID không hợp lệ cho image video creation".to_string());
    }
    
    // Lấy output_folder từ HashMap
    let output_folder = {
        let folders = IMAGE_VIDEO_OUTPUT_FOLDERS.lock().unwrap();
        folders.get(&process_id).cloned()
    };
    
    // Lấy và remove process từ ProcessStore để giải phóng memory
    let child_opt = {
        let mut procs = processes.lock().unwrap();
        procs.remove(&process_id)
    };
    
    // Kill process đang chạy
    if let Some(mut child) = child_opt {
        // Kill the process
        if let Err(e) = child.kill().await {
            eprintln!("Cảnh báo: Không thể kill process: {}", e);
        }
        
        // Đợi process kết thúc để giải phóng tài nguyên
        let _ = child.wait().await;
    }
    
    // Remove output_folder và output_file từ HashMap
    {
        let mut folders = IMAGE_VIDEO_OUTPUT_FOLDERS.lock().unwrap();
        folders.remove(&process_id);
    }
    {
        let mut files = IMAGE_VIDEO_OUTPUT_FILES.lock().unwrap();
        files.remove(&process_id);
    }
    
    // Cleanup work directory và các file đã tạo
    cleanup_images_workdir(&process_id, output_folder.as_deref()).await?;
    
    Ok("Đã dừng quá trình tạo video và xóa toàn bộ file tạm".to_string())
}

/// Cleanup output_folder từ HashMap khi process hoàn thành
pub fn cleanup_output_folder_from_map(process_id: &str) {
    let mut folders = IMAGE_VIDEO_OUTPUT_FOLDERS.lock().unwrap();
    folders.remove(process_id);
}

/// Lấy output_file_path từ HashMap
pub fn get_output_file_path(process_id: &str) -> Option<String> {
    let files = IMAGE_VIDEO_OUTPUT_FILES.lock().unwrap();
    files.get(process_id).cloned()
}

/// Cleanup output_file từ HashMap khi process hoàn thành
pub fn cleanup_output_file_from_map(process_id: &str) {
    let mut files = IMAGE_VIDEO_OUTPUT_FILES.lock().unwrap();
    files.remove(process_id);
}

/// Cleanup work directory và các file tạm của process image video
async fn cleanup_images_workdir(process_id: &str, output_folder: Option<&str>) -> Result<(), String> {
    // Parse timestamp từ process_id: images_video_{timestamp}
    let timestamp = process_id.strip_prefix("images_video_")
        .ok_or_else(|| "Invalid process ID format".to_string())?;
    
    // Nếu có output_folder, tìm work_dir trực tiếp từ đó
    if let Some(output_folder_path) = output_folder {
        let output_path = PathBuf::from(output_folder_path);
        if output_path.exists() && output_path.is_dir() {
            let work_dir = output_path.join(format!("ytbflow_images_{}", timestamp));
            if work_dir.exists() && work_dir.is_dir() {
                // Xóa toàn bộ folder work_dir (bao gồm tất cả file và subfolder)
                if let Err(e) = fs::remove_dir_all(&work_dir) {
                    return Err(format!("Không thể xóa folder {}: {}", work_dir.display(), e));
                }
                return Ok(());
            }
        }
    }
    
    // Fallback: Tìm work_dir trong các thư mục phổ biến
    let possible_dirs = vec![
        std::env::var("HOME").ok().map(|h| PathBuf::from(h).join("Downloads")),
        std::env::var("HOME").ok().map(|h| PathBuf::from(h).join("Desktop")),
        std::env::var("HOME").ok().map(|h| PathBuf::from(h)),
    ];
    
    // Tìm work_dir trong các thư mục phổ biến
    for maybe_dir in possible_dirs {
        if let Some(base_dir) = maybe_dir {
            let work_dir = base_dir.join(format!("ytbflow_images_{}", timestamp));
            if work_dir.exists() && work_dir.is_dir() {
                // Đọc cleanup_info.txt để lấy work_dir_path chính xác
                let cleanup_info_file = work_dir.join("cleanup_info.txt");
                if cleanup_info_file.exists() {
                    if let Ok(content) = fs::read_to_string(&cleanup_info_file) {
                        let parts: Vec<&str> = content.trim().split('|').collect();
                        if parts.len() >= 2 {
                            // output_folder là phần đầu tiên, work_dir_path là phần thứ hai
                            let work_dir_path = parts[1];
                            let correct_work_dir = PathBuf::from(work_dir_path);
                            
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
                
                // Nếu không có cleanup_info.txt hoặc không đọc được, xóa toàn bộ folder hiện tại
                if let Err(e) = fs::remove_dir_all(&work_dir) {
                    return Err(format!("Không thể xóa folder {}: {}", work_dir.display(), e));
                } else {
                    return Ok(());
                }
            }
        }
    }
    
    // Nếu không tìm thấy work_dir, có thể đã bị xóa hoặc không tồn tại
    // Không coi đây là lỗi nghiêm trọng - chỉ log warning
    eprintln!("Cảnh báo: Không tìm thấy work_dir cho process_id: {}", process_id);
    Ok(())
}

