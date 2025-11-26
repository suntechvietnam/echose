#!/bin/bash
# Download Whisper models for development

echo "🎤 Downloading Whisper models for YTBFlow development..."

MODELS_DIR="src-tauri/models"
cd "$(dirname "$0")"

# Ensure we're in the right directory
if [ ! -d "$MODELS_DIR" ]; then
    echo "❌ Error: $MODELS_DIR not found. Please run this script from the project root."
    exit 1
fi

# Download base model (recommended for most users)
echo "📥 Downloading ggml-base.bin (~142 MB)..."
if [ -f "$MODELS_DIR/ggml-base.bin" ]; then
    echo "✅ ggml-base.bin already exists"
else
    curl -L -o "$MODELS_DIR/ggml-base.bin" https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin
    if [ $? -eq 0 ]; then
        echo "✅ ggml-base.bin downloaded successfully"
    else
        echo "❌ Failed to download ggml-base.bin"
        exit 1
    fi
fi

# Verify file size
BASE_SIZE=$(stat -f%z "$MODELS_DIR/ggml-base.bin" 2>/dev/null || stat -c%s "$MODELS_DIR/ggml-base.bin" 2>/dev/null)
if [ "$BASE_SIZE" -gt 140000000 ]; then
    echo "✅ Model file size verification passed ($(echo "scale=1; $BASE_SIZE/1024/1024" | bc) MB)"
else
    echo "⚠️  Warning: Model file seems smaller than expected ($BASE_SIZE bytes)"
fi

echo ""
echo "🎉 Model setup complete! You can now run audio-to-text conversion."