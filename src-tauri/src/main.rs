// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod filesystem;
mod image_to_video;
mod process;

use process::ProcessStore;
use std::sync::Arc;
use std::sync::Mutex;
use std::collections::HashMap;

fn main() {
    let processes: ProcessStore = Arc::new(Mutex::new(HashMap::new()));
    
    // Chỉ giữ lại chức năng "Tạo video từ ảnh"
    tauri::Builder::default()
        .manage(processes)
        .invoke_handler(tauri::generate_handler![
            // Mở thư mục bằng file explorer mặc định
            filesystem::open_folder,
            // Image to video - chức năng duy nhất được giữ lại
            image_to_video::create_video_from_images,
            image_to_video::stop_image_video_creation,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
