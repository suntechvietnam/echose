// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod audio;
mod dashboard;
mod dialogs;
mod filesystem;
mod image_to_video;
mod models;
mod process;
mod video;
mod video_concat;
mod youtube;

use process::ProcessStore;
use std::sync::Arc;
use std::sync::Mutex;
use std::collections::HashMap;

fn main() {
    let processes: ProcessStore = Arc::new(Mutex::new(HashMap::new()));
    
    tauri::Builder::default()
        .manage(processes)
        .invoke_handler(tauri::generate_handler![
            dashboard::get_dashboard_stats,
            filesystem::read_directory,
            filesystem::check_file_exists,
            filesystem::get_file_size,
            filesystem::get_home_dir,
            filesystem::open_folder,
            filesystem::get_download_dir,
            filesystem::save_temp_file,
            dialogs::select_files,
            dialogs::select_folder,
            youtube::download_youtube_video,
            video::start_create_video,
            video::wait_for_video_creation,
            video::stop_video_creation,
            video::stop_all_video_creation,
            video::create_video_with_audio_mix,
            video::create_merged_mp3_step1,
            video::create_merged_video_step2,
            video::mix_video_audio_step3,
            video::get_step1_merged_mp3_path,
            video::get_step2_merged_video_path,
            image_to_video::create_video_from_images,
            image_to_video::stop_image_video_creation,
            video_concat::concat_video_segments_with_transitions,
            audio::create_audio_mix,
            audio::wait_for_audio_creation,
            audio::stop_audio_creation,
            audio::get_audio_duration,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
