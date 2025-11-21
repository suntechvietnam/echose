use crate::models::DashboardStats;

#[tauri::command]
pub fn get_dashboard_stats() -> DashboardStats {
    DashboardStats {
        total_videos: 42,
        total_views: 125000,
        total_revenue: 12500.50,
        active_campaigns: 5,
    }
}

