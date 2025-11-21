#!/bin/bash

# Script để fix lỗi "damaged" trên macOS
# Chạy script này sau khi install app từ DMG

APP_PATH="$1"

if [ -z "$APP_PATH" ]; then
    echo "Usage: $0 <path_to_app>"
    echo "Example: $0 /Applications/YTBFlow.app"
    exit 1
fi

if [ ! -d "$APP_PATH" ]; then
    echo "App not found: $APP_PATH"
    exit 1
fi

echo "Removing quarantine flag from $APP_PATH..."
xattr -cr "$APP_PATH"
xattr -d com.apple.quarantine "$APP_PATH" 2>/dev/null || true

echo "Done! You can now open the app."
echo ""
echo "If you still see the error, try:"
echo "1. Right-click the app and select 'Open'"
echo "2. Or run: sudo xattr -rd com.apple.quarantine $APP_PATH"


