# 🎯 **Audio to Text ASS Export Architecture**

## 📁 **Cấu trúc mới được tách biệt:**

### **1. Core Logic: `utils/audio_to_text_ass.rs`**
```rust
// Chứa tất cả logic export ASS thuần túy, tái sử dụng được
```

**🔧 Main Components:**
- `TranscriptSegment` - Struct chứa timestamp + text
- `CaptionPosition` - Enum vị trí caption (Top/Center/Bottom)
- `VideoFormat` - Enum format video (Landscape/Portrait)
- `AssColorConfig` - Struct cấu hình màu sắc
- `AssExportConfig` - Struct cấu hình hoàn chỉnh

**🎯 Key Functions:**
```rust
// Tạo nội dung ASS từ segments (return String)
pub fn segments_to_ass_content(
    segments: &[TranscriptSegment],
    config: &AssExportConfig,
) -> Result<String>

// Export trực tiếp ra file
pub fn export_segments_to_ass_file(
    segments: &[TranscriptSegment],
    config: &AssExportConfig,
    output_path: &str,
) -> Result<()>
```

### **2. Audio Processing: `audio_to_text.rs`**
```rust
// Chỉ chứa logic Whisper + Audio processing
```

**🔧 Responsibilities:**
- Whisper model loading & management
- Audio format conversion (FFmpeg)
- Speech-to-text transcription
- Calling ASS export module

**🎯 Cleaner Functions:**
```rust
pub fn audio_to_ass(...) -> Result<String> {
    // 1. Load Whisper model
    // 2. Process audio
    // 3. Transcribe to segments  
    // 4. Call ASS export module
    let ass_config = AssExportConfig::from_options(...);
    export_segments_to_ass_file(&segments, &ass_config, &final_output)?;
}
```

## 🚀 **Lợi ích của kiến trúc mới:**

### **✅ Separation of Concerns:**
```
audio_to_text.rs     → Whisper processing
audio_to_text_ass.rs → ASS formatting & export
```

### **✅ Reusability:**
```rust
// Có thể dùng ở nhiều nơi khác
use crate::utils::audio_to_text_ass::{
    segments_to_ass_content,
    AssExportConfig
};

// Từ any source → ASS content
let content = segments_to_ass_content(&segments, &config)?;
```

### **✅ Testability:**
```rust
#[cfg(test)]
mod tests {
    // Test ASS logic riêng biệt
    // Test color conversion
    // Test karaoke generation
    // Test configuration options
}
```

### **✅ Extensibility:**
```rust
// Dễ dàng thêm output formats khác:
// audio_to_text_srt.rs
// audio_to_text_vtt.rs
// audio_to_text_txt.rs
```

## 🎨 **Usage Examples:**

### **1. Standard workflow (như cũ):**
```rust
// Frontend calls
await invoke('convert_audio_to_ass', {
    inputPath: file,
    textColor: '#FF0000',
    borderColor: '#000000',
    highlightColor: '#FFFF00'
});
```

### **2. Direct ASS content generation:**
```rust
// New: Tạo ASS content từ segments có sẵn
await invoke('segments_to_ass_string', {
    segments: [(0.0, 5.0, "Hello world"), (5.0, 10.0, "How are you?")],
    textColor: '#FF0000'
});
```

### **3. Custom integration:**
```rust
// Trong Rust code khác
let segments = get_segments_from_somewhere();
let config = AssExportConfig {
    colors: AssColorConfig {
        text_color: "FF0000".to_string(),
        border_color: "000000".to_string(), 
        highlight_color: "00FF00".to_string(),
    },
    enable_karaoke: true,
    ..Default::default()
};

let ass_content = segments_to_ass_content(&segments, &config)?;
// Use ass_content for preview, API response, etc.
```

## 🔄 **Migration & Compatibility:**

### **✅ Backward Compatible:**
- Existing `convert_audio_to_ass` API unchanged
- Frontend code continues to work
- Same file output format

### **✅ Enhanced APIs:**
```rust
// New command cho advanced use cases
segments_to_ass_string() // Return content thay vì file path
```

### **✅ Future Extensible:**
```rust
// Có thể thêm sau:
convert_audio_to_srt()
convert_audio_to_vtt() 
convert_audio_to_json()
// Tất cả đều share core Whisper logic
```

## 🎯 **File Organization:**

```
src-tauri/src/
├── audio_to_text.rs           # Whisper + Audio processing
├── utils/
│   ├── mod.rs                 # Export modules
│   ├── find_ffmpeg_by_os.rs   # FFmpeg utilities
│   └── audio_to_text_ass.rs   # 🆕 ASS export logic
└── main.rs                    # Tauri commands
```

## 🚀 **Next Steps Possible:**

1. **More Output Formats:**
   - `audio_to_text_srt.rs`
   - `audio_to_text_vtt.rs` 
   - `audio_to_text_txt.rs`

2. **Enhanced ASS Features:**
   - Font selection
   - Animation effects
   - Multi-language subtitles
   - Custom styling templates

3. **Performance Optimizations:**
   - Model caching
   - Batch processing
   - Streaming transcription

4. **Integration Possibilities:**
   - Real-time preview
   - API endpoints
   - Batch file processing
   - External service integration

Kiến trúc mới này làm cho code **cleaner**, **more maintainable**, và **highly reusable**! 🎉