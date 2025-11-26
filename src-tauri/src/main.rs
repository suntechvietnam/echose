// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod audio_to_text_ass;  // Speech-to-text với whisper-rs for ASS format
mod file_audio;
mod filesystem;
mod image_to_video;
mod process;
mod utils;

use process::ProcessStore;
use std::sync::Arc;
use std::sync::Mutex;
use std::collections::HashMap;

fn main() {
    let processes: ProcessStore = Arc::new(Mutex::new(HashMap::new()));
    
    tauri::Builder::default()
        .manage(processes)
        .invoke_handler(tauri::generate_handler![
            // Mở thư mục bằng file explorer mặc định
            filesystem::open_folder,
            // Image to video
            image_to_video::create_video_from_images,
            image_to_video::stop_image_video_creation,
            // Audio utilities
            file_audio::get_audio_duration,
            // Audio to text (whisper-rs) - ASS format
            audio_to_text_ass::convert_audio_to_ass,
            audio_to_text_ass::segments_to_ass_string,
            audio_to_text_ass::list_whisper_models,
            audio_to_text_ass::save_ass_file,
            audio_to_text_ass::read_ass_file,
            audio_to_text_ass::delete_temp_ass_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
