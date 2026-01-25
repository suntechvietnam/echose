use std::process::{Command, Stdio};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::io::{BufRead, BufReader};
use once_cell::sync::Lazy;

// Global server process
static SERVER_PROCESS: Lazy<Arc<Mutex<Option<std::process::Child>>>> = 
    Lazy::new(|| Arc::new(Mutex::new(None)));

/// Khởi động Coqui TTS Server (chạy 1 lần duy nhất)
fn ensure_server_running() -> Result<(), String> {
    let mut server = SERVER_PROCESS.lock().unwrap();
    
    // Kiểm tra server đã chạy chưa
    if let Some(ref mut child) = *server {
        // Thử ping server
        if let Ok(response) = ureq::get("http://127.0.0.1:5555/health").call() {
            if response.status() == 200 {
                return Ok(());
            }
        }
        // Server chết, kill và restart
        let _ = child.kill();
        *server = None;
    }
    
    // Khởi động server mới
    println!("🚀 Đang khởi động Coqui TTS Server...");
    
    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let python_path = current_dir
        .join("python_libs")
        .join("coqui-tts")
        .join("venv")
        .join("bin")
        .join("python");
    
    let server_script = current_dir.join("coqui_server.py");
    
    let mut child = Command::new(&python_path)
        .env("COQUI_TOS_AGREED", "1")
        .env("MECABRC", current_dir.join("mecabrc"))
        .arg(&server_script)
        .arg("--port")
        .arg("5555")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Lỗi khởi động server: {}", e))?;
    
    // Pipe logs to terminal
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    std::thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            if let Ok(l) = line { println!("[TTS-Server] {}", l); }
        }
    });
    std::thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            if let Ok(l) = line { eprintln!("[TTS-Server-ERR] {}", l); }
        }
    });
    
    *server = Some(child);
    
    // Đợi server sẵn sàng (tối đa 90 giây)
    println!("⏳ Đợi server load model...");
    for i in 0..90 {
        std::thread::sleep(std::time::Duration::from_secs(1));
        if let Ok(response) = ureq::get("http://127.0.0.1:5555/health").call() {
            if response.status() == 200 {
                println!("✅ Server sẵn sàng sau {} giây!", i + 1);
                return Ok(());
            }
        }
    }
    
    Err("Server không khởi động được sau 90 giây".to_string())
}

#[tauri::command]
pub async fn clone_voice_metavoice(
    text: String,
    reference_audio_path: String,
    output_path: String,
    lang: String,
    speed: f32,
) -> Result<String, String> {
    println!("🎙️ Clone giọng với Coqui TTS Server (Speed: {})", speed);
    println!("📝 Text: {} chars", text.len());
    
    // Đảm bảo server đang chạy
    ensure_server_running()?;
    
    // Gửi request tới server với Timeout 10 phút
    let agent = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(600))
        .build();

    let request_body = serde_json::json!({
        "text": text,
        "reference_audio": reference_audio_path,
        "output_path": output_path,
        "lang": lang,
        "speed": speed
    });
    
    let response = agent.post("http://127.0.0.1:5555/tts")
        .send_json(&request_body)
        .map_err(|e| format!("Lỗi gọi server: {}", e))?;
    
    if response.status() == 200 {
        let result: serde_json::Value = response.into_json()
            .map_err(|e| format!("Lỗi parse response: {}", e))?;
        
        if let Some(output) = result.get("output_path").and_then(|v| v.as_str()) {
            println!("✅ Server TTS hoàn tất: {}", output);

            // POST-PROCESSING SPEED CHANGE (FFMPEG)
            // Nếu tốc độ != 1.0, dùng ffmpeg để đổi tốc độ (atempo)
            if (speed - 1.0).abs() > 0.01 {
                println!("⚡ Áp dụng Filter tốc độ: {}x", speed);
                let speed_out = format!("{}_speed.wav", output);
                
                // Sử dụng utility run_ffmpeg_sync để tìm đúng path ffmpeg
                let status = crate::utils::find_ffmpeg_by_os::run_ffmpeg_sync()
                    .map_err(|e| format!("Không tìm thấy FFmpeg: {}", e))?
                    .arg("-y")
                    .arg("-i")
                    .arg(output)
                    .arg("-filter:a")
                    .arg(format!("atempo={}", speed))
                    .arg(&speed_out)
                    .status()
                    .map_err(|e| format!("Lỗi chạy ffmpeg speed: {}", e))?;

                if status.success() {
                    // Replace original with speed version
                    std::fs::rename(&speed_out, output).map_err(|e| format!("Lỗi rename speed file: {}", e))?;
                    println!("✅ Đã thay đổi tốc độ thành công!");
                } else {
                    println!("⚠️ Cảnh báo: Không thể thay đổi tốc độ (ffmpeg failed). Dùng file gốc.");
                }
            }

            return Ok(output.to_string());
        }
    }
    
    Err("Server trả về lỗi khi render TTS".to_string())
}

#[tauri::command]
pub async fn save_temp_audio(
    audio_data: Vec<u8>,
) -> Result<String, String> {
    let temp_dir = std::env::temp_dir();
    let file_name = format!("echose_rec_{}.wav", uuid::Uuid::new_v4());
    let file_path = temp_dir.join(file_name);
    
    std::fs::write(&file_path, audio_data)
        .map_err(|e| format!("Lỗi lưu file thu âm: {}", e))?;
        
    Ok(file_path.to_string_lossy().to_string())
}

/// Tắt server khi app đóng
pub fn shutdown_server() {
    let mut server = SERVER_PROCESS.lock().unwrap();
    if let Some(ref mut child) = *server {
        println!("🛑 Đang tắt Coqui TTS Server...");
        let _ = child.kill();
        *server = None;
    }
}
