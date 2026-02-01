// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod audio_to_text_ass;  // Speech-to-text với whisper-rs for ASS format
mod audio_to_text_txt;  // Speech-to-text với whisper-rs for TXT format
mod file_audio;
mod filesystem;
mod image_to_video;
mod process;
mod utils;
mod video_to_video;
mod video_to_image;
mod tts;
mod tts_metavoice;
mod download;
mod audio_utils;
mod furigana_generator;  // Furigana (Ruby text) generator using MeCab
mod japanese_translator;  // Japanese to Vietnamese translation using Gemini

use process::ProcessStore;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn main() {
    let processes: ProcessStore = Arc::new(Mutex::new(HashMap::new()));
    
    // Khởi tạo state cho download
    let download_state = download::DownloadState {
        child_process: Arc::new(Mutex::new(None)),
    };
    
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_process::init())
        .manage(processes)
        .manage(download_state) // Đăng ký download state
        .invoke_handler(tauri::generate_handler![
            // Mở thư mục bằng file explorer mặc định
            filesystem::open_folder,
            filesystem::get_home_dir,
            // Image to video
            image_to_video::create_video_from_images,
            image_to_video::create_video_from_ai_image,
            image_to_video::stop_image_video_creation,
            // Video to video
            video_to_video::create_video_from_video,
            video_to_video::stop_video_video_creation,
            file_audio::get_audio_duration,
            file_audio::extract_audio_from_video,
            // Video to image
            video_to_image::extract_images_from_video_periodic,
            video_to_image::stop_image_extraction,
            // Audio to text (whisper-rs) - ASS format
            audio_to_text_ass::convert_audio_to_ass,
            audio_to_text_ass::segments_to_ass_string,
            audio_to_text_ass::list_whisper_models,
            audio_to_text_ass::save_ass_file,
            audio_to_text_ass::read_ass_file,
            audio_to_text_ass::delete_temp_ass_file,
            audio_to_text_ass::export_dialogue_ass,
            // Audio to text (whisper-rs) - TXT format
            audio_to_text_txt::convert_audio_to_txt,
            audio_to_text_txt::segments_to_txt_string,
            // AI Voice (TTS)
            tts::generate_tts,
            // Voice Cloning (MetaVoice)
            tts_metavoice::clone_voice_metavoice,
            tts_metavoice::save_temp_audio,
            // Download
            download::download_video,
            download::stop_download, // Đăng ký command dừng download
            // Audio Utils
            audio_utils::merge_audio_files,
            audio_utils::copy_external_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
