use crate::process::ProcessStore;
use crate::utils::find_ffmpeg_by_os::run_ffmpeg;
use std::sync::{Arc};
use tokio::sync::Mutex;
use std::path::Path;

/// Trích xuất ảnh từ video theo chu kỳ giây
#[tauri::command]
pub async fn extract_images_from_video_periodic(
    video_path: String,
    output_dir: String,
    interval: f64, // Số giây mỗi ảnh
    process_id: String,
    processes: tauri::State<'_, ProcessStore>,
) -> Result<String, String> {
    // 1. Kiểm tra đầu vào
    let video_p = Path::new(&video_path);
    if !video_p.exists() {
        return Err(format!("File video không tồn tại: {}", video_path));
    }

    let output_p = Path::new(&output_dir);
    if !output_p.exists() {
        std::fs::create_dir_all(output_p).map_err(|e| format!("Không thể tạo thư mục lưu: {}", e))?;
    }

    let mut cmd = run_ffmpeg()?;
    
    // Sử dụng fps filter để trích xuất ảnh theo chu kỳ
    // fps=1/interval ví dụ interval=5s thì fps=1/5 = 0.2 ảnh/giây
    let filter = format!("fps=1/{}", interval);
    
    cmd.arg("-i").arg(&video_path)
       .arg("-vf").arg(filter)
       .arg("-pix_fmt").arg("rgb24") // Đảm bảo hệ màu chuẩn
       .arg("-compression_level").arg("0") // Nén thấp nhất để giữ dung lượng lớn/chất lượng tối đa (nếu muốn file nặng)
       .arg(format!("{}/frame_%04d.png", output_dir))
       .arg("-y");
    
    println!("Running extract images: {:?}", cmd);
    
    let child = cmd.spawn().map_err(|e| format!("Lỗi khi khởi chạy FFmpeg: {}", e))?;
    let child_arc = Arc::new(Mutex::new(Some(child)));
    
    // Đăng ký process
    {
        let mut procs = processes.lock().unwrap();
        procs.entry(process_id.to_string()).or_insert_with(Vec::new).push(child_arc.clone());
    }
    
    // Đợi process hoàn thành
    let output = {
        let mut child_lock = child_arc.lock().await;
        if let Some(child) = child_lock.take() {
            child.wait_with_output().await.map_err(|e| format!("Lỗi khi đợi ffmpeg: {}", e))?
        } else {
            return Err("Tiến trình đã bị dừng".to_string());
        }
    };
    
    // Cleanup
    {
        let mut procs = processes.lock().unwrap();
        if let Some(vec) = procs.get_mut(&process_id) {
            vec.retain(|p| !Arc::ptr_eq(p, &child_arc));
        }
    }
    
    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("FFmpeg Error: {}", error_msg));
    }
    
    Ok(format!("Trích xuất ảnh thành công vào thư mục: {}", output_dir))
}

#[tauri::command]
pub async fn stop_image_extraction(
    process_id: String,
    processes: tauri::State<'_, ProcessStore>,
) -> Result<String, String> {
    let child_arcs = {
        let mut procs = processes.lock().unwrap();
        procs.remove(&process_id).unwrap_or_default()
    };
    
    for child_arc in child_arcs {
        let mut child_lock = child_arc.lock().await;
        if let Some(mut child) = child_lock.take() {
            let _ = child.kill().await;
            let _ = child.wait().await;
        }
    }
    
    Ok("Đã dừng trích xuất ảnh".to_string())
}
