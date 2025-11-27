# Hướng Dẫn Build Windows Installer

Có 3 cách để build Windows installer cho ứng dụng echose:

## Cách 1: Sử dụng GitHub Actions (Khuyến nghị) ⭐

Đây là cách đơn giản nhất và không cần máy Windows.

### Các bước:

1. **Push code lên GitHub** (nếu chưa có):
   ```bash
   git add .
   git commit -m "Add Windows build support"
   git push origin release/window
   ```

2. **Chạy workflow trên GitHub**:
   - Vào repository trên GitHub
   - Chọn tab **Actions**
   - Chọn workflow **Build Windows Installer**
   - Click **Run workflow** → Chọn branch → **Run workflow**

3. **Tải installer**:
   - Sau khi build xong (có dấu ✅ màu xanh), vào tab **Actions**
   - Click vào run vừa chạy (workflow run)
   - Scroll xuống phần **Artifacts** ở cuối trang
   - Bạn sẽ thấy 2 artifacts:
     - `windows-installer-msi` - chứa file `.msi` (MSI installer)
     - `windows-installer-nsis` - chứa file `.exe` (NSIS installer)
   - Click vào artifact bạn muốn tải → Click **Download** (file sẽ được tải về dạng `.zip`)
   - Giải nén file `.zip` để lấy file cài đặt

### File installer sẽ có ở:
- **Trong GitHub Actions Artifacts:**
  - `windows-installer-msi/echose_0.1.0_x64_en-US.msi` (MSI installer - khuyến nghị)
  - `windows-installer-nsis/echose_0.1.0_x64-setup.exe` (NSIS installer)

- **Trong workflow runner (chỉ để tham khảo):**
  - `src-tauri/target/release/bundle/msi/echose_0.1.0_x64_en-US.msi`
  - `src-tauri/target/release/bundle/nsis/echose_0.1.0_x64-setup.exe`

## Cách 2: Build trên máy Windows

Nếu bạn có máy Windows hoặc Windows VM:

### Yêu cầu:
- Node.js 18+
- Rust (cài từ https://rustup.rs/)
- Visual Studio Build Tools (cho Windows)

### Các bước:

1. **Clone repository**:
   ```bash
   git clone <your-repo-url>
   cd ytbflow
   ```

2. **Cài đặt dependencies**:
   ```bash
   npm install
   ```

3. **Build installer**:
   ```bash
   npm run tauri:build:windows
   ```

4. **File installer** sẽ có ở:
   - `src-tauri/target/release/bundle/msi/echose_0.1.0_x64_en-US.msi`
   - `src-tauri/target/release/bundle/nsis/echose_0.1.0_x64-setup.exe`

## Cách 3: Cross-compile từ macOS (Nâng cao)

⚠️ **Không khuyến nghị** vì phức tạp và có thể gặp lỗi.

Nếu muốn thử:

1. **Cài đặt mingw-w64**:
   ```bash
   brew install mingw-w64
   ```

2. **Thêm Windows target cho Rust**:
   ```bash
   rustup target add x86_64-pc-windows-gnu
   ```

3. **Cấu hình Cargo**:
   Tạo file `~/.cargo/config.toml`:
   ```toml
   [target.x86_64-pc-windows-gnu]
   linker = "x86_64-w64-mingw32-gcc"
   ```

4. **Build**:
   ```bash
   npm run build
   cargo build --release --target x86_64-pc-windows-gnu
   ```

⚠️ Lưu ý: Cross-compile có thể gặp vấn đề với native dependencies và không được Tauri khuyến nghị.

## Ký số (Code Signing) - Tùy chọn

Để ký số installer (tránh cảnh báo Windows Defender):

1. Mua code signing certificate
2. Cập nhật `tauri.conf.json`:
   ```json
   "windows": {
     "certificateThumbprint": "YOUR_THUMBPRINT"
   }
   ```

## Kiểm tra Installer

Sau khi build xong, bạn có thể:
- Test installer trên máy Windows
- Kiểm tra file `.msi` hoặc `.exe` trong thư mục `bundle`

## Troubleshooting

### Lỗi: "Cannot find Windows SDK"
- Cài Visual Studio Build Tools với Windows SDK

### Lỗi: "NSIS not found"
- Tauri sẽ tự động cài NSIS khi build

### Build chậm
- Lần đầu build sẽ mất thời gian để compile Rust
- Các lần sau sẽ nhanh hơn nhờ cache

