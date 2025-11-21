# YTBFlow Desktop App

Ứng dụng desktop được xây dựng bằng Tauri (Rust + Vue.js) với màn hình dashboard đơn giản.

## Yêu Cầu

- Node.js (v16 trở lên)
- Rust (cài đặt từ https://rustup.rs/)
- Tauri CLI

## Cài Đặt

1. Cài đặt dependencies:
```bash
npm install
```

2. Chạy ứng dụng ở chế độ development:
```bash
npm run tauri:dev
```

Hoặc chạy riêng frontend với Vite:
```bash
npm run dev
```

3. Build ứng dụng để production:
```bash
npm run tauri:build
```

Hoặc build riêng frontend:
```bash
npm run build
```

## Cấu Trúc Dự Án

```
frontend/
├── src/                    # Frontend (Vue.js)
│   ├── components/        # Vue components
│   │   ├── Sidebar.vue
│   │   ├── DownloadPage.vue
│   │   ├── AudioMixPage.vue
│   │   ├── VideoCreatePage.vue
│   │   └── VideoSourcePage.vue
│   ├── composables/       # Vue composables (reusable logic)
│   │   ├── useTauri.js
│   │   └── useFileSelection.js
│   ├── App.vue           # Root component
│   ├── main.js           # Entry point
│   ├── index.html        # HTML template
│   └── styles.css        # Global styles
├── src-tauri/            # Rust backend
│   ├── src/
│   │   ├── main.rs       # Tauri setup
│   │   ├── audio.rs      # Audio mixing
│   │   ├── video.rs      # Video creation
│   │   ├── youtube.rs    # YouTube download
│   │   └── ...
│   ├── Cargo.toml        # Rust dependencies
│   └── tauri.conf.json   # Tauri configuration
├── vite.config.js        # Vite configuration
└── package.json          # Node.js dependencies
```

## Tính Năng

- **Download Video**: Download video từ YouTube
- **Quản Lý File Nhạc Nguồn**: Tạo file nhạc mix từ nhiều file MP3 với xáo trộn thông minh
- **Quản Lý Video Nguồn**: Demo các tính năng file system
- **Tạo Video**: Tạo video từ audio, ảnh và video nguồn

## Tech Stack

- **Frontend**: Vue.js 3 + Vite
- **Backend**: Rust + Tauri
- **Styling**: CSS (có thể migrate sang Tailwind CSS sau)
