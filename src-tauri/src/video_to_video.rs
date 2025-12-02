use crate::utils::find_ffmpeg_by_os::run_ffmpeg;
use crate::utils::merge_audio_to_video::{merge_audio_without_caption, merge_audio_with_caption};
use crate::file_audio::find_ffprobe;
use std::path::PathBuf;
use std::fs;
use std::io::Write;
use std::process::Command;
use uuid::Uuid;
use futures::future;

/// Helper function để tạo timestamp
fn get_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Helper function để lấy duration của nhiều video files song song
async fn get_video_durations(video_files: &[String]) -> Result<Vec<f64>, String> {
    if video_files.is_empty() {
        return Err("Danh sách video rỗng".to_string());
    }
    
    // Nếu chỉ có 1 video, không cần tạo futures và join_all
    if video_files.len() == 1 {
        let duration = get_video_duration(&video_files[0]).await?;
        return Ok(vec![duration]);
    }
    
    // Nhiều video: lấy durations song song
    let futures: Vec<_> = video_files.iter()
        .map(|path| get_video_duration(path))
        .collect();
    
    future::try_join_all(futures).await
}

/// Helper function để lấy duration của video file bằng ffprobe
async fn get_video_duration(video_path: &str) -> Result<f64, String> {
    let ffprobe_path = find_ffprobe().ok_or_else(|| {
        "Không tìm thấy ffprobe. Vui lòng cài đặt ffmpeg (bao gồm ffprobe)".to_string()
    })?;
    
    // Kiểm tra file tồn tại
    let path = PathBuf::from(video_path);
    if !path.exists() {
        return Err(format!("File không tồn tại: {}", video_path));
    }
    
    // Sử dụng ffprobe để đọc duration
    let output = Command::new(&ffprobe_path)
        .arg("-v")
        .arg("error")
        .arg("-show_entries")
        .arg("format=duration")
        .arg("-of")
        .arg("default=noprint_wrappers=1:nokey=1")
        .arg(video_path)
        .output()
        .map_err(|e| format!("Lỗi khi chạy ffprobe: {}", e))?;
    
    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("ffprobe error: {}", error_msg));
    }
    
    // Parse output
    let duration_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    
    if duration_str.is_empty() {
        return Err("Không thể đọc duration từ file".to_string());
    }
    
    let duration: f64 = duration_str.parse()
        .map_err(|_| format!("Không thể parse duration: {}", duration_str))?;
    
    if duration.is_nan() || duration <= 0.0 || !duration.is_finite() {
        return Err(format!("Duration không hợp lệ: {}", duration));
    }
    
    Ok(duration)
}

/// Base scale filter: scale và pad để fit vào resolution (giữ nguyên tỉ lệ)
fn build_base_scale(width: i32, height: i32) -> String {
    format!("scale={}:{}:force_original_aspect_ratio=decrease,pad={}:{}:(ow-iw)/2:(oh-ih)/2", 
        width, height, width, height)
}

/**
 * Chuẩn bị video trước khi ghép: normalize resolution, framerate, codec và tùy chọn xóa audio
 * Tất cả videos sẽ được scale về cùng resolution
 */
async fn prepare_video_before_merge(
    video_path: &str,
    output_path: &str,
    target_width: i32,
    target_height: i32,
    crf: &str,
    remove_audio: bool,
) -> Result<(), String> {
    let scale_filter = build_base_scale(target_width, target_height);
    
    let mut cmd = run_ffmpeg()?;
    cmd.arg("-i")
        .arg(video_path)
        .arg("-vf")
        .arg(&scale_filter)
        .arg("-r")
        .arg("30") // Normalize framerate về 30fps
        .arg("-vsync")
        .arg("cfr") // Constant framerate
        .arg("-c:v")
        .arg("libx264")
        .arg("-preset")
        .arg("medium")
        .arg("-crf")
        .arg(crf)
        .arg("-pix_fmt")
        .arg("yuv420p")
        .arg("-g")
        .arg("60") // GOP size = 2x framerate
        .arg("-bf")
        .arg("2")
        .arg("-refs")
        .arg("4");
    
    // Chỉ xóa audio nếu remove_audio = true
    if remove_audio {
        cmd.arg("-an"); // Xóa audio
    } else {
        cmd.arg("-c:a").arg("copy"); // Copy audio stream nếu có
    }
    
    cmd.arg("-y")
        .arg(output_path);
    
    let output = cmd.output().await
        .map_err(|e| format!("Lỗi khi chuẩn bị video: {}", e))?;
    
    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Lỗi khi chuẩn bị video: {}", error_msg));
    }
    
    Ok(())
}

/// Ghép trực tiếp các video với scale và concat trong một lần chạy ffmpeg
async fn concat_videos_direct(
    video_files: &[String],
    output_path: &str,
    width: i32,
    height: i32,
    preset: &str,
    crf: &str,
) -> Result<(), String> {
    if video_files.is_empty() {
        return Err("Cần ít nhất một video".to_string());
    }
    
    if video_files.len() == 1 {
        // Chỉ có 1 video, scale và copy trực tiếp
        let scale_filter = build_base_scale(width, height);
        let mut cmd = run_ffmpeg()?;
        cmd.arg("-i")
            .arg(&video_files[0])
            .arg("-vf")
            .arg(&scale_filter)
            .arg("-r")
            .arg("25")
            .arg("-vsync")
            .arg("cfr")
            .arg("-c:v")
            .arg("libx264")
            .arg("-preset")
            .arg(preset)
            .arg("-crf")
            .arg(crf)
            .arg("-pix_fmt")
            .arg("yuv420p")
            .arg("-color_range")
            .arg("1")
            .arg("-g")
            .arg("50")
            .arg("-bf")
            .arg("2")
            .arg("-refs")
            .arg("4")
            .arg("-an") // Bỏ audio, audio sẽ được merge sau
            .arg("-y")
            .arg(output_path);
        
        let output = cmd.output().await
            .map_err(|e| format!("Lỗi khi xử lý video: {}", e))?;
        
        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Lỗi khi xử lý video: {}", error_msg));
        }
        
        return Ok(());
    }
    
    // Nhiều video: dùng filter_complex để scale và concat
    let scale_filter = build_base_scale(width, height);
    
    // Build filter_complex: scale từng video rồi concat
    // Format: [0:v]scale=...:pad=...[v0]; [1:v]scale=...:pad=...[v1]; [v0][v1]concat=n=2:v=1:a=0[vout]
    let mut filter_parts = Vec::new();
    let mut concat_inputs = Vec::new();
    
    for (i, _) in video_files.iter().enumerate() {
        // Scale từng video
        filter_parts.push(format!("[{}:v]{}[v{}]", i, scale_filter, i));
        concat_inputs.push(format!("[v{}]", i));
    }
    
    // Concat tất cả video đã scale
    let concat_filter = format!("{}concat=n={}:v=1:a=0[vout]", 
        concat_inputs.join(""), video_files.len());
    filter_parts.push(concat_filter);
    
    let filter_complex = filter_parts.join("; ");
    
    // Build ffmpeg command
    let mut cmd = run_ffmpeg()?;
    
    // Thêm tất cả input video
    for video_file in video_files {
        cmd.arg("-i").arg(video_file);
    }
    
    // Thêm filter_complex
    cmd.arg("-filter_complex")
        .arg(&filter_complex)
        .arg("-map")
        .arg("[vout]")
        .arg("-r")
        .arg("25")
        .arg("-vsync")
        .arg("cfr")
        .arg("-c:v")
        .arg("libx264")
        .arg("-preset")
        .arg(preset)
        .arg("-crf")
        .arg(crf)
        .arg("-pix_fmt")
        .arg("yuv420p")
        .arg("-color_range")
        .arg("1")
        .arg("-g")
        .arg("50")
        .arg("-bf")
        .arg("2")
        .arg("-refs")
        .arg("4")
        .arg("-an") // Bỏ audio, audio sẽ được merge sau
        .arg("-y")
        .arg(output_path);
    
    // Chạy và đợi process hoàn thành
    let output = cmd.output().await
        .map_err(|e| format!("Lỗi khi ghép video: {}", e))?;
    
    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Lỗi khi ghép video: {}", error_msg));
    }
    
    Ok(())
}

/// Ghép nhiều video lại với nhau theo thứ tự đã sắp xếp
/// - Đầu vào: danh sách file video (đã được sắp xếp ở UI)
/// - Video sẽ được scale + pad về cùng độ phân giải theo `video_quality` và `video_aspect_ratio`
/// - Logic chọn chất lượng (quality), hiệu ứng chuyển cảnh (video_effect_type),
///   merge audio và caption được giữ nguyên so với image_to_video
#[tauri::command]
pub async fn merge_videos(
    video_files: Vec<String>,
    video_quality: String,
    video_aspect_ratio: String,
    video_effect_type: String,
    audio_files: Vec<String>,
    output_folder: String,
    is_auto_caption: bool,
) -> Result<String, String> {
    if video_files.is_empty() {
        return Err("Cần ít nhất một video".to_string());
    }
    
    // Validate output folder
    let output_path = PathBuf::from(&output_folder);
    if !output_path.exists() || !output_path.is_dir() {
        return Err("Thư mục output không hợp lệ".to_string());
    }
    
    // Tạo timestamp để tránh conflict
    let timestamp = get_timestamp();
    
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
    
    // Tối ưu preset và CRF dựa trên chất lượng video
    let (preset, crf) = match video_quality.as_str() {
        "hd" => ("medium", "23"),        // HD: CRF 23 cho chất lượng tốt
        "fullhd" => ("medium", "22"),    // Full HD: CRF 22 cho chất lượng tốt hơn
        "2K" => ("medium", "20"),        // 2K: CRF 20 cho chất lượng cao
        "4K" => ("medium", "20"),        // 4K: CRF 20 để cân bằng tốc độ và chất lượng
        _ => ("medium", "22"),           // Default Full HD
    };
    
    // Ghép trực tiếp các video với scale và concat trong một lần chạy ffmpeg
    let output_filename = format!("final_video_{}.mp4", timestamp);
    let final_video = output_path.join(&output_filename);
    let final_video_path_str = final_video.to_string_lossy().to_string();
    
    // Hiệu ứng chuyển cảnh hiện tại chỉ hỗ trợ "none" (concat thẳng).
    // Các giá trị khác của video_effect_type có thể được hỗ trợ sau.
    let _ = video_effect_type; // tránh warning unused, logic chọn hiệu ứng giữ nguyên ở UI
    
    concat_videos_direct(
        &video_files,
        &final_video_path_str,
        width,
        height,
        preset,
        crf,
    ).await?;
    
    let final_video_path = final_video_path_str;
    
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
            is_auto_caption, // Use user's choice for auto caption
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
        // Chỉ có 1 segment, xóa audio và copy video stream
        let mut cmd = run_ffmpeg()?;
        cmd.arg("-i")
            .arg(&segment_files[0])
            .arg("-c:v")
            .arg("copy") // Copy video stream - không re-encode
            .arg("-an") // Xóa audio stream gốc
            .arg("-y")
            .arg(output_path);
        
        let output = cmd.output().await
            .map_err(|e| format!("Lỗi khi xóa audio từ video: {}", e))?;
        
        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Lỗi khi xóa audio từ video: {}", error_msg));
        }
        
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
        .arg("-an") // Xóa audio stream gốc - không ảnh hưởng performance vì chỉ bỏ qua audio
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
    crf: &str,
) -> Result<(), String> {
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
    let mut cmd = run_ffmpeg()?;
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
        // Chỉ có 1 segment, xóa audio và copy video stream
        let mut cmd = run_ffmpeg()?;
        cmd.arg("-i")
            .arg(&segment_files[0])
            .arg("-c:v")
            .arg("copy") // Copy video stream - không re-encode
            .arg("-an") // Xóa audio stream gốc
            .arg("-y")
            .arg(output_path);
        
        let output = cmd.output().await
            .map_err(|e| format!("Lỗi khi xóa audio từ video: {}", e))?;
        
        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Lỗi khi xóa audio từ video: {}", error_msg));
        }
        
        return Ok(());
    }
    
    // Tạo thư mục tạm để lưu các file trung gian
    let output_path_buf = PathBuf::from(output_path);
    let work_dir = output_path_buf.parent()
        .ok_or_else(|| "Không thể xác định thư mục output".to_string())?;
    
    let timestamp = get_timestamp();
    
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
        "20", // Default CRF for pipeline approach
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
            "20", // Default CRF for pipeline approach
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
        // Merge video với audio và caption
        merge_audio_with_caption(video_path, audio_path, output_path).await
    } else {
        // Chỉ merge video với audio
        merge_audio_without_caption(video_path, audio_path, output_path).await
    }
}

/**
 * Concat videos với transitions sử dụng actual video durations
 */
async fn concat_videos_with_transitions(
    video_files: &[String],
    video_effect_type: &str,
    transition_duration: f64,
    output_path: &str,
    crf: &str,
    target_width: i32,
    target_height: i32,
    remove_original_audio: bool,
) -> Result<(), String> {
    if video_files.is_empty() {
        return Err("Cần ít nhất một video".to_string());
    }
    
    let output_path_buf = PathBuf::from(output_path);
    let work_dir = output_path_buf.parent()
        .ok_or_else(|| "Không thể xác định thư mục output".to_string())?;
    
    let timestamp = get_timestamp();
    let prepare_dir = work_dir.join(format!("prepare_temp_{}", timestamp));
    fs::create_dir_all(&prepare_dir)
        .map_err(|e| format!("Lỗi khi tạo prepare directory: {}", e))?;
    
    // Bước 1: Prepare tất cả videos về cùng resolution, framerate, codec và xóa audio
    let mut prepared_videos = Vec::new();
    for (idx, video_file) in video_files.iter().enumerate() {
        let prepared_path = prepare_dir.join(format!("prepared_{:04}.mp4", idx));
        let prepared_path_str = prepared_path.to_string_lossy().to_string();
        
        prepare_video_before_merge(
            video_file,
            &prepared_path_str,
            target_width,
            target_height,
            crf,
            remove_original_audio,
        ).await.map_err(|e| format!("Lỗi khi chuẩn bị video {}: {}", idx + 1, e))?;
        
        prepared_videos.push(prepared_path_str);
    }
    
    if video_files.len() == 1 {
        // Chỉ có 1 video, copy prepared video vào output
        fs::copy(&prepared_videos[0], output_path)
            .map_err(|e| format!("Lỗi khi copy video: {}", e))?;
        
        // Cleanup
        let _ = fs::remove_dir_all(&prepare_dir);
        return Ok(());
    }
    
    // Nếu không có hiệu ứng, dùng concat demuxer với prepared videos
    if video_effect_type == "none" {
        let result = concat_without_effects_fast(prepared_videos, output_path).await;
        
        // Cleanup
        let _ = fs::remove_dir_all(&prepare_dir);
        return result;
    }
    
    // Có hiệu ứng: dùng pipeline approach với actual durations
    let pipeline_work_dir = work_dir.join(format!("pipeline_temp_{}", timestamp));
    fs::create_dir_all(&pipeline_work_dir)
        .map_err(|e| format!("Lỗi khi tạo pipeline work directory: {}", e))?;
    
    // Lấy tất cả durations từ prepared videos (sẽ giống nhau vì đã normalize)
    let video_durations = get_video_durations(&prepared_videos).await?;
    
    // Bước 1: Ghép video đầu tiên với video thứ 2
    let current_output = pipeline_work_dir.join("pipeline_000.mp4");
    let mut current_output_str = current_output.to_string_lossy().to_string();
    
    let first_video_duration = video_durations[0];
    let second_video_duration = video_durations[1];
    let mut current_duration = first_video_duration + second_video_duration - transition_duration;
    
    merge_two_videos_with_transition(
        &prepared_videos[0],
        &prepared_videos[1],
        &current_output_str,
        video_effect_type,
        transition_duration,
        first_video_duration,
        second_video_duration,
        crf,
    ).await.map_err(|e| format!("Lỗi khi ghép video 1-2: {}", e))?;
    
    // Bước 2: Ghép output hiện tại với các video tiếp theo
    for (idx, next_video) in prepared_videos.iter().enumerate().skip(2) {
        let next_output = pipeline_work_dir.join(format!("pipeline_{:03}.mp4", idx));
        let next_output_str = next_output.to_string_lossy().to_string();
        
        let next_video_duration = video_durations[idx];
        
        merge_two_videos_with_transition(
            &current_output_str,
            next_video,
            &next_output_str,
            video_effect_type,
            transition_duration,
            current_duration,
            next_video_duration,
            crf,
        ).await.map_err(|e| format!("Lỗi khi ghép video {}: {}", idx + 1, e))?;
        
        // Cập nhật duration cho output mới
        current_duration = current_duration + next_video_duration - transition_duration;
        
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
    
    // Bước 5: Xóa thư mục pipeline tạm
    let _ = fs::remove_dir_all(&pipeline_work_dir);
    
    // Bước 6: Cleanup prepared videos directory
    let _ = fs::remove_dir_all(&prepare_dir);
    
    Ok(())
}

/**
 * Tạo video từ danh sách video với transitions, audio và caption
 */
#[tauri::command]
pub async fn create_video_from_video(
    video_files: Vec<String>,
    audio_files: Vec<String>,
    output_path: String,
    crf: String,
    is_has_auto_caption: bool,
    video_effect_type: String,
    video_quality: String,
    video_aspect_ratio: String,
    remove_original_audio: bool,
) -> Result<String, String> {
    if video_files.is_empty() {
        return Err("Cần ít nhất một video".to_string());
    }
    
    // Validate output path
    let output_path_buf = PathBuf::from(&output_path);
    let output_dir = output_path_buf.parent()
        .ok_or_else(|| "Không thể xác định thư mục output".to_string())?;
    
    if !output_dir.exists() || !output_dir.is_dir() {
        return Err("Thư mục output không hợp lệ".to_string());
    }
    
    // Xác định resolution target dựa trên video_quality và video_aspect_ratio (giống merge_videos)
    let (base_width, base_height) = match video_quality.as_str() {
        "hd" => (1280, 720),      // HD 720p
        "fullhd" => (1920, 1080), // Full HD 1080p
        "2K" => (2048, 1080),     // 2K
        "4K" => (3840, 2160),     // 4K
        _ => (1920, 1080),        // Default Full HD
    };
    
    // Áp dụng aspect ratio: 9:16 (dọc) hoặc 16:9 (ngang)
    let (target_width, target_height) = match video_aspect_ratio.as_str() {
        "9:16" => (base_height, base_width), // Swap cho video dọc (9:16)
        "16:9" => (base_width, base_height),  // Giữ nguyên cho video ngang (16:9)
        _ => (base_width, base_height),       // Default 16:9
    };
    
    // Transition duration mặc định (1 giây)
    let transition_duration = 1.0;
    
    // Tạo timestamp một lần để reuse
    let timestamp = get_timestamp();
    
    // Bước 1: Concat videos với transitions
    let temp_video_path = output_dir.join(format!("temp_concat_{}.mp4", timestamp));
    let temp_video_path_str = temp_video_path.to_string_lossy().to_string();
    
    concat_videos_with_transitions(
        &video_files,
        &video_effect_type,
        transition_duration,
        &temp_video_path_str,
        &crf,
        target_width,
        target_height,
        remove_original_audio,
    ).await?;
    
    // Bước 2: Xử lý audio nếu có
    let final_output_path = if !audio_files.is_empty() {
        // Lưu số lượng audio files trước khi move
        let audio_files_count = audio_files.len();
        
        // Merge audio files nếu có nhiều hơn 1 file
        let merged_audio_path = if audio_files_count > 1 {
            let merged_audio_filename = format!("merged_audio_{}.mp3", timestamp);
            let merged_audio_file = output_dir.join(&merged_audio_filename);
            let merged_audio_path_str = merged_audio_file.to_string_lossy().to_string();
            
            merge_audio_files(audio_files, &merged_audio_path_str).await
                .map_err(|e| format!("Lỗi khi merge audio: {}", e))?;
            
            merged_audio_path_str
        } else {
            audio_files[0].clone()
        };
        
        // Merge video với audio (và caption nếu cần)
        merge_video_with_audio(
            &temp_video_path_str,
            &merged_audio_path,
            &output_path,
            is_has_auto_caption,
        ).await.map_err(|e| format!("Lỗi khi merge video với audio: {}", e))?;
        
        // Xóa file audio tạm nếu đã merge nhiều file
        if audio_files_count > 1 {
            let _ = fs::remove_file(&merged_audio_path);
        }
        
        // Xóa file video tạm
        let _ = fs::remove_file(&temp_video_path_str);
        
        output_path
    } else {
        // Không có audio, chỉ cần rename temp file thành output
        fs::rename(&temp_video_path_str, &output_path)
            .map_err(|e| format!("Lỗi khi rename file: {}", e))?;
        output_path
    };
    
    Ok(format!("Video đã được tạo thành công! {}", final_output_path))
}

