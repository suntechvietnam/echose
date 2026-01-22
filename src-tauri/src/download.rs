use std::process::{Child, Command, Stdio};
use std::io::{BufRead, BufReader};
use tauri::{Emitter, Window, State};
use std::path::PathBuf;
use std::thread;
use std::sync::{Arc, Mutex};

pub struct DownloadState {
    pub child_process: Arc<Mutex<Option<Child>>>,
}

#[tauri::command]
pub async fn stop_download(state: State<'_, DownloadState>) -> Result<String, String> {
    let mut lock = state.child_process.lock().unwrap();
    if let Some(mut child) = lock.take() {
        match child.kill() {
            Ok(_) => {
                println!("Đã gửi lệnh dừng tiến trình.");
                Ok("Đã dừng tải xuống thành công.".to_string())
            },
            Err(e) => Err(format!("Không thể dừng tiến trình: {}", e)),
        }
    } else {
        Ok("Không có tiến trình nào đang chạy.".to_string())
    }
}

#[tauri::command]
pub async fn download_video(
    url: String,
    mode: String, 
    quality: Option<String>, 
    no_watermark: Option<bool>,
    output_dir: String,
    window: Window,
    state: State<'_, DownloadState>,
) -> Result<String, String> {
    println!("Bắt đầu download: {} (chế độ: {}, chất lượng: {:?})", url, mode, quality);
    
    let output_path = PathBuf::from(&output_dir);
    if !output_path.exists() {
        std::fs::create_dir_all(&output_path).map_err(|e| e.to_string())?;
    }

    let output_template = if mode == "list" {
        format!("{}/%(uploader)s/%(title)s.%(ext)s", output_dir)
    } else {
        format!("{}/%(title)s.%(ext)s", output_dir)
    };

    let mut cmd = Command::new("yt-dlp");
    
    cmd.arg("-o").arg(output_template)
       .arg("--no-mtime")
       .arg("--newline")
       .arg("--progress")
       .arg("--ignore-errors")
       .arg("--no-check-certificates")
       .arg("--retries").arg("10")
       .arg("--fragment-retries").arg("10")
       .arg("--format-sort").arg("res,vbr,abr,size");

    // LUÔN SỬ DỤNG COOKIES TỪ CHROME CHO CÁC TRANG LỚN (YOUTUBE, FB, TIKTOK, IG)
    // Để tránh lỗi "Sign in to confirm you're not a bot"
    cmd.arg("--cookies-from-browser").arg("chrome");

    if url.contains("tiktok.com") || url.contains("facebook.com") || url.contains("fb.watch") || 
       url.contains("instagram.com") || url.contains("x.com") || url.contains("twitter.com") ||
       url.contains("youtube.com") || url.contains("youtu.be") {
        
        cmd.arg("--sleep-requests").arg("1.5");     
        cmd.arg("--sleep-interval").arg("1.5");     
        
        cmd.arg("--user-agent").arg("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36");
        
        if url.contains("tiktok.com") {
            cmd.arg("--add-header").arg("Referer:https://www.tiktok.com/");
        } else if url.contains("youtube.com") || url.contains("youtu.be") {
            cmd.arg("--add-header").arg("Referer:https://www.youtube.com/");
            cmd.arg("--mark-watched"); // Giả lập người dùng thật đang xem
        } else if url.contains("facebook.com") || url.contains("fb.watch") {
            cmd.arg("--add-header").arg("Referer:https://www.facebook.com/");
        }
        
        cmd.arg("--no-warnings"); 
    }

    // CHẤT LƯỢNG CHO CẢ TIKTOK & YOUTUBE
    let format_str = match quality.as_deref() {
        Some("4k") => "bestvideo[height<=2160]+bestaudio/best[height<=2160]",
        Some("2k") => "bestvideo[height<=1440]+bestaudio/best[height<=1440]",
        Some("1080p") => "bestvideo[height<=1080]+bestaudio/best[height<=1080]",
        Some("720p") => "bestvideo[height<=720]+bestaudio/best[height<=720]",
        _ => "bestvideo+bestaudio/best", 
    };
    
    cmd.arg("-f").arg(format_str);
    cmd.arg("--merge-output-format").arg("mp4");

    if mode == "list" {
        cmd.arg("--yes-playlist");
    } else {
        cmd.arg("--no-playlist");
    }

    cmd.arg(url);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| format!("Lỗi khởi chạy yt-dlp: {}. Hãy chắc chắn đã cài đặt yt-dlp.", e))?;
    
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    
    {
        let mut lock = state.child_process.lock().unwrap();
        *lock = Some(child);
    }

    let window_clone = window.clone();
    let thread_stdout = thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            if let Ok(l) = line {
                window_clone.emit("download-log", l).unwrap_or(());
            }
        }
    });

    let window_clone_err = window.clone();
    let thread_stderr = thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            if let Ok(l) = line {
                // Lọc bỏ các dòng cảnh báo không quan trọng để người dùng đỡ rối
                if l.contains("ERROR") || l.contains("Unable to extract") {
                    window_clone_err.emit("download-log", format!("❌ {}", l)).unwrap_or(());
                } else if l.contains("WARNING") {
                    // Chỉ hiện cảnh báo nếu nó thực sự quan trọng
                    if !l.contains("impersonation") {
                        window_clone_err.emit("download-log", format!("⚠️ {}", l)).unwrap_or(());
                    }
                }
            }
        }
    });

    let mut success = false;
    loop {
        thread::sleep(std::time::Duration::from_millis(500));
        let mut lock = state.child_process.lock().unwrap();
        if let Some(ref mut c) = *lock {
            match c.try_wait() {
                Ok(Some(status)) => {
                    success = status.success();
                    break;
                },
                Ok(None) => {}, 
                Err(_) => break,
            }
        } else {
            break;
        }
    }
    
    let _ = thread_stdout.join();
    let _ = thread_stderr.join();

    {
        let mut lock = state.child_process.lock().unwrap();
        *lock = None;
    }

    if success {
        Ok("Download hoàn tất!".to_string())
    } else {
        Ok("Quá trình kết thúc. Nếu file không xuất hiện, hãy kiểm tra link hoặc quyền truy cập thư mục.".to_string())
    }
}
