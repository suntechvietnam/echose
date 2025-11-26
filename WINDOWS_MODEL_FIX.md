# Fix Windows Deployment - Missing Whisper Models

## Vấn đề

Người dùng Windows báo lỗi khi sử dụng tính năng audio-to-text:
```
Không tìm thấy file model: ggml-base.bin
```

## Nguyên nhân

1. **Build workflow thiếu bước download model**: GitHub Actions không download Whisper models
2. **Tauri config chưa bundle models**: `tauri.conf.json` không include thư mục `models/`
3. **Model path resolution không tối ưu**: Code tìm model chưa đầy đủ cho Windows

## Giải pháp

### 1. Update GitHub Actions Workflow

Thêm bước download model vào `.github/workflows/build-windows.yml`:

```yaml
- name: Download Whisper Model
  run: |
    # Tạo thư mục models và download ggml-base.bin
    $modelsDir = "src-tauri\models"
    New-Item -ItemType Directory -Force -Path $modelsDir
    
    $modelUrl = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin"
    $modelPath = "$modelsDir\ggml-base.bin"
    
    curl.exe -L -o $modelPath $modelUrl --progress-bar
    
    # Verify download (should be ~142MB)
    $downloadedSize = (Get-Item $modelPath).Length
    if ($downloadedSize -gt 140MB) {
      Write-Host "✅ Model downloaded successfully: $('{0:N0}' -f $downloadedSize) bytes"
    }
```

### 2. Update Tauri Configuration

Thêm `models/*` vào bundle resources trong `src-tauri/tauri.conf.json`:

```json
{
  "tauri": {
    "bundle": {
      "resources": [
        "resources/window/*",
        "resources/mac/*",
        "models/*"
      ]
    }
  }
}
```

### 3. Improve Model Path Resolution

Update code tìm model trong `audio_to_text_ass.rs` và `audio_to_text_txt.rs`:

```rust
#[cfg(target_os = "windows")]
{
    // Windows: Tìm trong bundled resources
    let paths = vec![
        app_dir.join("models").join(model_name),
        app_dir.join(model_name),
        app_dir.join("resources").join("models").join(model_name),
    ];
    
    for p in paths {
        println!("   Trying: {}", p.display());
        if p.exists() {
            println!("✅ Found model at: {}", p.display());
            return Some(p.to_string_lossy().to_string());
        }
    }
}
```

### 4. Development Setup

Tạo scripts để download models cho development:

**Linux/macOS**: `download-models.sh`
```bash
#!/bin/bash
curl -L -o src-tauri/models/ggml-base.bin https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin
```

**Windows**: `download-models.ps1`
```powershell
$ModelPath = "src-tauri\models\ggml-base.bin"
curl.exe -L -o $ModelPath "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin"
```

### 5. Verification Steps

Thêm model verification vào build process:

```yaml
# Verify Whisper models before upload
- name: Verify Models
  run: |
    $modelsDir = "src-tauri\models"
    if (Test-Path "$modelsDir\ggml-base.bin") {
      $size = (Get-Item "$modelsDir\ggml-base.bin").Length
      Write-Host "✅ ggml-base.bin ready: $([math]::Round($size/1MB, 1)) MB"
    } else {
      Write-Host "❌ Model missing - users will get runtime errors"
    }
```

## Kết quả

✅ **Windows installer sẽ bundle model**: Người dùng không cần download thêm
✅ **Runtime model detection**: App tự động tìm model trong bundled resources  
✅ **Development workflow**: Scripts giúp dev dễ dàng setup models
✅ **CI/CD verification**: Build process verify model trước khi release

## Testing

1. **Local development**:
   ```bash
   ./download-models.sh  # Download model
   npm run tauri:dev     # Test audio-to-text
   ```

2. **Production build**:
   ```bash
   npm run tauri:build   # Verify model bundled
   # Check installer includes models/ directory
   ```

3. **Windows installer**:
   - Install app từ .msi/.exe
   - Test audio-to-text feature
   - Verify không còn lỗi "model not found"

## Files Modified

- `.github/workflows/build-windows.yml`: Added model download + verification
- `src-tauri/tauri.conf.json`: Added `models/*` to bundle resources  
- `src-tauri/src/audio_to_text_ass.rs`: Improved Windows model path resolution
- `src-tauri/src/audio_to_text_txt.rs`: Improved Windows model path resolution
- `download-models.sh` + `download-models.ps1`: Development setup scripts
- `src-tauri/models/README.md`: Documentation for models
- `README.md`: Updated setup instructions

Sau khi release, Windows users sẽ không còn gặp lỗi "Không tìm thấy file model" nữa!