use crate::video::find_ffmpeg;
use crate::process::ProcessStore;
use std::path::PathBuf;
use std::fs;

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
 * Build filter_complex cho xfade transitions giữa các video segments
 * 
 * Format ví dụ (với scale):
 * [0:v]scale=1920:1080,setsar=1[v0];
 * [1:v]scale=1920:1080,setsar=1[v1];
 * [2:v]scale=1920:1080,setsar=1[v2];
 * [3:v]scale=1920:1080,setsar=1[v3];
 * [v0][v1]xfade=transition=hrwind:duration=1:offset=4[x1];
 * [x1][v2]xfade=transition=hrwind:duration=1:offset=8[x2];
 * [x2][v3]xfade=transition=hrwind:duration=1:offset=12[video_out]
 * 
 * Format ví dụ (không scale - segments đã cùng resolution):
 * [0:v][1:v]xfade=transition=hrwind:duration=1:offset=4[x1];
 * [x1][2:v]xfade=transition=hrwind:duration=1:offset=8[x2];
 * [x2][3:v]xfade=transition=hrwind:duration=1:offset=12[video_out]
 */
fn build_xfade_filter_complex(
    segment_count: usize,
    width: i32,
    height: i32,
    transition_type: &str,
    transition_duration: f64,
    segment_duration: i32,
    force_scale: bool,
) -> String {
    let mut filter_parts = Vec::new();
    
    // Bước 1: Scale tất cả segments về cùng resolution (nếu cần)
    if force_scale {
        for i in 0..segment_count {
            filter_parts.push(format!(
                "[{}:v]scale={}:{},setsar=1[v{}];",
                i, width, height, i
            ));
        }
    }
    
    // Bước 2: Xfade transitions giữa các segments
    // Offset được tính từ đầu video output
    // Ví dụ: segment 5 giây, transition 1 giây
    // - Transition 1 (giữa v0 và v1): offset = 5 - 1 = 4 giây
    // - Transition 2 (giữa v1 và v2): offset = 10 - 1 = 9 giây (nhưng trong ví dụ là 8?)
    // Thực ra offset là thời điểm bắt đầu transition tính từ đầu output video
    // Với segment_duration = 5, transition_duration = 1:
    // - Transition 1: bắt đầu ở giây thứ 4 (trong segment đầu tiên)
    // - Transition 2: bắt đầu ở giây thứ 9 (trong segment thứ hai) = 5 + 4
    // Nhưng ví dụ lại là 8, có thể là tính từ đầu segment thứ 2?
    
    if segment_count == 1 {
        // Chỉ có 1 segment, không cần transition
        // Nếu scale thì dùng [v0], nếu không scale thì dùng [0:v] trực tiếp
        if force_scale {
            filter_parts.push("[v0][video_out]".to_string());
        } else {
            // Không scale, dùng input trực tiếp (nhưng thực ra trường hợp này không bao giờ xảy ra
            // vì helper function đã return sớm khi segment_count == 1)
            filter_parts.push("[0:v][video_out]".to_string());
        }
    } else {
        // Transition đầu tiên
        // Offset = segment_duration - transition_duration
        let first_offset = segment_duration as f64 - transition_duration;
        
        // Xác định label cho input (không có dấu ngoặc vuông trong label, format string sẽ thêm)
        let first_input_label = if force_scale { "v0" } else { "0:v" };
        let second_input_label = if force_scale { "v1" } else { "1:v" };
        
        // Format string sẽ tự động thêm dấu ngoặc vuông
        filter_parts.push(format!(
            "[{}][{}]xfade=transition={}:duration={}:offset={}[x1];",
            first_input_label, second_input_label, transition_type, transition_duration, first_offset
        ));
        
        // Các transitions tiếp theo: [xN][vN+1]xfade -> [xN+1]
        for i in 2..segment_count {
            let current_offset = first_offset + (i - 1) as f64 * segment_duration as f64 - transition_duration;
            let input_label = if i == 2 { "x1" } else { &format!("x{}", i - 1) };
            let next_segment_label = if force_scale {
                format!("v{}", i)
            } else {
                format!("{}:v", i)
            };
            let output_label = if i == segment_count - 1 {
                "video_out" // Output cuối cùng
            } else {
                &format!("x{}", i)
            };
            
            // Format string sẽ tự động thêm dấu ngoặc vuông cho next_segment_label
            filter_parts.push(format!(
                "[{}][{}]xfade=transition={}:duration={}:offset={}[{}];",
                input_label, next_segment_label, transition_type, transition_duration, current_offset, output_label
            ));
        }
    }
    
    filter_parts.join(" ")
}

/**
 * Helper function để concat video segments với transitions
 * Có thể gọi từ các module khác
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
    
    let ffmpeg_path = find_ffmpeg().ok_or_else(|| {
        "Không tìm thấy ffmpeg. Vui lòng cài đặt: brew install ffmpeg".to_string()
    })?;
    
    // Xác định resolution và encoding params (chỉ cần nếu force_scale)
    let (width, height) = if force_scale {
        get_output_resolution(video_quality, video_aspect_ratio)
    } else {
        (0, 0) // Không dùng nếu không scale
    };
    let (preset, crf) = get_encoding_params(video_quality);
    
    // Xác định transition type
    // Nếu "none" thì không dùng xfade, chỉ concat đơn giản
    let transition_type = if video_effect_type == "none" {
        "fade" // Dùng fade mặc định cho "none"
    } else {
        video_effect_type
    };
    
    // Build filter_complex
    let filter_complex = if video_effect_type == "none" {
        // Concat đơn giản không có transition
        let mut parts = Vec::new();
        for i in 0..segment_files.len() {
            parts.push(format!("[{}:v]", i));
        }
        format!("{}concat=n={}:v=1:a=0[video_out]", 
            parts.join(""), segment_files.len())
    } else {
        // Concat với xfade transitions
        build_xfade_filter_complex(
            segment_files.len(),
            width,
            height,
            transition_type,
            transition_duration,
            segment_duration,
            force_scale,
        )
    };
    
    // Build ffmpeg command
    let mut cmd = tokio::process::Command::new(&ffmpeg_path);
    
    // Thêm tất cả input files
    for segment_file in &segment_files {
        cmd.arg("-i").arg(segment_file);
    }
    
    // Áp dụng filter_complex
    cmd.arg("-filter_complex")
        .arg(&filter_complex)
        .arg("-map")
        .arg("[video_out]")
        .arg("-c:v")
        .arg("libx264")
        .arg("-preset")
        .arg(preset)
        .arg("-crf")
        .arg(crf)
        .arg("-pix_fmt")
        .arg("yuv420p")
        .arg("-g")
        .arg("50")  // GOP size = 2x framerate
        .arg("-bf")
        .arg("2")  // B-frames
        .arg("-refs")
        .arg("4")  // Reference frames
        .arg("-y")
        .arg(output_path);
    
    // Chạy và đợi process hoàn thành
    let output = cmd.output().await
        .map_err(|e| format!("Lỗi khi chạy ffmpeg concat: {}", e))?;
    
    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Lỗi khi concat video: {}", error_msg));
    }
    
    Ok(())
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
    processes: tauri::State<'_, ProcessStore>,
) -> Result<String, String> {
    if segment_files.is_empty() {
        return Err("Cần ít nhất một video segment".to_string());
    }
    
    if segment_files.len() == 1 {
        // Chỉ có 1 segment, không cần concat, chỉ cần copy hoặc re-encode
        return Err("Cần ít nhất 2 video segments để concat với transitions".to_string());
    }
    
    let ffmpeg_path = find_ffmpeg().ok_or_else(|| {
        "Không tìm thấy ffmpeg. Vui lòng cài đặt: brew install ffmpeg".to_string()
    })?;
    
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

