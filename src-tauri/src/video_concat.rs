use crate::video::find_ffmpeg;
use crate::process::ProcessStore;
use std::path::{PathBuf, Path};
use std::fs;
use std::io::Write;
use uuid::Uuid;

/**
 * Xác định resolution dựa trên video quality và aspect ratio
 * Trả về (width, height) cho video output
 */
fn get_output_resolution(quality: &str, aspect_ratio: &str) -> (i32, i32) {
    // Xác định base resolution dựa trên quality
    let (base_width, base_height) = match quality {
        "hd" => (1280, 720),      // HD 720p
        "fullhd" => (1920, 1080), // Full HD 1080p
        "2K" => (2048, 1080),     // 2K
        "4K" => (3840, 2160),     // 4K
        _ => (1920, 1080),        // Default Full HD
    };
    
    // Áp dụng aspect ratio: 9:16 (dọc) hoặc 16:9 (ngang)
    match aspect_ratio {
        "9:16" => (base_height, base_width), // Swap cho video dọc (9:16)
        "16:9" => (base_width, base_height),  // Giữ nguyên cho video ngang (16:9)
        _ => (base_width, base_height),       // Default 16:9
    }
}

/**
 * Xác định preset và CRF dựa trên video quality
 * Trả về (preset, crf) cho encoding
 */
fn get_encoding_params(quality: &str) -> (&str, &str) {
    match quality {
        "hd" => ("medium", "23"),        // HD: CRF 23 cho chất lượng tốt
        "fullhd" => ("medium", "22"),    // Full HD: CRF 22 cho chất lượng tốt hơn
        "2K" => ("medium", "20"),        // 2K: CRF 20 cho chất lượng cao
        "4K" => ("medium", "20"),        // 4K: CRF 20 để cân bằng tốc độ và chất lượng
        _ => ("medium", "22"),           // Default Full HD
    }
}

/**
 * Tạo file concat list cho ffmpeg concat demuxer
 */
fn create_concat_list_file(files: &[String], work_dir: &Path) -> Result<PathBuf, String> {
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
 * Sử dụng file list .txt và copy stream, không re-encode
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
        "Không tìm thấy ffmpeg. Vui lòng cài đặt: brew install ffmpeg".to_string()
    })?;
    
    // Tạo thư mục tạm để lưu file concat list
    let output_path_buf = PathBuf::from(output_path);
    let work_dir = output_path_buf.parent()
        .ok_or_else(|| "Không thể xác định thư mục output".to_string())?;
    
    // Tạo file concat list
    let concat_list_file = create_concat_list_file(&segment_files, work_dir)?;
    
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
        eprintln!("FFmpeg Error Details: {}", error_msg);
        return Err(format!("Lỗi khi concat video (không hiệu ứng): {}", error_msg));
    }
    
    Ok(())
}

/**
 * Ghép 2 video với transition (Pipeline approach - tối ưu cho hiệu suất)
 * Chỉ xử lý 2 video mỗi lần, phần lớn video được copy, chỉ encode phần transition
 * 
 * # Arguments
 * * `video1_duration` - Duration của video đầu tiên (có thể là segment gốc hoặc output đã ghép)
 */
async fn merge_two_videos_with_transition(
    video1_path: &str,
    video2_path: &str,
    output_path: &str,
    transition_type: &str,
    transition_duration: f64,
    video_quality: &str,
    video_aspect_ratio: &str,
    video1_duration: f64, // Duration của video đầu tiên (có thể là segment hoặc output đã ghép)
    video2_duration: f64, // Duration của video thứ hai (thường là segment_duration)
    force_scale: bool,
    is_intermediate: bool,
) -> Result<(), String> {
    let ffmpeg_path = find_ffmpeg().ok_or_else(|| {
        "Không tìm thấy ffmpeg. Vui lòng cài đặt: brew install ffmpeg".to_string()
    })?;
    
    // Xác định resolution và encoding params
    let (width, height) = if force_scale {
        get_output_resolution(video_quality, video_aspect_ratio)
    } else {
        (0, 0)
    };
    
    // Chọn preset: ultrafast cho file trung gian, quality-based cho file cuối cùng
    let (preset, crf) = if is_intermediate {
        ("ultrafast", "20") // Nhanh cho file trung gian
    } else {
        get_encoding_params(video_quality)
    };
    
    // Build filter_complex cho 2 video với transition
    // Format: [0:v][1:v]xfade=transition=type:duration=d:offset=offset[video_out]
    // Offset = duration của video1 - transition_duration
    let offset = video1_duration - transition_duration;
    
    let filter_complex = if force_scale {
        format!(
            "[0:v]scale={}:{},setsar=1[v0];[1:v]scale={}:{},setsar=1[v1];[v0][v1]xfade=transition={}:duration={}:offset={}[video_out]",
            width, height, width, height, transition_type, transition_duration, offset
        )
    } else {
        format!(
            "[0:v][1:v]xfade=transition={}:duration={}:offset={}[video_out]",
            transition_type, transition_duration, offset
        )
    };
    
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
        .arg(preset)
        .arg("-crf")
        .arg(crf)
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
        eprintln!("FFmpeg Error Details: {}", error_msg);
        return Err(format!("Lỗi khi ghép 2 video với transition: {}", error_msg));
    }
    
    Ok(())
}

/**
 * Concat video segments với transitions sử dụng Pipeline approach
 * Ghép từng cặp video một: (v1+v2)→out1, (out1+v3)→out2, ... → final
 * Tối ưu hiệu suất vì chỉ encode phần transition, phần còn lại copy
 */
async fn concat_with_pipeline_approach(
    segment_files: Vec<String>,
    video_effect_type: &str,
    video_quality: &str,
    video_aspect_ratio: &str,
    segment_duration: i32,
    transition_duration: f64,
    output_path: &str,
    force_scale: bool,
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
    
    println!("Pipeline: Ghép video 1 và 2...");
    merge_two_videos_with_transition(
        &segment_files[0],
        &segment_files[1],
        &current_output_str,
        video_effect_type,
        transition_duration,
        video_quality,
        video_aspect_ratio,
        segment_duration as f64, // Duration của video đầu tiên
        segment_duration as f64, // Duration của video thứ hai
        force_scale,
        true, // Là file trung gian
    ).await.map_err(|e| format!("Lỗi khi ghép video 1-2: {}", e))?;
    
    // Bước 2: Ghép output hiện tại với các video tiếp theo
    for (idx, next_video) in segment_files.iter().enumerate().skip(2) {
        let next_output = pipeline_work_dir.join(format!("pipeline_{:03}.mp4", idx));
        let next_output_str = next_output.to_string_lossy().to_string();
        
        println!("Pipeline: Ghép output ({} videos) với video {}...", idx + 1, idx + 2);
        
        merge_two_videos_with_transition(
            &current_output_str,
            next_video,
            &next_output_str,
            video_effect_type,
            transition_duration,
            video_quality,
            video_aspect_ratio,
            current_duration, // Duration của output hiện tại (đã ghép nhiều video)
            segment_duration as f64, // Duration của video tiếp theo
            force_scale,
            idx < segment_files.len() - 2, // Chỉ file cuối cùng mới không phải intermediate
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
    
    // Cleanup: Xóa file trung gian cuối cùng và thư mục
    let _ = fs::remove_file(&current_output_str);
    let _ = fs::remove_dir_all(&pipeline_work_dir);
    
    Ok(())
}

/**
 * Helper function để concat video segments với transitions
 * Tự động chọn phương pháp tối ưu nhất:
 * - Không hiệu ứng: Concat demuxer (copy stream - rất nhanh)
 * - Có hiệu ứng: Pipeline approach (ghép từng cặp - tối ưu)
 * 
 * # Arguments
 * * `force_scale` - Nếu true, sẽ scale tất cả segments về cùng resolution. 
 *                   Nếu false (default), giả định segments đã có cùng resolution và không scale.
 */
pub async fn concat_video_segments_with_transitions_helper(
    segment_files: Vec<String>,
    video_effect_type: &str,
    video_quality: &str,
    video_aspect_ratio: &str,
    segment_duration: i32,
    transition_duration: f64,
    output_path: &str,
    force_scale: bool,
) -> Result<(), String> {
    if segment_files.is_empty() {
        return Err("Cần ít nhất một video segment".to_string());
    }
    
    if segment_files.len() == 1 {
        // Chỉ có 1 segment, không cần concat
        return Ok(());
    }
    
    // === XỬ LÝ RIÊNG CHO TRƯỜNG HỢP KHÔNG CÓ HIỆU ỨNG ===
    // Nếu không có hiệu ứng, dùng concat demuxer với file list để nhanh hơn nhiều
    if video_effect_type == "none" {
        return concat_without_effects_fast(segment_files, output_path).await;
    }
    
    // === XỬ LÝ VỚI HIỆU ỨNG: DÙNG PIPELINE APPROACH ===
    // Pipeline approach tối ưu hơn batch processing cho trường hợp có transition
    // Vì chỉ encode phần transition, phần còn lại copy
    return concat_with_pipeline_approach(
        segment_files,
        video_effect_type,
        video_quality,
        video_aspect_ratio,
        segment_duration,
        transition_duration,
        output_path,
        force_scale,
    ).await;
}

/**
 * Concat các video segments với xfade transitions (Tauri command)
 * 
 * # Arguments
 * * `segment_files` - Danh sách đường dẫn đến các video segments
 * * `video_effect_type` - Loại transition effect (hrwind, hlwind, dissolve, etc.)
 * * `video_quality` - Chất lượng video output (hd, fullhd, 2K, 4K)
 * * `video_aspect_ratio` - Aspect ratio (16:9 hoặc 9:16)
 * * `segment_duration` - Thời lượng mỗi segment (giây)
 * * `transition_duration` - Thời lượng transition (giây, mặc định 1.0)
 * * `output_path` - Đường dẫn file output
 * * `force_scale` - Nếu true, sẽ scale tất cả segments về cùng resolution. Default: false
 * * `processes` - ProcessStore để quản lý process
 */
#[tauri::command]
pub async fn concat_video_segments_with_transitions(
    segment_files: Vec<String>,
    video_effect_type: String,
    video_quality: String,
    video_aspect_ratio: String,
    segment_duration: i32,
    transition_duration: f64,
    output_path: String,
    force_scale: Option<bool>,
    _processes: tauri::State<'_, ProcessStore>,  // Reserved for future use
) -> Result<String, String> {
    if segment_files.is_empty() {
        return Err("Cần ít nhất một video segment".to_string());
    }
    
    if segment_files.len() == 1 {
        // Chỉ có 1 segment, không cần concat, chỉ cần copy hoặc re-encode
        return Err("Cần ít nhất 2 video segments để concat với transitions".to_string());
    }
    
    // Default force_scale = false (không scale nếu segments đã cùng resolution)
    let force_scale = force_scale.unwrap_or(false);
    
    // Gọi helper function
    concat_video_segments_with_transitions_helper(
        segment_files,
        &video_effect_type,
        &video_quality,
        &video_aspect_ratio,
        segment_duration,
        transition_duration,
        &output_path,
        force_scale,
    ).await?;
    
    // Generate process ID
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let process_id = format!("concat_video_{}", timestamp);
    
    Ok(process_id)
}

