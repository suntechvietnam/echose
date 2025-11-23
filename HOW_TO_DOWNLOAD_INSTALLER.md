# Cách Tải File Cài Đặt Windows

## 📥 Tải từ GitHub Actions (Sau khi build xong)

### Bước 1: Vào GitHub Repository
1. Mở trình duyệt và vào: `https://github.com/suntech-projects/ytbflow`
2. Click vào tab **Actions** (ở trên cùng)

### Bước 2: Tìm Workflow Run đã hoàn thành
1. Tìm workflow run có tên **"Build Windows Installer"**
2. Click vào run có dấu ✅ màu xanh (đã thành công)
3. Nếu chưa thấy, có thể cần đợi vài phút để build xong

### Bước 3: Tải Artifacts
1. Scroll xuống cuối trang, tìm phần **Artifacts**
2. Bạn sẽ thấy 2 artifacts:
   - 📦 **windows-installer-msi** - File `.msi` (MSI installer)
   - 📦 **windows-installer-nsis** - File `.exe` (NSIS installer)

### Bước 4: Download và Giải nén
1. Click vào artifact bạn muốn (khuyến nghị: `windows-installer-msi`)
2. Click nút **Download** (bên phải)
3. File sẽ được tải về dạng `.zip`
4. Giải nén file `.zip` để lấy file cài đặt:
   - `YTBFlow_0.1.0_x64_en-US.msi` (từ windows-installer-msi)
   - `YTBFlow_0.1.0_x64-setup.exe` (từ windows-installer-nsis)

## 📍 Vị trí File trong Workflow

Trong quá trình build trên GitHub Actions, file được tạo ở:
- **MSI:** `src-tauri/target/release/bundle/msi/YTBFlow_0.1.0_x64_en-US.msi`
- **NSIS:** `src-tauri/target/release/bundle/nsis/YTBFlow_0.1.0_x64-setup.exe`

Sau đó được upload lên Artifacts để bạn tải về.

## 🔄 Nếu Build trên Máy Windows

Nếu bạn build trên máy Windows local:
```bash
npm run tauri:build:windows
```

File sẽ có ở:
- `src-tauri/target/release/bundle/msi/YTBFlow_0.1.0_x64_en-US.msi`
- `src-tauri/target/release/bundle/nsis/YTBFlow_0.1.0_x64-setup.exe`

## 💡 Lưu ý

- **Artifacts được lưu 30 ngày** (theo cấu hình `retention-days: 30`)
- **MSI installer** thường được khuyến nghị hơn NSIS
- File `.msi` có thể được cài đặt bằng cách double-click hoặc qua Windows Installer
- File `.exe` là NSIS installer, cũng có thể double-click để cài đặt

## 🎯 Quick Links

- **Repository:** https://github.com/suntech-projects/ytbflow
- **Actions:** https://github.com/suntech-projects/ytbflow/actions
- **Workflow:** https://github.com/suntech-projects/ytbflow/actions/workflows/build-windows.yml

