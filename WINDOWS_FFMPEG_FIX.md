# Hướng dẫn sửa lỗi FFmpeg trên Windows

## Vấn đề 1: FFmpeg không được tìm thấy
Khi build và cài đặt ứng dụng trên Windows, ffmpeg không được tìm thấy vì file không có phần mở rộng `.exe`.

## Vấn đề 2: OS Error 216 - Binary không tương thích  
```
Lồi: Lỗi khi tạo segment 1: Lồi khi chạy ffmpeg cho segment 1: This version of %1 is not compatible with the version of Windows you're running. Check your computer's syster information and then contact the software publisher. (os error 216)
```

**Nguyên nhân:**
- FFmpeg binary không tương thích với Windows 10 architecture
- Binary có thể là 32-bit chạy trên hệ thống 64-bit hoặc ngược lại
- Thiếu Visual C++ Redistributables
- Binary bị corrupt hoặc version không phù hợp

## Giải pháp cho Vấn đề 1

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

## Giải pháp cho Vấn đề 2 (OS Error 216)

### Nguyên nhân và Cách khắc phục:

#### Option 1: Download FFmpeg Binary chính thức (Khuyến nghị)
1. **Tải FFmpeg Windows Binary:**
   - Truy cập: https://www.gyan.dev/ffmpeg/builds/
   - Tải "essentials" build (static, no dependencies)
   - Extract và copy `ffmpeg.exe`, `ffprobe.exe` vào `src-tauri/resources/`

2. **Cập nhật CI Workflow:**
```yaml
- name: Download FFmpeg for Windows
  run: |
    $url = "https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip"
    Invoke-WebRequest -Uri $url -OutFile "ffmpeg.zip"
    Expand-Archive "ffmpeg.zip" "ffmpeg-temp"
    $dir = Get-ChildItem "ffmpeg-temp" -Directory | Select -First 1
    Copy-Item "$($dir.FullName)\bin\ffmpeg.exe" "src-tauri\resources\"
    Copy-Item "$($dir.FullName)\bin\ffprobe.exe" "src-tauri\resources\"
```

#### Option 2: Kiểm tra System Requirements
- **Cài Visual C++ Redistributables:** https://aka.ms/vs/17/release/vc_redist.x64.exe
- **Đảm bảo Windows 10/11 64-bit**
- **Kiểm tra architecture compatibility**

#### Option 3: Static Build
Sử dụng static build của FFmpeg để tránh dependency issues.

### Validation Code
Thêm validation trong Rust:

```rust
#[cfg(target_os = "windows")]
fn validate_ffmpeg_compatibility(ffmpeg_path: &str) -> Result<(), String> {
    let output = std::process::Command::new(ffmpeg_path)
        .arg("-version")
        .output();
    
    match output {
        Ok(result) => {
            if result.status.success() {
                Ok(())
            } else {
                Err("FFmpeg binary không tương thích với Windows version này".to_string())
            }
        }
        Err(e) => {
            if e.raw_os_error() == Some(216) {
                Err("Binary architecture không tương thích. Cần FFmpeg 64-bit cho Windows 64-bit".to_string())
            } else {
                Err(format!("Không thể chạy FFmpeg: {}", e))
            }
        }
    }
}
```

## Kiểm tra sau khi cài đặt
Nếu vẫn gặp lỗi, có thể kiểm tra:
1. File ffmpeg.exe có trong thư mục cài đặt không
2. Quyền thực thi của file ffmpeg.exe  
3. Thử chạy ffmpeg.exe trực tiếp từ Command Prompt:
   ```cmd
   cd "C:\Program Files\YTBFlow"
   .\ffmpeg.exe -version
   ```
4. Kiểm tra Windows architecture: `systeminfo | findstr "System Type"`
5. Cài Visual C++ Redistributables nếu cần thiết

## Auto-Download Solution ✅

### Từ phiên bản mới:
**Ứng dụng sẽ tự động:**
1. **Detect hệ điều hành** (Windows 10/11, macOS, Linux)  
2. **Detect architecture** (32-bit/64-bit, Intel/ARM)
3. **Tải FFmpeg tương thích** từ nguồn chính thức
4. **Validate compatibility** trước khi sử dụng
5. **Tự động retry** nếu có lỗi

### Tính năng Auto-Recovery:
- **Tự động detect OS errors** (216, 2, etc.)
- **Smart error messages** với hướng dẫn cụ thể
- **Fallback mechanisms** cho các trường hợp edge case
- **User-friendly instructions** không cần kiến thức kỹ thuật

### Build Process:
```bash
# Auto-detect và download FFmpeg trước khi build
npm run auto-download:ffmpeg

# Build với FFmpeg đã được tối ưu cho từng platform  
npm run tauri:build:windows
```

## System Requirements
- **Windows 10/11 (tự động detect 32/64-bit)**
- **Visual C++ Redistributables 2015-2022** (tự động thông báo nếu thiếu)
- **Minimum 4GB RAM**
- **1GB free disk space** 
- **Internet connection** (lần đầu cài đặt để tải FFmpeg)