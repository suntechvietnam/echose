use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::process::Child;

// Cho phép lưu nhiều process cho một task ID (ví dụ: tạo nhiều segment song song)
pub type ProcessStore = Arc<std::sync::Mutex<HashMap<String, Vec<Arc<tokio::sync::Mutex<Option<Child>>>>>>>;
