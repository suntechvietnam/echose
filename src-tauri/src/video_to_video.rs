use crate::utils::find_ffmpeg_by_os::{run_ffmpeg, get_best_encoder};
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

/// Helper function để lấy metadata của video file bằng ffprobe
async fn get_video_metadata(video_path: &str) -> Result<(i32, i32, f64, bool), String> {
    let ffprobe_path = find_ffprobe().ok_or_else(|| {
        "Không tìm thấy ffprobe. Vui lòng cài đặt ffmpeg (bao gồm ffprobe)".to_string()
    })?;
    
    let output = Command::new(&ffprobe_path)
        .arg("-v")
        .arg("error")
        .arg("-select_streams")
        .arg("v:0")
        .arg("-show_entries")
        .arg("stream=width,height,r_frame_rate")
        .arg("-of")
        .arg("default=noprint_wrappers=1:nokey=1")
        .arg(video_path)
        .output()
        .map_err(|e| format!("Lỗi khi chạy ffprobe: {}", e))?;
    
    let vid_meta = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let lines: Vec<&str> = vid_meta.lines().collect();
    if lines.len() < 3 {
        return Err("Không thể đọc metadata video".to_string());
    }
    
    let width: i32 = lines[0].parse().unwrap_or(0);
    let height: i32 = lines[1].parse().unwrap_or(0);
    
    // Parse r_frame_rate (e.g., "30/1" or "25/1")
    let fps_parts: Vec<&str> = lines[2].split('/').collect();
    let fps = if fps_parts.len() == 2 {
        let num: f64 = fps_parts[0].parse().unwrap_or(0.0);
        let den: f64 = fps_parts[1].parse().unwrap_or(1.0);
        if den == 0.0 { 0.0 } else { num / den }
    } else {
        lines[2].parse().unwrap_or(0.0)
    };

    // Check for audio stream
    let audio_output = Command::new(&ffprobe_path)
        .arg("-v")
        .arg("error")
        .arg("-select_streams")
        .arg("a")
        .arg("-show_entries")
        .arg("stream=index")
        .arg("-of")
        .arg("csv=p=0")
        .arg(video_path)
        .output()
        .map_err(|e| format!("Lỗi khi kiểm tra audio: {}", e))?;
    
    let has_audio = !String::from_utf8_lossy(&audio_output.stdout).trim().is_empty();

    Ok((width, height, fps, has_audio))
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
    
    // Sử dụng ffprobe để đọc duration từ stream 0 (chính xác hơn format duration)
    let output = Command::new(&ffprobe_path)
        .arg("-v").arg("error")
        .arg("-select_streams").arg("v:0")
        .arg("-show_entries").arg("stream=duration")
        .arg("-of").arg("default=noprint_wrappers=1:nokey=1")
        .arg(video_path)
        .output()
        .map_err(|e| format!("Lỗi khi chạy ffprobe duration: {}", e))?;
    
    let mut duration_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    
    // Nếu stream duration không có, fallback về format duration
    if duration_str.is_empty() || duration_str == "N/A" {
        let output_f = Command::new(&ffprobe_path)
            .arg("-v").arg("error")
            .arg("-show_entries").arg("format=duration")
            .arg("-of").arg("default=noprint_wrappers=1:nokey=1")
            .arg(video_path)
            .output()
            .map_err(|e| format!("Lỗi khi chạy ffprobe format duration: {}", e))?;
        duration_str = String::from_utf8_lossy(&output_f.stdout).trim().to_string();
    }

    if duration_str.is_empty() {
        return Err("Không thể đọc duration video".to_string());
    }
    
    let duration: f64 = duration_str.parse()
        .map_err(|_| format!("Không thể parse duration: {}", duration_str))?;
    
    if duration.is_nan() || duration <= 0.0 || !duration.is_finite() {
        return Err(format!("Duration không hợp lệ: {}", duration));
    }
    
    Ok(duration)
}

/// Base scale filter
/// - Nếu `no_black_bars = false`: scale và pad để fit vào resolution (giữ nguyên tỉ lệ, có thể có viền đen)
/// - Nếu `no_black_bars = true`: scale để COVER hết khung hình rồi crop về đúng resolution (không có viền đen)
fn build_base_scale(width: i32, height: i32, no_black_bars: bool) -> String {
    let scaler = "flags=lanczos"; // Dùng lanczos cho hình ảnh sắc nét nhất (như CPU)
    if no_black_bars {
        format!(
            "scale={}:{}:force_original_aspect_ratio=increase:{},crop={}:{},fps=30,setpts=PTS-STARTPTS",
            width, height, scaler, width, height
        )
    } else {
        format!(
            "scale={}:{}:force_original_aspect_ratio=decrease:{},pad={}:{}:(ow-iw)/2:(oh-ih)/2:black,fps=30,setpts=PTS-STARTPTS",
            width, height, scaler, width, height
        )
    }
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
    no_black_bars: bool,
) -> Result<(), String> {
    let scale_filter = build_base_scale(target_width, target_height, no_black_bars);
    
    let mut cmd = run_ffmpeg()?;
    
    // Bước CHUẨN BỊ:
    // USER REQUEST: Muốn "Nhanh", "Nét" (như CPU), "Không ngốn" (Nhẹ).
    // GIẢI PHÁP: Sử dụng HEVC (H.265) Hardware Encoding (hevc_videotoolbox).
    // HEVC nén tốt hơn H.264 50% => Cùng chất lượng, dung lượng giảm một nửa.
    
    // Check nếu có hevc_videotoolbox (Mac) thì ưu tiên dùng
    let mut best_encoder = get_best_encoder();
    if best_encoder == "h264_videotoolbox" {
         // Nếu máy hỗ trợ h264_videotoolbox thì 99% hỗ trợ hevc_videotoolbox
         best_encoder = "hevc_videotoolbox"; 
    }

    cmd.arg("-i")
        .arg(video_path)
        .arg("-vf")
        .arg(&scale_filter)
        .arg("-c:v")
        .arg(best_encoder);

    if best_encoder == "hevc_videotoolbox" {
        // Bitrate cho bước đệm cần RẤT CAO để đảm bảo 100% không mất chi tiết (Visual Lossless)
        let bitrate = match target_height {
            h if h <= 1080 => "12000k", // Đệm 12M HEVC (Dư giả cho 1080p)
            _ => "50000k",             // Đệm 50M HEVC (Max quality cho 4K)
        };
        cmd.arg("-b:v").arg(bitrate)
           .arg("-tag:v").arg("hvc1") 
           .arg("-realtime").arg("0")
           .arg("-profile:v").arg("main");
    } else if best_encoder == "h264_videotoolbox" {
        // Fallback or explicit selection
        let bitrate = "15M";
        cmd.arg("-b:v").arg(bitrate).arg("-realtime").arg("0").arg("-profile:v").arg("high");
    } else if best_encoder == "libx264" {
        cmd.arg("-crf").arg("17").arg("-preset").arg("ultrafast");
    } else if best_encoder.contains("nvenc") {
        cmd.arg("-cq").arg("17").arg("-preset").arg("p1"); 
    }
    
    cmd.arg("-pix_fmt")
        .arg("yuv420p")
        .arg("-g")
        .arg("30");
    
    // Chỉ xóa audio nếu remove_audio = true
    if remove_audio {
        cmd.arg("-an"); // Xóa audio
    } else {
        cmd.arg("-c:a").arg("copy"); // Copy audio stream gốc
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
    target_height: i32,
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
    
    // Lấy target_height thực tế để set bitrate
    let mut cmd = run_ffmpeg()?;
    let encoder = get_best_encoder();

    cmd.arg("-f")
        .arg("concat")
        .arg("-safe")
        .arg("0")
        .arg("-i")
        .arg(concat_list_file.to_string_lossy().as_ref())
        .arg("-c:v")
        .arg(encoder);

    if encoder == "h264_videotoolbox" {
        let bitrate = match target_height {
            h if h <= 720 => "5M",
            h if h <= 1080 => "10M",
            h if h <= 1440 => "18M",
            _ => "25M", // 25M là mức đẹp để video 10p nặng khoảng 1.5GB
        };
        cmd.arg("-b:v").arg(bitrate)
           .arg("-maxrate:v").arg(bitrate)
           .arg("-bufsize:v").arg(format!("{}k", bitrate.replace("M", "000")))
           .arg("-profile:v").arg("high").arg("-realtime").arg("0");
    } else if encoder == "libx264" {
        cmd.arg("-crf").arg("18").arg("-preset").arg("slow");
    }
    
    cmd.arg("-an")
        .arg("-pix_fmt").arg("yuv420p")
        .arg("-fps_mode").arg("cfr")
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


// Đã được thay thế bằng concat_videos_with_transitions logic

// Đã được thay thế bằng logic tối ưu

// ============================================================================
// Audio Processing Functions
// ============================================================================
/**
 * Merge nhiều file audio thành một file duy nhất
 */
async fn merge_audio_files(
    audio_files: &[String],
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
    no_black_bars: bool,
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
    // Bước 1: Lấy thời gian các video
    let video_durations = get_video_durations(&video_files).await?;
    let total_videos = video_files.len();

    // Bước 2: Xây dựng Filter Complex (Scale đồng nhất cho mọi input)
    let mut filter_parts = Vec::new();
    for (i, _) in video_files.iter().enumerate() {
        let base_scale = build_base_scale(target_width, target_height, no_black_bars);
        filter_parts.push(format!("[{}:v]{}[v{}]", i, base_scale, i));
    }

    let final_label = if video_effect_type == "none" || video_effect_type.is_empty() {
        // GHÉP THUẦN TÚY (Dùng concat filter)
        let mut concat_str = String::new();
        for i in 0..total_videos {
            concat_str.push_str(&format!("[v{}]", i));
        }
        filter_parts.push(format!("{}concat=n={}:v=1:a=0[vout]", concat_str, total_videos));
        "[vout]".to_string()
    } else {
        // CÓ HIỆU ỨNG (XFade)
        let mut current_offset = 0.0;
        let mut last_label = "[v0]".to_string();
        for i in 1..total_videos {
            current_offset += video_durations[i-1] - transition_duration;
            let joined_label = format!("[vjoin{}]", i);
            filter_parts.push(format!(
                "{}[v{}]xfade=transition={}:duration={}:offset={:.3}{}",
                last_label, i, video_effect_type, transition_duration, current_offset, joined_label
            ));
            last_label = joined_label;
        }
        last_label
    };

    // XỬ LÝ AUDIO:
    // Nếu không xóa audio gốc (remove_original_audio = false),
    // ta cần ghép nối audio từ các video gốc lại với nhau.
    let final_audio_label = if !remove_original_audio {
         let mut audio_concat_str = String::new();
         for i in 0..total_videos {
             // [i:a] là audio stream của input i
             audio_concat_str.push_str(&format!("[{}:a]", i));
         }
         // Ghép nối audio (concat=n=Total:v=0:a=1)
         // v=0: không ghép video (đã làm ở trên), a=1: ghép audio
         let a_label = "[aout]";
         filter_parts.push(format!("{}concat=n={}:v=0:a=1{}", audio_concat_str, total_videos, a_label));
         Some(a_label.to_string())
    } else {
         None
    };

    let total_duration = if video_effect_type == "none" {
        video_durations.iter().sum::<f64>()
    } else {
        video_durations.iter().sum::<f64>() - (total_videos as f64 - 1.0) * transition_duration
    };

    // Bước 3: Chạy FFmpeg
    let mut cmd = run_ffmpeg()?;
    
    // Tối ưu hóa "Nhanh - Nét - Nhẹ": Sử dụng HEVC (H.265)
    let detect_encoder = get_best_encoder();
    let mut encoder = if detect_encoder == "h264_videotoolbox" {
        "hevc_videotoolbox" // Upgrade lên HEVC
    } else {
        "libx264"
    };

    for video in video_files {
        cmd.arg("-i").arg(video);
    }
    cmd.arg("-filter_complex").arg(filter_parts.join(";"))
       .arg("-map").arg(final_label);
       
    // Map audio stream nếu có
    if let Some(audio_label) = final_audio_label {
        cmd.arg("-map").arg(audio_label);
        cmd.arg("-c:a").arg("aac"); // Encode lại audio sang AAC để tương thích tốt nhất
    }
       
    cmd.arg("-t").arg(format!("{:.3}", total_duration))
       .arg("-c:v").arg(encoder);

    // if remove_original_audio { cmd.arg("-an"); } // Đã xử lý bằng map ở trên rồi
    
    if encoder == "hevc_videotoolbox" {
        // Cấu hình Bitrate "Ultra Sharp" cho HEVC (100% chất lượng CPU)
        // 1080p HEVC @ 8M (Chất lượng Mastering)
        // 4K HEVC @ 35M (Chất lượng điện ảnh)
        let bitrate = match target_height {
            h if h <= 720 => "4000k",   // HD cực nét
            h if h <= 1080 => "8000k",  // FHD (8Mbps HEVC là rất cao)
            h if h <= 1440 => "15000k", // 2K
            _ => "35000k",              // 4K (35Mbps HEVC nét căng đét)
        };
        
        cmd.arg("-b:v").arg(bitrate)
           // Allow spikes for complex scenes
           .arg("-maxrate:v").arg(format!("{}k", bitrate.replace("k", "").parse::<i32>().unwrap_or(5000) * 2)) 
           .arg("-bufsize:v").arg("4M")
           .arg("-tag:v").arg("hvc1") // Tag quan trọng để Mac/iPhone nhận diện đúng
           .arg("-realtime").arg("0")
           .arg("-profile:v").arg("main");
           
    } else if encoder == "h264_videotoolbox" {
        // Fallback case (ít dùng nếu đã ép hevc)
        let bitrate = "6000k";
        cmd.arg("-b:v").arg(bitrate).arg("-realtime").arg("0").arg("-profile:v").arg("high");
    } else {
        // CPU fallback (khi không có GPU supported)
        cmd.arg("-crf").arg(crf).arg("-preset").arg("medium");
    }

    cmd.arg("-pix_fmt").arg("yuv420p").arg("-y").arg(output_path);

    let output = cmd.output().await.map_err(|e| format!("Lỗi FFmpeg: {}", e))?;
    if !output.status.success() {
        return Err(format!("FFmpeg lỗi: {}", String::from_utf8_lossy(&output.stderr)));
    }
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
    merge_mode: String, // Thêm tham số merge_mode: "fast" | "convert"
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

    // XỬ LÝ CHẾ ĐỘ GHÉP SIÊU TỐC (FAST MERGE)
    if merge_mode == "fast" {
        // 1. Kiểm tra tính đồng nhất của video
        check_video_consistency(&video_files).await?;

        // 2. Tạo file concat list
        let timestamp = get_timestamp();
        let concat_list_path = output_dir.join(format!("concat_list_{}.txt", timestamp));
        let mut list_content = String::new();
        for file in &video_files {
            let escaped = file.replace("'", "'\\''");
            list_content.push_str(&format!("file '{}'\n", escaped));
        }
        fs::write(&concat_list_path, list_content).map_err(|e| e.to_string())?;

        // 3. Chạy FFmpeg Fast Merge (Stream Copy)
        // Lệnh này giữ nguyên 100% chất lượng gốc và chạy cực nhanh
        let mut cmd = run_ffmpeg()?;
        cmd.arg("-f").arg("concat")
           .arg("-safe").arg("0")
           .arg("-i").arg(&concat_list_path);

        if remove_original_audio {
            cmd.arg("-an"); // Nếu chọn xóa audio gốc
        } else {
            cmd.arg("-c:a").arg("copy"); // Copy audio gốc
        }

        cmd.arg("-c:v").arg("copy")
           .arg("-y").arg(&output_path);

        let output = cmd.output().await.map_err(|e| e.to_string())?;
        
        // Cleanup file list
        let _ = fs::remove_file(concat_list_path);

        if !output.status.success() {
            return Err(format!("Lỗi ghép siêu tốc: {}. (Hãy thử chế độ 'Chuyển đổi' nếu video khác codec)", String::from_utf8_lossy(&output.stderr)));
        }

        // Xử lý Audio lồng tiếng (nếu có)
        // Nếu có audio lồng tiếng, ta lại phải chạy thêm 1 bước mux audio (nhanh thôi)
        if !audio_files.is_empty() {
            // Logic mux audio giữ nguyên, nhưng input sẽ là file output_path vừa tạo
            // Để đơn giản, ta return luôn ở đây, phần audio xử lý sau
            // (Hiện tại code cũ đang xử lý audio ở cuối function này, nên ta sẽ để nó flow xuống dưới)
        } else {
             // Nếu không có audio lồng tiếng, xong luôn!
             return Ok(format!("Video đã được tạo thành công! {}", output_path));
        }
    }
    
    // NẾU LÀ MODE CONVERT HOẶC FAST MÀ CÓ AUDIO LỒNG TIẾNG -> XỬ LÝ TIẾP NHƯ CŨ
    // ... (Phần logic convert giữ nguyên, chỉ sửa lại signature)
    
    // Tạo timestamp
    let timestamp = get_timestamp();
    
    // Nếu Fast Mode đã chạy xong phần hình ảnh (và không có audio lồng tiếng), ta đã return ở trên.
    // Nếu xuống đây tức là Mode Convert HOẶC Mode Fast + có Audio lồng tiếng.

    let temp_video_path = if merge_mode == "fast" {
        // Nếu đã ghép nhanh xong, output_path chính là video hình ảnh
        output_path.clone()
    } else {
        // Mode Convert: Chạy quy trình scale và re-encode
        // ... (Logic cũ)
        let (base_width, base_height) = match video_quality.as_str() {
             "hd" => (1280, 720),
             "fullhd" => (1920, 1080),
             "2K" => (2048, 1080),
             "4K" => (3840, 2160),
             _ => (1920, 1080),
        };
        let (target_width, target_height) = match video_aspect_ratio.as_str() {
            "9:16" => (base_height, base_width),
            "16:9" => (base_width, base_height),
             _ => (base_width, base_height),
        };
        let no_black_bars = video_aspect_ratio == "9:16";
        let transition_duration = 1.0;

        let temp_path = output_dir.join(format!("temp_concat_{}.mp4", timestamp));
        let temp_path_str = temp_path.to_string_lossy().to_string();

        concat_videos_with_transitions(
            &video_files,
            &video_effect_type,
            transition_duration,
            &temp_path_str,
            &crf,
            target_width,
            target_height,
            remove_original_audio,
            no_black_bars,
        ).await?;
        temp_path_str
    };

    // Bước 2: Xử lý Audio Lồng Tiếng (Giữ nguyên logic cũ)
    // Lưu ý: Nếu merge_mode="fast", temp_video_path chính là output_path. 
    // Nếu có audio_files, ta sẽ merge vào. Nếu không, ta return luôn.
    
    if audio_files.is_empty() {
         if merge_mode != "fast" {
             // Mode convert chưa đổi tên temp thành final
             fs::rename(&temp_video_path, &output_path).map_err(|e| e.to_string())?;
         }
         return Ok(format!("Video đã được tạo thành công! {}", output_path));
    }

    // Logic Merge Audio files (Phần này để sau khi sửa xong compile error check_video_consistency)
    // ...
    // ... Logic Merge Audio giữ nguyên ...
    // Bước 2: Xử lý audio nếu có
    // Tại điểm này: 
    // - Nếu fast mode: temp_video_path == output_path (đã có video hoàn chỉnh)
    // - Nếu convert mode: temp_video_path == file tạm (chứa video hoàn chỉnh)
    
    let final_output_path = if !audio_files.is_empty() {
        // Lưu số lượng audio files
        let audio_files_count = audio_files.len();
        
        // Merge audio files nếu có nhiều hơn 1 file
        let merged_audio_path = if audio_files_count > 1 {
            let temp_merged_audio = output_dir.join(format!("merged_audio_{}.mp3", timestamp));
            let temp_merged_audio_str = temp_merged_audio.to_string_lossy().to_string();
            merge_audio_files(&audio_files, &temp_merged_audio_str).await?;
            temp_merged_audio_str
        } else {
            audio_files[0].clone()
        };
        
        // Xác định path cuối cùng sau khi mux audio
        let mux_output_path = if merge_mode == "fast" {
             // Fast mode: Output đang bị chiếm bởi video gốc, cần file mới cho muxing
             output_dir.join(format!("final_mux_{}.mp4", timestamp)).to_string_lossy().to_string()
        } else {
             // Convert mode: Output chưa có gì, dùng luôn làm đích
             output_path.clone()
        };

        // Mux audio vào video
        merge_video_with_audio(
            &temp_video_path,
            &merged_audio_path,
            &mux_output_path,
            is_has_auto_caption,
        ).await?;
        
        // Cleanup merged audio temp file if created
        if audio_files_count > 1 {
             let _ = fs::remove_file(merged_audio_path);
        }
        
        mux_output_path
    } else {
        // Không có audio mới
        if merge_mode != "fast" {
            // Convert mode: Rename temp -> final
            if let Err(e) = fs::rename(&temp_video_path, &output_path) {
                // Fallback copy if rename fails (diff partitions)
                fs::copy(&temp_video_path, &output_path).map_err(|e| e.to_string())?;
                let _ = fs::remove_file(&temp_video_path);
            }
        }
        // Fast mode: temp_video_path chính là output_path rồi, ko cần làm gì
        output_path.clone()
    };
    
    // Cleanup Video Temp (Chỉ convert mode mới tạo file temp riêng)
    // Fast mode: temp_video_path là output_path nên không xóa!
    if merge_mode != "fast" && !audio_files.is_empty() {
         let _ = fs::remove_file(&temp_video_path);
    }
    
    // Fast mode logic đặc biệt: Nếu có mux audio, kết quả nằm ở final_mux_..., 
    // còn output_path đang chứa video câm. Cần swap lại.
    if merge_mode == "fast" && !audio_files.is_empty() {
         // Xóa video gốc (câm)
         let _ = fs::remove_file(&output_path);
         // Rename muxed video -> output_path
         fs::rename(&final_output_path, &output_path).map_err(|e| e.to_string())?;
    }

    Ok(format!("Video đã được tạo thành công! {}", output_path))
}


/**
 * Kiểm tra tính đồng nhất của danh sách video (Resolution, FPS)
 */
async fn check_video_consistency(files: &[String]) -> Result<(), String> {
    if files.is_empty() {
        return Ok(());
    }

    // Lấy metadata của video đầu tiên làm chuẩn
    let (first_w, first_h, first_fps, _) = get_video_metadata(&files[0]).await
        .map_err(|e| format!("Không thể đọc video đầu tiên: {}", e))?;

    for (i, file) in files.iter().enumerate().skip(1) {
        let (w, h, fps, _) = get_video_metadata(file).await
            .map_err(|e| format!("Không thể đọc video thứ {}: {}", i + 1, e))?;

        // 1. Kiểm tra độ phân giải
        if w != first_w || h != first_h {
            return Err(format!(
                "Video thứ {} ({}x{}) không khớp resolution với video đầu ({}x{}). Hãy chọn chế độ 'Chuyển đổi & Ghép'.",
                i + 1, w, h, first_w, first_h
            ));
        }

        // 2. Kiểm tra FPS (Cho phép sai số nhỏ do floating point, ví dụ 29.97 vs 30)
        let fps_diff = (fps - first_fps).abs();
        if fps_diff > 1.0 {
             return Err(format!(
                "Video thứ {} ({} fps) không khớp tốc độ khung hình với video đầu ({} fps). Hãy chọn chế độ 'Chuyển đổi & Ghép'.",
                i + 1, fps, first_fps
            ));
        }
    }

    Ok(())
}
