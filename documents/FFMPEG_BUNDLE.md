# FFmpeg Bundle vào Windows Installer

## Tổng quan

Ứng dụng YTBFlow tự động bundle FFmpeg và FFprobe vào Windows installer, **người dùng không cần cài đặt FFmpeg trước**.

## Cách hoạt động

### 1. Trong quá trình build (GitHub Actions)

Workflow tự động:
- Tải FFmpeg từ GitHub releases (BtbN/FFmpeg-Builds)
- Giải nén và copy `ffmpeg.exe` và `ffprobe.exe` vào `src-tauri/resources/`
- Tauri tự động bundle các file này vào Windows installer

### 2. Khi cài đặt trên máy người dùng

Installer sẽ cài đặt FFmpeg vào:
- `C:\Program Files\YTBFlow\resources\ffmpeg.exe`
- `C:\Program Files\YTBFlow\resources\ffprobe.exe`

### 3. Khi ứng dụng chạy

Code tự động tìm FFmpeg theo thứ tự ưu tiên:

1. **Trong thư mục resources/** (khi được bundle)
   - `{app_dir}/resources/ffmpeg.exe`
   
2. **Cùng thư mục với .exe** (fallback)
   - `{app_dir}/ffmpeg.exe`

3. **Trong system PATH** (nếu user đã cài FFmpeg)
   - Tìm `ffmpeg` trong PATH

## Cấu hình

### Cargo.toml
```toml
[package.metadata.tauri.bundle]
resources = ["resources/**"]
```

Điều này báo cho Tauri biết bundle tất cả file trong `src-tauri/resources/` vào installer.

### Workflow (GitHub Actions)

Workflow tự động tải FFmpeg cho Windows trong bước `Download FFmpeg for Windows`.

## Kiểm tra

Sau khi build Windows installer:

1. **Giải nén hoặc cài đặt** file `.msi` hoặc `.exe`
2. **Kiểm tra** thư mục cài đặt có file:
   - `resources/ffmpeg.exe`
   - `resources/ffprobe.exe`

## Troubleshooting

### FFmpeg không được bundle

- Kiểm tra file có trong `src-tauri/resources/` trước khi build
- Kiểm tra log build để xem có lỗi download không
- Đảm bảo `Cargo.toml` có cấu hình `resources = ["resources/**"]`

### Ứng dụng không tìm thấy FFmpeg

- Kiểm tra file có trong thư mục cài đặt
- Kiểm tra quyền truy cập file
- Xem log ứng dụng để biết đường dẫn đang tìm

## Lưu ý

- **Kích thước installer**: Bundle FFmpeg sẽ tăng kích thước installer (~50-100MB)
- **License**: FFmpeg được cấp phép GPL, đảm bảo tuân thủ license khi phân phối
- **Version**: Workflow tải version mới nhất từ GitHub releases

