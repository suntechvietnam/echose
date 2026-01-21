#!/usr/bin/env bash

set -euo pipefail

# Thư mục chứa ffmpeg cho mac
MAC_RESOURCE_DIR="src-tauri/resources/mac"
mkdir -p "$MAC_RESOURCE_DIR"

echo "==> Downloading static FFmpeg and FFprobe for macOS..."

# Download FFmpeg (Static build from osxexperts.net - universal/static)
# Hoặc dùng evermeet.cx (rất uy tín)
# Ở đây dùng link direct từ evermeet.cx cho bản release mới nhất
# FFmpeg
curl -L -o ffmpeg.zip https://evermeet.cx/ffmpeg/ffmpeg-8.0.zip
unzip -o ffmpeg.zip -d "$MAC_RESOURCE_DIR"
rm ffmpeg.zip

# FFprobe
curl -L -o ffprobe.zip https://evermeet.cx/ffmpeg/ffprobe-8.0.zip
unzip -o ffprobe.zip -d "$MAC_RESOURCE_DIR"
rm ffprobe.zip

# Chỉnh quyền thực thi
chmod +x "$MAC_RESOURCE_DIR/ffmpeg"
chmod +x "$MAC_RESOURCE_DIR/ffprobe"

# Remove quarantine attribute (tránh lỗi macOS blocking)
xattr -d com.apple.quarantine "$MAC_RESOURCE_DIR/ffmpeg" 2>/dev/null || true
xattr -d com.apple.quarantine "$MAC_RESOURCE_DIR/ffprobe" 2>/dev/null || true

echo "==> Done! Static FFmpeg and FFprobe are ready in $MAC_RESOURCE_DIR"
ls -lh "$MAC_RESOURCE_DIR"
