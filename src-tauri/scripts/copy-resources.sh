#!/bin/bash

# Script to copy FFmpeg resources into app bundle after build
# This runs after Tauri bundles the app

APP_BUNDLE="$1"
RESOURCES_DIR="$2"

if [ -z "$APP_BUNDLE" ] || [ -z "$RESOURCES_DIR" ]; then
    echo "Usage: $0 <app_bundle_path> <resources_source_dir>"
    exit 1
fi

if [ ! -d "$APP_BUNDLE" ]; then
    echo "App bundle not found: $APP_BUNDLE"
    exit 1
fi

if [ ! -d "$RESOURCES_DIR" ]; then
    echo "Resources directory not found: $RESOURCES_DIR"
    exit 1
fi

# Copy resources to app bundle
APP_RESOURCES="$APP_BUNDLE/Contents/Resources"
APP_LIB="$APP_BUNDLE/Contents/Frameworks"
mkdir -p "$APP_RESOURCES"
mkdir -p "$APP_LIB"

# Copy FFmpeg binaries
cp "$RESOURCES_DIR/ffmpeg" "$APP_RESOURCES/ffmpeg"
cp "$RESOURCES_DIR/ffprobe" "$APP_RESOURCES/ffprobe"

chmod +x "$APP_RESOURCES/ffmpeg"
chmod +x "$APP_RESOURCES/ffprobe"

# Copy FFmpeg dependencies (dylibs)
FFMPEG_LIB_DIR="/opt/homebrew/Cellar/ffmpeg/8.0_2/lib"
if [ -d "$FFMPEG_LIB_DIR" ]; then
    echo "Copying FFmpeg dependencies..."
    cp "$FFMPEG_LIB_DIR"/*.dylib "$APP_LIB/" 2>/dev/null || true
    
    # Fix library paths in binaries to use @rpath
    for lib in "$APP_LIB"/*.dylib; do
        if [ -f "$lib" ]; then
            libname=$(basename "$lib")
            # Set library ID to @rpath
            install_name_tool -id "@rpath/$libname" "$lib" 2>/dev/null || true
            
            # Fix all dependencies within this library
            for dep_lib in "$APP_LIB"/*.dylib; do
                if [ -f "$dep_lib" ]; then
                    dep_name=$(basename "$dep_lib")
                    # Replace any reference to FFMPEG_LIB_DIR with @rpath
                    install_name_tool -change "$FFMPEG_LIB_DIR/$dep_name" "@rpath/$dep_name" "$lib" 2>/dev/null || true
                fi
            done
        fi
    done
    
    # Fix FFmpeg to use @rpath for dependencies
    install_name_tool -add_rpath "@executable_path/../Frameworks" "$APP_RESOURCES/ffmpeg" 2>/dev/null || true
    install_name_tool -add_rpath "@executable_path/../Frameworks" "$APP_RESOURCES/ffprobe" 2>/dev/null || true
    
    # Update library paths in FFmpeg binaries
    for lib in "$APP_LIB"/*.dylib; do
        if [ -f "$lib" ]; then
            libname=$(basename "$lib")
            install_name_tool -change "$FFMPEG_LIB_DIR/$libname" "@rpath/$libname" "$APP_RESOURCES/ffmpeg" 2>/dev/null || true
            install_name_tool -change "$FFMPEG_LIB_DIR/$libname" "@rpath/$libname" "$APP_RESOURCES/ffprobe" 2>/dev/null || true
        fi
    done
else
    echo "Warning: FFmpeg lib directory not found, dependencies may not work on other machines"
fi

# Xóa quarantine flag để tránh lỗi "damaged" trên macOS (chỉ trên app bundle, không phải từng file)
xattr -d com.apple.quarantine "$APP_BUNDLE" 2>/dev/null || true

echo "Resources copied successfully to $APP_RESOURCES"
echo "FFmpeg dependencies copied to $APP_LIB"
echo "Quarantine flags removed from app bundle"

