# Whisper Models for echose

This directory contains Whisper models for audio-to-text conversion.

## Required Models

The application requires `ggml-base.bin` model to work properly.

### Download Models

You can download the base model from HuggingFace:

```bash
# Base model (~142 MB) - Required for audio-to-text conversion
curl -L -o ggml-base.bin https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin
```

### Model Information

| Model | Size | Performance |
|-------|------|-------------|
| ggml-base.bin | ~142 MB | Fast, good accuracy for most use cases |

### Development vs Production

- **Development**: Models are loaded from `src-tauri/models/`
- **Production**: Models are bundled into the app installer via `tauri.conf.json`

### Troubleshooting

If you see "Không tìm thấy file model: ggml-base.bin" error:

1. Download the model to this directory
2. For production builds, ensure `models/*` is in `tauri.conf.json` bundle resources
3. Check the build logs to verify model was downloaded during CI/CD

### GitHub Actions Build

The Windows build automatically downloads `ggml-base.bin` during the build process.
See `.github/workflows/build-windows.yml` for details.