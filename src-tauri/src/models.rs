use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardStats {
    pub total_videos: u32,
    pub total_views: u64,
    pub total_revenue: f64,
    pub active_campaigns: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: Option<u64>,
}

