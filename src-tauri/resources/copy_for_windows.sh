#!/bin/bash
# Script để copy ffmpeg files với extension phù hợp cho Windows
cd "$(dirname "$0")"

echo "Current directory: $(pwd)"
echo "Files in directory:"
ls -la

# Copy ffmpeg thành ffmpeg.exe cho Windows
if [ -f "ffmpeg" ]; then
    cp ffmpeg ffmpeg.exe
    echo "✓ Created ffmpeg.exe"
else
    echo "✗ ffmpeg not found"
fi

# Copy ffprobe thành ffprobe.exe cho Windows  
if [ -f "ffprobe" ]; then
    cp ffprobe ffprobe.exe
    echo "✓ Created ffprobe.exe"
else
    echo "✗ ffprobe not found"
fi

echo "Final file list:"
ls -la