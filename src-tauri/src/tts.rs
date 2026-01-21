use serde::Serialize;
use std::fs;
use tokio::process::Command;
use uuid::Uuid;

#[derive(Serialize)]
struct OpenAITTSRequest {
    model: String,
    input: String,
    voice: String,
}

#[tauri::command]
pub async fn generate_tts(
    provider: String,
    api_key: String,
    text: String,
    voice: String,
    pitch: String,
    rate: String,
    volume: String,
    bass: i32,
    treble: i32,
    output_folder: Option<String>,
) -> Result<String, String> {
    // 1. Path Setup
    let filename = format!("tts_{}_{}.mp3", voice, Uuid::new_v4());
    let (final_path, temp_dir) = if let Some(folder) = output_folder {
        let path = std::path::PathBuf::from(&folder);
        if !path.exists() || !path.is_dir() { return Err("Thư mục không hợp lệ".into()); }
        (path.join(&filename), path)
    } else {
        let home = dirs::home_dir().unwrap_or_else(|| std::env::temp_dir());
        let t = home.join("echose_temp");
        if !t.exists() { let _ = fs::create_dir_all(&t).map_err(|e| e.to_string())?; }
        (t.join(&filename), t)
    };
    
    let final_path_str = final_path.to_string_lossy().to_string();
    let raw_path = final_path.with_extension("raw.mp3");
    let raw_path_str = raw_path.to_string_lossy().to_string();

    // 2. Generation logic
    if provider == "openai" && !api_key.is_empty() {
        println!("Đang tạo giọng nói bằng OpenAI...");
        let client = reqwest::Client::new();
        let body = OpenAITTSRequest {
            model: "tts-1".to_string(),
            input: text.clone(),
            voice: voice.clone(), 
        };
        
        let res = client.post("https://api.openai.com/v1/audio/speech")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Lỗi kết nối OpenAI: {}", e))?;
            
        if !res.status().is_success() {
            let err_text = res.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("Lỗi OpenAI API: {}", err_text));
        }
        
        let bytes = res.bytes().await.map_err(|e| e.to_string())?;
        fs::write(&raw_path, bytes).map_err(|e| e.to_string())?;
    } else {
        // Fallback to Edge-TTS
        println!("Đang tạo giọng nói bằng Edge-TTS...");
        
        let possible_tts_paths = [
            "edge-tts",
            "/Users/kiennguyentien/Library/Python/3.9/bin/edge-tts",
            "/usr/local/bin/edge-tts",
        ];
        
        let mut tts_cmd = None;
        for path in possible_tts_paths {
            if Command::new(path).arg("--version").output().await.is_ok() {
                tts_cmd = Some(Command::new(path));
                break;
            }
        }
        
        let mut tts_cmd = tts_cmd.ok_or_else(|| "Không tìm thấy lệnh 'edge-tts'. Vui lòng cài đặt bằng pip.".to_string())?;
        
        if text.trim().starts_with("<speak") {
           let ssml_path = temp_dir.join(format!("ssml_{}.xml", Uuid::new_v4()));
           fs::write(&ssml_path, &text).map_err(|e| e.to_string())?;
           tts_cmd.arg("--file").arg(&ssml_path).arg("--write-media").arg(&raw_path_str);
           let out = tts_cmd.output().await.map_err(|e| e.to_string())?;
           let _ = fs::remove_file(&ssml_path);
           if !out.status.success() { return Err(String::from_utf8_lossy(&out.stderr).into()); }
        } else {
           tts_cmd.arg("--voice").arg(&voice)
                  .arg("--text").arg(&text)
                  .arg("--write-media").arg(&raw_path_str)
                  .arg(format!("--pitch={}", pitch))
                  .arg(format!("--rate={}", rate))
                  .arg(format!("--volume={}", volume));
                  
           let out = tts_cmd.output().await.map_err(|e| e.to_string())?;
           if !out.status.success() { return Err(String::from_utf8_lossy(&out.stderr).into()); }
        }
    }

    // 3. Mastering logic (FFmpeg)
    let filter = format!(
        "bass=g={},treble=g={},compand=attacks=0:points=-30/-30|-20/-15|0/-10,aexciter=level_in=10:level_out=10:amount=5",
        bass, treble
    );
    
    let possible_ffmpeg_paths = [
        "ffmpeg",
        "/opt/homebrew/bin/ffmpeg",
        "/usr/local/bin/ffmpeg",
    ];
    
    let mut ffmpeg_cmd = None;
    for path in possible_ffmpeg_paths {
        if Command::new(path).arg("-version").output().await.is_ok() {
            ffmpeg_cmd = Some(Command::new(path));
            break;
        }
    }
    
    let mut ffmpeg_cmd = ffmpeg_cmd.ok_or_else(|| "Không tìm thấy lệnh 'ffmpeg'. Vui lòng cài đặt brew install ffmpeg.".to_string())?;
    
    ffmpeg_cmd.arg("-y")
              .arg("-i").arg(&raw_path_str)
              .arg("-af").arg(filter)
              .arg(&final_path_str);

    let out = ffmpeg_cmd.output().await.map_err(|e| e.to_string())?;
    
    if !out.status.success() { 
        println!("Lỗi FFmpeg Mastering, giữ nguyên file gốc.");
        fs::rename(&raw_path, &final_path).map_err(|e| e.to_string())?; 
    } else { 
        let _ = fs::remove_file(&raw_path); 
    }

    println!("TTS Thành công: {}", final_path_str);
    Ok(final_path_str)
}
