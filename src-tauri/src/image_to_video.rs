use crate::process::ProcessStore;
use crate::utils::find_ffmpeg_by_os::{find_ffmpeg_or_error, run_ffmpeg, get_best_encoder};
use crate::utils::merge_audio_to_video::{merge_audio_without_caption, merge_audio_with_caption, merge_video_all_in_one};
use std::path::PathBuf;
use std::fs;
use std::io::Write;
use std::sync::Arc;
use tokio::sync::Semaphore;
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

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

/// Tính toán scale factor dựa trên video quality ĐỂ ZOOM MƯỢT mà không quá nặng RAM
fn get_scale_factor(quality: &str) -> i32 {
    match quality {
        "hd" => 1800,      // 720p (1280x720) -> 1800px enough for zoom
        "fullhd" => 2500,  // 1080p (1920x1080) -> 2500px enough
        "2K" => 3500,      // 2K
        "4K" => 4500,      // 4K
        _ => 2500,         // Default
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

// Không có hiệu ứng
struct NoneEffect;

impl ImageEffect for NoneEffect {
    fn build_filter(&self, params: &ImageEffectParams) -> String {
        let base_scale = build_base_scale(params.width, params.height);
        format!("{},fps=25", base_scale)
    }
}

// Zoom In effect
struct ZoomInEffect;

impl ImageEffect for ZoomInEffect {
    fn build_filter(&self, params: &ImageEffectParams) -> String {
        let scale_factor = get_scale_factor(&params.video_quality);
        let base_scale = format!("scale={}:-1", scale_factor);
        format!("{},zoompan=z='min(zoom+0.0033,1.5)':d={}:s={}x{}:x='iw/2-(iw/zoom/2)':y='ih/2-(ih/zoom/2)',fps=25",
            base_scale, params.total_frames, params.width, params.height)
    }
}

// Zoom Out effect
struct ZoomOutEffect;

impl ImageEffect for ZoomOutEffect {
    fn build_filter(&self, params: &ImageEffectParams) -> String {
        let scale_factor = get_scale_factor(&params.video_quality);
        let base_scale = format!("scale={}:-1", scale_factor);
        format!("{},zoompan=z='if(eq(on,1),1.5,zoom-0.0033)':d={}:s={}x{}:x='iw/2-(iw/zoom/2)':y='ih/2-(ih/zoom/2)',fps=25",
            base_scale, params.total_frames, params.width, params.height)
    }
}

// Fade In effect
struct FadeInEffect;

impl ImageEffect for FadeInEffect {
    fn build_filter(&self, params: &ImageEffectParams) -> String {
        let base_scale = build_base_scale(params.width, params.height);
        format!("{},fade=t=in:st=0:d=1,fps=25", base_scale)
    }
}

// Fade Out effect
struct FadeOutEffect;

impl ImageEffect for FadeOutEffect {
    fn build_filter(&self, params: &ImageEffectParams) -> String {
        let base_scale = build_base_scale(params.width, params.height);
        // Fade out trong 1 giây cuối
        let fade_start = params.total_frames as f32 / 25.0 - 1.0; // total_duration - 1 second
        format!("{},fade=t=out:st={}:d=1,fps=25", base_scale, fade_start)
    }
}

// Pan effect
struct PanEffect;

impl ImageEffect for PanEffect {
    fn build_filter(&self, params: &ImageEffectParams) -> String {
        let base_scale = build_base_scale(params.width, params.height);
        format!("{},zoompan=z=1.2:x='on/{}*(iw-iw/zoom)':y='ih/2-(ih/zoom/2)':d={}:s={}x{},fps=25",
            base_scale, params.total_frames, params.total_frames, params.width, params.height)
    }
}

// Factory function để tạo effect instance từ string
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
    _width: i32,
    _height: i32,
    _total_frames: i32,
    filter: &str,
    preset: &str,
    crf: &str,
    process_id: &str,
    processes: &ProcessStore,
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
        .arg("cfr"); // Constant framerate để đảm bảo video không bị đứng
    
    let mut encoder = get_best_encoder();
    if encoder == "h264_videotoolbox" {
        encoder = "hevc_videotoolbox"; // Force HEVC for Images
    }
    
    cmd.arg("-c:v").arg(encoder);
    
    if encoder == "libx264" {
        cmd.arg("-preset").arg(preset).arg("-crf").arg(crf);
    } else if encoder == "hevc_videotoolbox" {
        // UPGRADE: Chuyển sang HEVC Quality-based nếu có thể (như video_to_video)
        // Tuy nhiên hàm create_single_segment đang nhận encoder từ get_best_encoder() 
        // vốn trả về h264_videotoolbox. Ta sẽ override ở đây để dùng HEVC cho xịn.
        
        // Remove previous -c:v arg if possible or just override
        // Rust Command doesn't support removing args easily. 
        // Instead, we trust the user has a Mac with HEVC.
        
        // Sửa lại logic: Thay vì check encoder string, ta check OS và ép dùng hevc_videotoolbox
        // Nhưng để an toàn, ta dùng logic tương tự video_to_video:
        
        // "h264_videotoolbox" detected -> Force HEVC
        // let hevc_encoder = "hevc_videotoolbox"; (Đã set ở trên)
        
        // Xóa arg -c:v cũ (Workaround: pop arg ko được, nên ta sẽ phải sửa từ đoạn gọi get_best_encoder)
        // TẠM THỜI: Để đơn giản, ta cứ dùng arg bitrate/quality cho encoder hiện tại, 
        // nhưng nếu là Mac, ta sẽ thêm tag và quality mode thay vì bitrate.
        
        // Cấu hình Quality-Based cho Mac (Nét căng, Màu chuẩn)
        cmd.arg("-q:v").arg("65") 
           .arg("-tag:v").arg("hvc1")
           .arg("-realtime").arg("0")
           .arg("-profile:v").arg("main")
           // FIX MÀU:
           .arg("-color_primaries").arg("1")
           .arg("-color_trc").arg("1")
           .arg("-colorspace").arg("1");
           
    } else if encoder.contains("nvenc") {
        cmd.arg("-cq").arg(crf).arg("-preset").arg("p4");
    }
    
    cmd.arg("-pix_fmt")
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
    
    // Chạy và spawn process
    let mut child = cmd.spawn()
        .map_err(|e| format!("Lỗi khi spawn ffmpeg cho segment {}: {}", index + 1, e))?;
    
    let child_arc = Arc::new(tokio::sync::Mutex::new(Some(child)));
    
    // Đăng ký process để có thể stop
    {
        let mut procs = processes.lock().unwrap();
        procs.entry(process_id.to_string()).or_insert_with(Vec::new).push(child_arc.clone());
    }
    
    // Đợi process hoàn thành
    let status = {
        let mut child_lock = child_arc.lock().await;
        if let Some(mut child) = child_lock.take() {
            child.wait().await.map_err(|e| format!("Lỗi khi đợi segment {}: {}", index + 1, e))?
        } else {
            return Err(format!("Segment {} đã bị dừng trước khi bắt đầu", index + 1));
        }
    };
    
    // Xong thì remove khỏi store (tùy chọn, stop_image_video_creation cũng dọn dẹp rồi)
    {
        let mut procs = processes.lock().unwrap();
        if let Some(vec) = procs.get_mut(process_id) {
            vec.retain(|p| !Arc::ptr_eq(p, &child_arc));
        }
    }
    
    if !status.success() {
        return Err(format!("Lỗi khi tạo segment {} (exit code {:?})", index + 1, status.code()));
    }
    
    Ok(segment_path)
}

/// Helper function to run ffmpeg and register it for cancellation
async fn run_ffmpeg_with_cancellation(
    mut cmd: tokio::process::Command,
    process_id: &str,
    processes: &tauri::State<'_, ProcessStore>,
) -> Result<std::process::Output, String> {
    let child = cmd.spawn().map_err(|e| format!("Lỗi khi spawn ffmpeg: {}", e))?;
    let child_arc = Arc::new(tokio::sync::Mutex::new(Some(child)));
    
    // Đăng ký process để có thể stop
    {
        let mut procs = processes.lock().unwrap();
        procs.entry(process_id.to_string()).or_insert_with(Vec::new).push(child_arc.clone());
    }
    
    // Đợi process hoàn thành
    let result = {
        let mut child_lock = child_arc.lock().await;
        if let Some(child) = child_lock.take() {
            child.wait_with_output().await.map_err(|e| e.to_string())
        } else {
            Err("Tiến trình đã bị dừng".to_string())
        }
    };
    
    // Sau khi xong, remove khỏi store (chỉ remove chính nó khỏi Vec)
    {
        let mut procs = processes.lock().unwrap();
        if let Some(vec) = procs.get_mut(process_id) {
            vec.retain(|p| !Arc::ptr_eq(p, &child_arc));
        }
    }
    
    result
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
    is_auto_caption: bool,
    logo_path: Option<String>,
    logo_position: Option<String>,
    logo_margin_top: i32,
    logo_margin_right: i32,
    logo_margin_bottom: i32,
    logo_margin_left: i32,
    subtitle_path: Option<String>,
    subtitle_margin_v: i32,
    subtitle_font_size: i32,
    subtitle_language: String,
    subtitle_font_name: String,
    process_id: String,
    processes: tauri::State<'_, ProcessStore>, 
) -> Result<String, String> {
    if image_files.is_empty() {
        return Err("Cần ít nhất một ảnh".to_string());
    }
    
    if image_duration < 5 {
        return Err("Khoảng cách giữa các ảnh phải tối thiểu 5 giây".to_string());
    }
    
    let ffmpeg_path = find_ffmpeg_or_error()?;
    
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
    let work_dir = output_path.join(format!("echose_images_{}", timestamp));
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
    // Tối ưu số lượng concurrent tasks dựa trên CPU và chất lượng
    let cpus = num_cpus::get();
    let max_concurrent = match video_quality.as_str() {
        "4K" | "2K" => (cpus / 4).max(1),
        "fullhd" => (cpus / 2).max(2),
        _ => (cpus - 1).max(2),
    };

    let semaphore = Arc::new(Semaphore::new(max_concurrent));
    
    // Sử dụng Arc để chia sẻ ffmpeg_path và work_dir
    let ffmpeg_path_arc = Arc::new(ffmpeg_path.clone());
    let work_dir_arc = Arc::new(work_dir.clone());
    let image_effect_type_arc = Arc::new(image_effect_type.clone());
    let video_quality_arc = Arc::new(video_quality.clone());
    
    // Tạo tasks để chạy song song với giới hạn concurrent
    let processes_store = processes.inner().clone();
    let process_id_clone_for_tasks = process_id.clone();
    let mut tasks = Vec::new();
    for (index, image_file) in image_files.iter().enumerate() {
        let ffmpeg_path_clone = ffmpeg_path_arc.clone();
        let work_dir_clone = work_dir_arc.clone();
        let image_effect_type_clone = image_effect_type_arc.clone();
        let video_quality_clone = video_quality_arc.clone();
        let image_file_clone = image_file.clone();
        let permit = semaphore.clone();
        let process_id_task = process_id_clone_for_tasks.clone();
        let processes_task = processes_store.clone();
        
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
                &process_id_task,
                &processes_task,
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
            &process_id,
            &processes,
        ).await?;
        
        // Lưu output file path và folder vào HashMap
        // let process_id = format!("images_video_{}", timestamp); // Dùng process_id từ frontend
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
    
    // Xử lý HẬU KỲ: Audio + Logo + Subtitle (Master Merge)
    let final_output_path = {
        // Tạo tên file cuối cùng
        let final_filename = format!("echose_final_{}.mp4", timestamp);
        let final_path = output_path.join(&final_filename);
        let final_path_str = final_path.to_string_lossy().to_string();
        
        // 1. Chuẩn bị Audio (Merge nếu nhiều file)
        let audio_to_use = if !audio_files.is_empty() {
             let audio_files_count = audio_files.len();
             if audio_files_count > 1 {
                let merged_audio_filename = format!("merged_audio_{}.mp3", timestamp);
                let merged_audio_file = output_path.join(&merged_audio_filename);
                let merged_audio_path_str = merged_audio_file.to_string_lossy().to_string();
                
                merge_audio_files(audio_files, &merged_audio_path_str, &process_id, &processes).await
                    .map_err(|e| format!("Lỗi khi merge audio: {}", e))?;
                
                Some(merged_audio_path_str)
            } else {
                Some(audio_files[0].clone())
            }
        } else {
            None
        };

        // 2. Gọi MASTER MERGE
        merge_video_all_in_one(
            &final_video_path,
            audio_to_use.as_deref(),
            logo_path.as_deref(),
            logo_position.as_deref(),
            logo_margin_top,
            logo_margin_right,
            logo_margin_bottom,
            logo_margin_left,
            subtitle_path.as_deref(),
            subtitle_margin_v,
            subtitle_font_size,
            Some(&subtitle_language),
            Some(&subtitle_font_name),
            is_auto_caption,
            &final_path_str,
        ).await.map_err(|e| format!("Lỗi khi xử lý hậu kỳ (Audio/Logo/Sub): {}", e))?;

        // 3. Cleanup
        // Xóa file video trung gian (không có logo/audio/sub)
        if final_path_str != final_video_path {
            let _ = fs::remove_file(&final_video_path);
        }
        // Xóa file audio tạm nếu có
        if let Some(ref path) = audio_to_use {
            if path.contains("merged_audio_") {
                let _ = fs::remove_file(path);
            }
        }

        final_path_str
    };
    
    Ok(format!("Video đã được tạo thành công! {}", final_output_path))
}

/// Dừng quá trình tạo video từ ảnh và cleanup toàn bộ file đã tạo
#[tauri::command]
pub async fn stop_image_video_creation(
    process_id: String,
    processes: tauri::State<'_, ProcessStore>,
) -> Result<String, String> {
    // Lấy và remove danh sách process từ ProcessStore
    let child_arcs = {
        let mut procs = processes.lock().unwrap();
        procs.remove(&process_id).unwrap_or_default()
    };
    
    // Kill tất cả process đang chạy cho ID này
    for child_arc in child_arcs {
        let mut child_lock = child_arc.lock().await;
        if let Some(mut child) = child_lock.take() {
            let _ = child.kill().await;
            let _ = child.wait().await;
        }
    }
    
    // Remove output_folder và output_file từ HashMap (nếu có dùng ở chỗ khác)
    {
        let mut folders = IMAGE_VIDEO_OUTPUT_FOLDERS.lock().unwrap();
        folders.remove(&process_id);
    }
    {
        let mut files = IMAGE_VIDEO_OUTPUT_FILES.lock().unwrap();
        files.remove(&process_id);
    }
    
    Ok("Đã dừng toàn bộ tiến trình tạo video".to_string())
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
    process_id: &str,
    processes: &tauri::State<'_, ProcessStore>,
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
    
    // Tạo file concat list
    let output_path_buf = PathBuf::from(output_path);
    let work_dir = output_path_buf.parent()
        .ok_or_else(|| "Không thể xác định thư mục output".to_string())?;
    
    let concat_list_file = create_concat_list_file(&segment_files, &work_dir.to_path_buf())?;
    
    // Build ffmpeg command với concat demuxer
    let mut cmd = run_ffmpeg()?;
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
    
    // Chạy và đợi với khả năng stop
    let output = run_ffmpeg_with_cancellation(cmd, process_id, processes).await?;
    
    // Cleanup file concat list
    let _ = fs::remove_file(&concat_list_file);
    
    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Lỗi khi concat video (không hiệu ứng): {}", error_msg));
    }
    
    Ok(())
}


async fn concat_video_segments_with_transitions_helper(
    segment_files: Vec<String>,
    video_effect_type: &str,
    video_quality: &str,
    _video_aspect_ratio: &str,
    segment_duration: i32,
    transition_duration: f64,
    output_path: &str,
    _force_scale: bool,
    process_id: &str,
    processes: &tauri::State<'_, ProcessStore>,
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
        return concat_without_effects_fast(segment_files, output_path, process_id, processes).await;
    }
    
    // Có hiệu ứng: sử dụng SINGLE-PASS TRANSITION (Tối ưu nhất cho ảnh)
    let mut filter_parts = Vec::new();
    let mut current_offset = 0.0;
    let mut last_output_label = "[v0]".to_string();
    
    // Init video đầu tiên
    filter_parts.push(format!("[0:v]null[v0]"));

    for (i, _) in segment_files.iter().enumerate().skip(1) {
        current_offset += segment_duration as f64 - transition_duration;
        let joined_label = format!("[vjoin{}]", i);
        
        filter_parts.push(format!(
            "{}[{}:v]xfade=transition={}:duration={}:offset={:.2}{}",
            last_output_label, i, video_effect_type, transition_duration, current_offset, joined_label
        ));
        
        last_output_label = joined_label;
    }

    let filter_complex = filter_parts.join("; ");
    let final_label = last_output_label;

    let mut cmd = run_ffmpeg()?;
    let mut encoder = get_best_encoder();
    if encoder == "h264_videotoolbox" {
        encoder = "hevc_videotoolbox"; // Force HEVC for Images
    }
    
    for segment in &segment_files {
        cmd.arg("-i").arg(segment);
    }
    
    cmd.arg("-filter_complex").arg(&filter_complex)
       .arg("-map").arg(final_label)
       .arg("-r").arg("30") // Đồng bộ framerate ngõ ra
       .arg("-c:v").arg(encoder);

    if encoder == "libx264" {
        cmd.arg("-crf").arg("20").arg("-preset").arg("medium");
    } else if encoder == "hevc_videotoolbox" {
        // UPGRADE: Ép dùng HEVC cho output cuối cùng
        
        // Cấu hình Quality-Based (Nét 100%) + Fix Màu
        cmd.arg("-q:v").arg("65")
           .arg("-tag:v").arg("hvc1")
           .arg("-realtime").arg("0")
           .arg("-profile:v").arg("main")
           .arg("-color_primaries").arg("1")
           .arg("-color_trc").arg("1")
           .arg("-colorspace").arg("1");
           
    } else if encoder.contains("nvenc") {
        cmd.arg("-cq").arg("20");
    }

    cmd.arg("-pix_fmt").arg("yuv420p")
       .arg("-y").arg(output_path);

    let output = run_ffmpeg_with_cancellation(cmd, process_id, processes).await?;
    if !output.status.success() {
        return Err(format!("FFmpeg image transition failing: {}", String::from_utf8_lossy(&output.stderr)));
    }
    Ok(())
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
    process_id: &str,
    processes: &tauri::State<'_, ProcessStore>,
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
    
    // Tạo file concat list cho audio
    let output_path_buf = PathBuf::from(output_audio_path);
    let work_dir = output_path_buf.parent()
        .ok_or_else(|| "Không thể xác định thư mục output".to_string())?;
    
    let audio_concat_list_file = create_concat_list_file(&audio_files, &work_dir.to_path_buf())?;
    
    // Build ffmpeg command để merge audio
    let mut cmd = run_ffmpeg()?;
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
    
    // Chạy với khả năng stop
    let output = run_ffmpeg_with_cancellation(cmd, process_id, processes).await?;
    
    // Cleanup file concat list
    let _ = fs::remove_file(&audio_concat_list_file);
    
    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Lỗi khi merge audio: {}", error_msg));
    }
    
    Ok(())
}

/**
 * Merge video với audio, với tùy chọn thêm caption
 * has_caption: true = merge cả audio và caption, false = chỉ merge audio
 */
async fn merge_video_with_audio(
    video_path: &str,
    audio_path: &str,
    output_path: &str,
    has_caption: bool,
) -> Result<(), String> {
    if has_caption {
        // Merge video with audio and caption
        merge_audio_with_caption(video_path, audio_path, output_path).await
    } else {
        // Just merge video with audio
        merge_audio_without_caption(video_path, audio_path, output_path).await
    }
}

/// Tải ảnh từ URL AI và biến nó thành một đoạn Video cinematic
#[tauri::command]
pub async fn create_video_from_ai_image(
    image_url: String,
    effect: String,
    duration: i32,
    processes: tauri::State<'_, ProcessStore>,
) -> Result<String, String> {
    println!("Bắt đầu tạo Video AI từ ảnh: {}", image_url);
    
    let ffmpeg_path = find_ffmpeg_or_error()?;
    
    // 1. Tạo thư mục tạm để làm việc
    let temp_dir = std::env::temp_dir().join(format!("ai_video_{}", Uuid::new_v4()));
    fs::create_dir_all(&temp_dir).map_err(|e| format!("Lỗi tạo folder tạm: {}", e))?;
    
    let image_path = temp_dir.join("source_image.png");
    let video_path = temp_dir.join("result_video.mp4");

    // 2. Tải ảnh từ AI URL
    let client = reqwest::Client::new();
    let response = client.get(&image_url).send().await
        .map_err(|e| format!("Lỗi khi kết nối tới AI Image server: {}", e))?;
    
    let bytes = response.bytes().await
        .map_err(|e| format!("Lỗi khi đọc dữ liệu ảnh: {}", e))?;
    
    fs::write(&image_path, bytes).map_err(|e| format!("Lỗi lưu ảnh tạm: {}", e))?;

    // 3. Xử lý kĩ thuật FFmpeg để tạo Video Cinematic
    let total_frames = duration * 25;
    let width = 1280;
    let height = 720;
    let video_quality = "fullhd";
    
    let filter = build_image_filter_string(
        &effect,
        width,
        height,
        total_frames,
        0,
        video_quality
    );

    let mut cmd = tokio::process::Command::new(ffmpeg_path);
    cmd.arg("-loop").arg("1")
       .arg("-framerate").arg("25")
       .arg("-i").arg(image_path.to_str().unwrap())
       .arg("-vf").arg(filter)
       .arg("-t").arg(duration.to_string())
       .arg("-r").arg("25")
       .arg("-pix_fmt").arg("yuv420p");

    let encoder = get_best_encoder();
    cmd.arg("-c:v").arg(&encoder);
    
    if encoder == "h264_videotoolbox" {
        cmd.arg("-q:v").arg("60").arg("-tag:v").arg("hvc1");
    } else {
        cmd.arg("-crf").arg("20").arg("-preset").arg("medium");
    }

    cmd.arg("-y").arg(video_path.to_str().unwrap());

    // 4. Chạy FFmpeg và đợi
    let process_id = format!("ai_v_{}", Uuid::new_v4());
    let output = run_ffmpeg_with_cancellation(cmd, &process_id, &processes).await?;
    
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("FFmpeg Error: {}", err));
    }

    Ok(video_path.to_string_lossy().to_string())
}

