use crate::process::ProcessStore;
use crate::video::find_ffmpeg;
use crate::video_concat;
use std::path::PathBuf;
use std::fs;
use std::sync::Arc;
use tokio::sync::Semaphore;
use std::collections::HashMap;
use std::sync::Mutex;

/// Trait để định nghĩa cách build filter cho từng image effect
trait ImageEffect {
    /// Build filter string cho effect này
    fn build_filter(&self, params: &ImageEffectParams) -> String;
}

/// Parameters để build filter cho image effect
struct ImageEffectParams {
    width: i32,
    height: i32,
    total_frames: i32,
    _index: usize,  // Reserved for future use
    video_quality: String,
}

/// Tính toán scale factor dựa trên video quality
fn get_scale_factor(quality: &str) -> i32 {
    match quality {
        "hd" => 4000,      // 720p
        "fullhd" => 4000,  // 1080p
        "2K" => 8000,      // 2K
        "4K" => 8000,      // 4K
        _ => 4000,         // Default
    }
}

/// Base scale filter: scale và pad để fit vào resolution
fn build_base_scale(width: i32, height: i32) -> String {
    format!("scale={}:{}:force_original_aspect_ratio=decrease,pad={}:{}:(ow-iw)/2:(oh-ih)/2", 
        width, height, width, height)
}

// ============================================================================
// Image Effect Implementations
// ============================================================================

/// Không có hiệu ứng
struct NoneEffect;

impl ImageEffect for NoneEffect {
    fn build_filter(&self, params: &ImageEffectParams) -> String {
        let base_scale = build_base_scale(params.width, params.height);
        format!("{},fps=25", base_scale)
    }
}

/// Zoom In effect
struct ZoomInEffect;

impl ImageEffect for ZoomInEffect {
    fn build_filter(&self, params: &ImageEffectParams) -> String {
        let scale_factor = get_scale_factor(&params.video_quality);
        let base_scale = format!("scale={}:-1", scale_factor);
        format!("{},zoompan=z='min(zoom+0.0033,1.5)':d={}:s={}x{}:x='iw/2-(iw/zoom/2)':y='ih/2-(ih/zoom/2)',fps=25",
            base_scale, params.total_frames, params.width, params.height)
    }
}

/// Zoom Out effect
struct ZoomOutEffect;

impl ImageEffect for ZoomOutEffect {
    fn build_filter(&self, params: &ImageEffectParams) -> String {
        let scale_factor = get_scale_factor(&params.video_quality);
        let base_scale = format!("scale={}:-1", scale_factor);
        format!("{},zoompan=z='if(eq(on,1),1.5,zoom-0.0033)':d={}:s={}x{}:x='iw/2-(iw/zoom/2)':y='ih/2-(ih/zoom/2)',fps=25",
            base_scale, params.total_frames, params.width, params.height)
    }
}

/// Fade In effect
struct FadeInEffect;

impl ImageEffect for FadeInEffect {
    fn build_filter(&self, params: &ImageEffectParams) -> String {
        let base_scale = build_base_scale(params.width, params.height);
        format!("{},fade=t=in:st=0:d=1,fps=25", base_scale)
    }
}

/// Fade Out effect
struct FadeOutEffect;

impl ImageEffect for FadeOutEffect {
    fn build_filter(&self, params: &ImageEffectParams) -> String {
        let base_scale = build_base_scale(params.width, params.height);
        // Fade out trong 1 giây cuối
        let fade_start = params.total_frames as f32 / 25.0 - 1.0; // total_duration - 1 second
        format!("{},fade=t=out:st={}:d=1,fps=25", base_scale, fade_start)
    }
}

/// Pan effect
struct PanEffect;

impl ImageEffect for PanEffect {
    fn build_filter(&self, params: &ImageEffectParams) -> String {
        let base_scale = build_base_scale(params.width, params.height);
        format!("{},zoompan=z=1.2:x='on/{}*(iw-iw/zoom)':y='ih/2-(ih/zoom/2)':d={}:s={}x{},fps=25",
            base_scale, params.total_frames, params.total_frames, params.width, params.height)
    }
}

/// Factory function để tạo effect instance từ string
fn create_image_effect(effect_type: &str) -> Box<dyn ImageEffect> {
    match effect_type {
        "none" => Box::new(NoneEffect),
        "zoom-in" => Box::new(ZoomInEffect),
        "zoom-out" => Box::new(ZoomOutEffect),
        "fade-in" => Box::new(FadeInEffect),
        "fade-out" => Box::new(FadeOutEffect),
        "pan" => Box::new(PanEffect),
        _ => Box::new(NoneEffect), // Default to none
    }
}

/// Build filter string từ các parameters (helper function để tránh Send issue)
fn build_image_filter_string(
    image_effect_type: &str,
    width: i32,
    height: i32,
    total_frames: i32,
    index: usize,
    video_quality: &str,
) -> String {
    let effect = create_image_effect(image_effect_type);
    let params = ImageEffectParams {
        width,
        height,
        total_frames,
        _index: index,
        video_quality: video_quality.to_string(),
    };
    effect.build_filter(&params)
}

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
    _width: i32,  // Reserved for future use
    _height: i32,  // Reserved for future use
    _total_frames: i32,  // Reserved for future use
    filter: &str,  // Filter string đã được build sẵn
    preset: &str,
    crf: &str,
) -> Result<String, String> {
    let segment_file = work_dir.join(format!("segment_{:04}.mp4", index));
    let segment_path = segment_file.to_string_lossy().to_string();
    
    let mut cmd = tokio::process::Command::new(ffmpeg_path);
    cmd.arg("-loop")
        .arg("1")
        .arg("-framerate")
        .arg("25")  // Set framerate để tránh lỗi với PNG
        .arg("-i")
        .arg(image_file)
        .arg("-vf")
        .arg(filter)
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

/// Tạo video từ danh sách ảnh với các hiệu ứng và chất lượng tùy chọn
#[tauri::command]
pub async fn create_video_from_images(
    image_files: Vec<String>,
    image_duration: i32,
    video_quality: String,
    video_aspect_ratio: String,
    image_effect_type: String,
    video_effect_type: String,
    output_folder: String,
    _processes: tauri::State<'_, ProcessStore>,  // Reserved for future use
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
    
    // Xác định resolution dựa trên chất lượng và aspect ratio
    let (base_width, base_height) = match video_quality.as_str() {
        "hd" => (1280, 720),      // HD 720p
        "fullhd" => (1920, 1080), // Full HD 1080p
        "2K" => (2048, 1080),     // 2K
        "4K" => (3840, 2160),     // 4K
        _ => (1920, 1080),        // Default Full HD
    };
    
    // Áp dụng aspect ratio: 9:16 (dọc) hoặc 16:9 (ngang)
    let (width, height) = match video_aspect_ratio.as_str() {
        "9:16" => (base_height, base_width), // Swap cho video dọc (9:16)
        "16:9" => (base_width, base_height),  // Giữ nguyên cho video ngang (16:9)
        _ => (base_width, base_height),       // Default 16:9
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
        "fullhd" => 3,     // Full HD: vừa phải
        "2K" => 2,         // 2K: ít hơn một chút
        "4K" => 2,         // 4K: ít nhất để tránh quá tải
        _ => 4,            // Default Full HD
    };

    let semaphore = Arc::new(Semaphore::new(max_concurrent));
    
    // Sử dụng Arc để chia sẻ ffmpeg_path và work_dir
    let ffmpeg_path_arc = Arc::new(ffmpeg_path.clone());
    let work_dir_arc = Arc::new(work_dir.clone());
    let image_effect_type_arc = Arc::new(image_effect_type.clone());
    let video_quality_arc = Arc::new(video_quality.clone());
    
    // Tạo tasks để chạy song song với giới hạn concurrent
    let mut tasks = Vec::new();
    for (index, image_file) in image_files.iter().enumerate() {
        let ffmpeg_path_clone = ffmpeg_path_arc.clone();
        let work_dir_clone = work_dir_arc.clone();
        let image_effect_type_clone = image_effect_type_arc.clone();
        let video_quality_clone = video_quality_arc.clone();
        let image_file_clone = image_file.clone();
        let permit = semaphore.clone();
        
        // Build filter riêng cho từng segment (trước khi vào async block để tránh Send issue)
        // Mỗi segment có thể có index khác nhau nên cần build riêng
        let segment_filter = build_image_filter_string(
            &image_effect_type_clone,
            width,
            height,
            total_frames,
            index,
            &video_quality_clone,
        );
        
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
                &segment_filter,
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
    
    if video_segments.is_empty() {
        return Err("Không có segment nào được tạo".to_string());
    }
    
    // Concat các video segments với transitions
    // Nếu chỉ có 1 segment thì không cần concat
    let final_video_path = if video_segments.len() == 1 {
        // Chỉ có 1 segment, không cần concat
        video_segments[0].clone()
    } else {
        // Concat các segments với transitions
        let output_filename = format!("final_video_{}.mp4", timestamp);
        let final_video = work_dir.join(&output_filename);
        let final_video_path_str = final_video.to_string_lossy().to_string();
        
        // Transition duration mặc định là 1 giây
        let transition_duration = 1.0;
        
        // Gọi hàm concat helper từ video_concat module
        // force_scale = false vì các segments đã được tạo với cùng resolution rồi
        video_concat::concat_video_segments_with_transitions_helper(
            video_segments.clone(),
            &video_effect_type,
            &video_quality,
            &video_aspect_ratio,
            image_duration,
            transition_duration,
            &final_video_path_str,
            false, // Không scale vì segments đã cùng resolution
        ).await?;
        
        // Lưu output file path và folder vào HashMap
        let process_id = format!("images_video_{}", timestamp);
        {
            let mut files = IMAGE_VIDEO_OUTPUT_FILES.lock().unwrap();
            files.insert(process_id.clone(), final_video_path_str.clone());
        }
        {
            let mut folders = IMAGE_VIDEO_OUTPUT_FOLDERS.lock().unwrap();
            folders.insert(process_id.clone(), output_folder.clone());
        }
        
        final_video_path_str
    };
    
    Ok(format!("Video đã được tạo thành công! {}", final_video_path))
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

