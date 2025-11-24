# Hướng dẫn sửa lỗi FFmpeg trên Windows

## Vấn đề
Khi build và cài đặt ứng dụng trên Windows, ffmpeg không được tìm thấy vì file không có phần mở rộng `.exe`.

## Giải pháp

### Bước 1: Copy files với extension phù hợp cho Windows
Mở Terminal/Command Prompt và chạy lệnh sau từ thư mục gốc của dự án:

**Trên macOS/Linux (để build cho Windows):**
```bash
cd src-tauri/resources
cp ffmpeg ffmpeg.exe
cp ffprobe ffprobe.exe
```

**Trên Windows:**
```cmd
cd src-tauri\resources
copy ffmpeg ffmpeg.exe
copy ffprobe ffprobe.exe
```

### Bước 2: Build lại ứng dụng
```bash
npm run tauri build
```

### Bước 3: Cài đặt và test
1. Cài đặt file MSI mới
2. Thử tạo video từ ảnh
3. Nếu vẫn lỗi, kiểm tra xem file ffmpeg.exe có được copy vào thư mục cài đặt không

## Các thay đổi đã thực hiện

1. **tauri.conf.json**: Đã cập nhật `resources` để bundle tất cả files trong thư mục resources
2. **image_to_video.rs**: 
   - Cải thiện hàm `find_ffmpeg()` để tìm ffmpeg.exe trên Windows
   - Sửa thông báo lỗi cho phù hợp với từng platform
3. **Cấu trúc file**: Thêm file ffmpeg.exe và ffprobe.exe cho Windows

## Kiểm tra sau khi cài đặt
Nếu vẫn gặp lỗi, có thể kiểm tra:
1. File ffmpeg.exe có trong thư mục cài đặt không
2. Quyền thực thi của file ffmpeg.exe
3. Thử chạy ffmpeg.exe trực tiếp từ Command Prompt