use crate::process::ProcessStore;
use std::path::PathBuf;
use std::fs;
use std::io::Write;
use std::sync::Arc;
use tokio::sync::Semaphore;
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

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
    audio_files: Vec<String>,
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
        let final_video = output_path.join(&output_filename); // Lưu trực tiếp vào thư mục user chọn
        let final_video_path_str = final_video.to_string_lossy().to_string();
        
        // Transition duration mặc định là 1 giây
        let transition_duration = 1.0;
        
        // Concat các segments với transitions
        // force_scale = false vì các segments đã được tạo với cùng resolution rồi
        concat_video_segments_with_transitions_helper(
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
        
        // Xóa thư mục work_dir chứa các video segments sau khi hoàn thành
        let _ = fs::remove_dir_all(&work_dir);
        
        final_video_path_str
    };
    
    // Xử lý audio nếu có (tùy chọn không bắt buộc)
    let final_output_path = if !audio_files.is_empty() {
        
        // Tạo tên file cuối cùng với audio
        let final_with_audio_filename = format!("final_video_with_audio_{}.mp4", timestamp);
        let final_with_audio_path = output_path.join(&final_with_audio_filename);
        let final_with_audio_path_str = final_with_audio_path.to_string_lossy().to_string();
        
        // Lưu số lượng audio files trước khi move
        let audio_files_count = audio_files.len();
        
        // Bước 1: Merge audio files nếu có nhiều hơn 1 file
        let merged_audio_path = if audio_files_count > 1 {
            let merged_audio_filename = format!("merged_audio_{}.mp3", timestamp);
            let merged_audio_file = output_path.join(&merged_audio_filename);
            let merged_audio_path_str = merged_audio_file.to_string_lossy().to_string();
            
            merge_audio_files(audio_files, &merged_audio_path_str).await
                .map_err(|e| format!("Lỗi khi merge audio: {}", e))?;
            
            merged_audio_path_str
        } else {
            // Chỉ có 1 file audio, sử dụng trực tiếp
            let single_audio_path = audio_files[0].clone();
            // Drop audio_files để tránh warning về unused value
            drop(audio_files);
            single_audio_path
        };
        
        // Bước 2: Merge video với audio (video sẽ loop để match audio duration)
        
        merge_video_with_audio(
            &final_video_path,
            &merged_audio_path,
            &final_with_audio_path_str,
        ).await.map_err(|e| format!("Lỗi khi merge video với audio: {}", e))?;
        
        // Xóa file audio tạm nếu đã merge nhiều file
        if audio_files_count > 1 {
            let _ = fs::remove_file(&merged_audio_path);
        }
        
        // Xóa file video gốc (không có audio)
        let _ = fs::remove_file(&final_video_path);
        
        final_with_audio_path_str
    } else {
        final_video_path
    };
    
    Ok(format!("Video đã được tạo thành công! {}", final_output_path))
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
    
    // Lấy và remove process từ ProcessStore để giải phóng memory
    let child_opt = {
        let mut procs = processes.lock().unwrap();
        procs.remove(&process_id)
    };
    
    // Kill process đang chạy
    if let Some(mut child) = child_opt {
        if let Err(_e) = child.kill().await {
            // Ignore kill errors
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
    
    Ok("Đã dừng quá trình tạo video".to_string())
}

// ============================================================================
// Video Concat Helper Functions
// ============================================================================

/**
 * Tạo file concat list cho ffmpeg concat demuxer
 */
fn create_concat_list_file(files: &[String], work_dir: &PathBuf) -> Result<PathBuf, String> {
    let concat_file = work_dir.join(format!("concat_list_{}.txt", Uuid::new_v4()));
    let mut file = fs::File::create(&concat_file)
        .map_err(|e| format!("Lỗi khi tạo file concat list: {}", e))?;
    
    for file_path in files {
        // Escape single quotes và format cho concat demuxer
        // Format: file 'path/to/file.mp4'
        let escaped_path = file_path.replace('\'', "'\\''");
        writeln!(file, "file '{}'", escaped_path)
            .map_err(|e| format!("Lỗi khi ghi file concat list: {}", e))?;
    }
    
    Ok(concat_file)
}

/**
 * Concat video segments không có hiệu ứng bằng concat demuxer (nhanh hơn nhiều)
 */
async fn concat_without_effects_fast(
    segment_files: Vec<String>,
    output_path: &str,
) -> Result<(), String> {
    if segment_files.is_empty() {
        return Err("Cần ít nhất một video segment".to_string());
    }
    
    if segment_files.len() == 1 {
        // Chỉ có 1 segment, copy trực tiếp
        fs::copy(&segment_files[0], output_path)
            .map_err(|e| format!("Lỗi khi copy video: {}", e))?;
        return Ok(());
    }
    
    let ffmpeg_path = find_ffmpeg().ok_or_else(|| {
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
    })?;
    
    // Tạo file concat list
    let output_path_buf = PathBuf::from(output_path);
    let work_dir = output_path_buf.parent()
        .ok_or_else(|| "Không thể xác định thư mục output".to_string())?;
    
    let concat_list_file = create_concat_list_file(&segment_files, &work_dir.to_path_buf())?;
    
    // Build ffmpeg command với concat demuxer
    let mut cmd = tokio::process::Command::new(&ffmpeg_path);
    cmd.arg("-f")
        .arg("concat")
        .arg("-safe")
        .arg("0") // Cho phép absolute paths
        .arg("-i")
        .arg(concat_list_file.to_string_lossy().as_ref())
        .arg("-c:v")
        .arg("copy") // Copy video stream - không re-encode, rất nhanh
        .arg("-c:a")
        .arg("copy") // Copy audio stream nếu có
        .arg("-y")
        .arg(output_path);
    
    // Chạy và đợi process hoàn thành
    let output = cmd.output().await
        .map_err(|e| format!("Lỗi khi chạy ffmpeg concat: {}", e))?;
    
    // Cleanup file concat list
    let _ = fs::remove_file(&concat_list_file);
    
    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Lỗi khi concat video (không hiệu ứng): {}", error_msg));
    }
    
    Ok(())
}

/**
 * Ghép 2 video với transition
 */
async fn merge_two_videos_with_transition(
    video1_path: &str,
    video2_path: &str,
    output_path: &str,
    transition_type: &str,
    transition_duration: f64,
    video1_duration: f64,
    video2_duration: f64,
) -> Result<(), String> {
    let ffmpeg_path = find_ffmpeg().ok_or_else(|| {
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
    })?;
    
    // Build filter_complex cho 2 video với transition
    // Offset = duration của video1 - transition_duration
    let offset = video1_duration - transition_duration;
    
    let filter_complex = format!(
        "[0:v][1:v]xfade=transition={}:duration={}:offset={}[video_out]",
        transition_type, transition_duration, offset
    );
    
    // Tính tổng duration: video1 + video2 - transition (vì có overlap)
    let total_duration = video1_duration + video2_duration - transition_duration;
    
    // Build ffmpeg command
    let mut cmd = tokio::process::Command::new(&ffmpeg_path);
    cmd.arg("-i")
        .arg(video1_path)
        .arg("-i")
        .arg(video2_path)
        .arg("-filter_complex")
        .arg(&filter_complex)
        .arg("-map")
        .arg("[video_out]")
        .arg("-t")
        .arg(format!("{:.2}", total_duration))
        .arg("-c:v")
        .arg("libx264")
        .arg("-preset")
        .arg("medium")
        .arg("-crf")
        .arg("20")
        .arg("-pix_fmt")
        .arg("yuv420p")
        .arg("-g")
        .arg("50")
        .arg("-bf")
        .arg("2")
        .arg("-refs")
        .arg("4")
        .arg("-y")
        .arg(output_path);
    
    // Chạy và đợi process hoàn thành
    let output = cmd.output().await
        .map_err(|e| format!("Lỗi khi ghép 2 video: {}", e))?;
    
    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Lỗi khi ghép 2 video với transition: {}", error_msg));
    }
    
    Ok(())
}

/**
 * Concat video segments với transitions sử dụng Pipeline approach
 */
async fn concat_with_pipeline_approach(
    segment_files: Vec<String>,
    video_effect_type: &str,
    segment_duration: i32,
    transition_duration: f64,
    output_path: &str,
) -> Result<(), String> {
    if segment_files.is_empty() {
        return Err("Cần ít nhất một video segment".to_string());
    }
    
    if segment_files.len() == 1 {
        // Chỉ có 1 segment, copy trực tiếp
        fs::copy(&segment_files[0], output_path)
            .map_err(|e| format!("Lỗi khi copy video: {}", e))?;
        return Ok(());
    }
    
    // Tạo thư mục tạm để lưu các file trung gian
    let output_path_buf = PathBuf::from(output_path);
    let work_dir = output_path_buf.parent()
        .ok_or_else(|| "Không thể xác định thư mục output".to_string())?;
    
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let pipeline_work_dir = work_dir.join(format!("pipeline_temp_{}", timestamp));
    fs::create_dir_all(&pipeline_work_dir)
        .map_err(|e| format!("Lỗi khi tạo pipeline work directory: {}", e))?;
    
    // Bước 1: Ghép video đầu tiên với video thứ 2
    let current_output = pipeline_work_dir.join(format!("pipeline_000.mp4"));
    let mut current_output_str = current_output.to_string_lossy().to_string();
    
    // Tính duration hiện tại của output (bắt đầu với 2 video)
    let mut current_duration = segment_duration as f64 * 2.0 - transition_duration;
    
    merge_two_videos_with_transition(
        &segment_files[0],
        &segment_files[1],
        &current_output_str,
        video_effect_type,
        transition_duration,
        segment_duration as f64,
        segment_duration as f64,
    ).await.map_err(|e| format!("Lỗi khi ghép video 1-2: {}", e))?;
    
    // Bước 2: Ghép output hiện tại với các video tiếp theo
    for (idx, next_video) in segment_files.iter().enumerate().skip(2) {
        let next_output = pipeline_work_dir.join(format!("pipeline_{:03}.mp4", idx));
        let next_output_str = next_output.to_string_lossy().to_string();
        
        merge_two_videos_with_transition(
            &current_output_str,
            next_video,
            &next_output_str,
            video_effect_type,
            transition_duration,
            current_duration,
            segment_duration as f64,
        ).await.map_err(|e| format!("Lỗi khi ghép video {}: {}", idx + 1, e))?;
        
        // Cập nhật duration cho output mới
        current_duration = current_duration + segment_duration as f64 - transition_duration;
        
        // Xóa file trung gian trước đó
        let _ = fs::remove_file(&current_output_str);
        
        // Cập nhật current_output cho lần lặp tiếp theo
        current_output_str = next_output_str;
    }
    
    // Bước 3: Copy file cuối cùng vào output_path
    fs::copy(&current_output_str, output_path)
        .map_err(|e| format!("Lỗi khi copy file cuối cùng: {}", e))?;
    
    // Bước 4: Xóa file trung gian cuối cùng
    let _ = fs::remove_file(&current_output_str);
    
    // Bước 5: Xóa thư mục pipeline tạm (bao gồm tất cả file trung gian)
    let _ = fs::remove_dir_all(&pipeline_work_dir);

    Ok(())
}

/**
 * Helper function để concat video segments với transitions
 */
async fn concat_video_segments_with_transitions_helper(
    segment_files: Vec<String>,
    video_effect_type: &str,
    _video_quality: &str,
    _video_aspect_ratio: &str,
    segment_duration: i32,
    transition_duration: f64,
    output_path: &str,
    _force_scale: bool,
) -> Result<(), String> {
    if segment_files.is_empty() {
        return Err("Cần ít nhất một video segment".to_string());
    }
    
    if segment_files.len() == 1 {
        // Chỉ có 1 segment, không cần concat
        return Ok(());
    }
    
    // Nếu không có hiệu ứng, dùng concat demuxer với file list để nhanh hơn nhiều
    if video_effect_type == "none" {
        return concat_without_effects_fast(segment_files, output_path).await;
    }
    
    // Có hiệu ứng: dùng Pipeline approach
    return concat_with_pipeline_approach(
        segment_files,
        video_effect_type,
        segment_duration,
        transition_duration,
        output_path,
    ).await;
}

// ============================================================================
// Audio Processing Functions
// ============================================================================
/**
 * Merge nhiều file audio thành một file duy nhất
 */
async fn merge_audio_files(
    audio_files: Vec<String>,
    output_audio_path: &str,
) -> Result<(), String> {
    if audio_files.is_empty() {
        return Err("Cần ít nhất một file audio".to_string());
    }
    
    if audio_files.len() == 1 {
        // Chỉ có 1 file, copy trực tiếp
        fs::copy(&audio_files[0], output_audio_path)
            .map_err(|e| format!("Lỗi khi copy file audio: {}", e))?;
        return Ok(());
    }
    
    let ffmpeg_path = find_ffmpeg().ok_or_else(|| {
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
    })?;
    
    // Tạo file concat list cho audio
    let output_path_buf = PathBuf::from(output_audio_path);
    let work_dir = output_path_buf.parent()
        .ok_or_else(|| "Không thể xác định thư mục output".to_string())?;
    
    let audio_concat_list_file = create_concat_list_file(&audio_files, &work_dir.to_path_buf())?;
    
    // Build ffmpeg command để merge audio
    let mut cmd = tokio::process::Command::new(&ffmpeg_path);
    cmd.arg("-f")
        .arg("concat")
        .arg("-safe")
        .arg("0")
        .arg("-i")
        .arg(audio_concat_list_file.to_string_lossy().as_ref())
        .arg("-c:a")
        .arg("copy") // Copy audio stream - nhanh nhất
        .arg("-y")
        .arg(output_audio_path);
    
    // Chạy và đợi process hoàn thành
    let output = cmd.output().await
        .map_err(|e| format!("Lỗi khi merge audio: {}", e))?;
    
    // Cleanup file concat list
    let _ = fs::remove_file(&audio_concat_list_file);
    
    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Lỗi khi merge audio: {}", error_msg));
    }
    
    Ok(())
}

/**
 * Merge video với audio, đảm bảo video loop để match với audio duration
 */
async fn merge_video_with_audio(
    video_path: &str,
    audio_path: &str,
    output_path: &str,
) -> Result<(), String> {
    let ffmpeg_path = find_ffmpeg().ok_or_else(|| {
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
    })?;
    
    // Build ffmpeg command để merge video với audio
    // Sử dụng -stream_loop -1 để loop video cho đến hết audio
    // -shortest để đảm bảo output dừng khi audio kết thúc
    let mut cmd = tokio::process::Command::new(&ffmpeg_path);
    cmd.arg("-stream_loop")
        .arg("-1") // Loop video vô hạn
        .arg("-i")
        .arg(video_path)
        .arg("-i")
        .arg(audio_path)
        .arg("-c:v")
        .arg("copy") // Copy video stream - nhanh nhất
        .arg("-c:a")
        .arg("aac") // Encode audio thành AAC (video có thể không có audio stream)
        .arg("-b:a")
        .arg("256k") // Bitrate audio 256kbps cho chất lượng cao nhất
        .arg("-map")
        .arg("0:v:0") // Video từ input 0
        .arg("-map")
        .arg("1:a:0") // Audio từ input 1
        .arg("-shortest") // Dừng khi stream ngắn nhất (audio) kết thúc
        .arg("-avoid_negative_ts")
        .arg("make_zero") // Tránh lỗi timestamp âm
        .arg("-y")
        .arg(output_path);
    
    // Chạy và đợi process hoàn thành
    let output = cmd.output().await
        .map_err(|e| format!("Lỗi khi merge video với audio: {}", e))?;
    
    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Lỗi khi merge video với audio: {}", error_msg));
    }
    Ok(())
}

