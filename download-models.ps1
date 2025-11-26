# Download Whisper models for development on Windows

Write-Host "🎤 Downloading Whisper models for YTBFlow development..." -ForegroundColor Green

$ModelsDir = "src-tauri\models"

# Ensure we're in the right directory
if (-not (Test-Path $ModelsDir)) {
    Write-Host "❌ Error: $ModelsDir not found. Please run this script from the project root." -ForegroundColor Red
    exit 1
}

# Download base model (recommended for most users)
Write-Host "📥 Downloading ggml-base.bin (~142 MB)..." -ForegroundColor Yellow
$BaseModelPath = "$ModelsDir\ggml-base.bin"

if (Test-Path $BaseModelPath) {
    Write-Host "✅ ggml-base.bin already exists" -ForegroundColor Green
} else {
    try {
        # Use curl (available in Windows 10 1803+ and Windows 11)
        & curl.exe -L -o $BaseModelPath "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin" --progress-bar
        
        if (Test-Path $BaseModelPath) {
            Write-Host "✅ ggml-base.bin downloaded successfully" -ForegroundColor Green
        } else {
            throw "Model file not found after download"
        }
    } catch {
        Write-Host "❌ curl failed, trying PowerShell download..." -ForegroundColor Yellow
        try {
            Invoke-WebRequest -Uri "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin" -OutFile $BaseModelPath -UseBasicParsing
            Write-Host "✅ ggml-base.bin downloaded via PowerShell" -ForegroundColor Green
        } catch {
            Write-Host "❌ Failed to download ggml-base.bin: $_" -ForegroundColor Red
            exit 1
        }
    }
}

# Verify file size
if (Test-Path $BaseModelPath) {
    $BaseSize = (Get-Item $BaseModelPath).Length
    $BaseSizeMB = [math]::Round($BaseSize / 1MB, 1)
    
    if ($BaseSize -gt 140MB) {
        Write-Host "✅ Model file size verification passed ($BaseSizeMB MB)" -ForegroundColor Green
    } else {
        Write-Host "⚠️ Warning: Model file seems smaller than expected ($BaseSizeMB MB)" -ForegroundColor Yellow
    }
}

Write-Host ""
Write-Host "🎉 Model setup complete! You can now run audio-to-text conversion." -ForegroundColor Green