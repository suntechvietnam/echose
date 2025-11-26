# 🎨 Demo: Tùy chỉnh màu sắc cho Audio to Text ASS

## ✨ Tính năng mới đã thêm:

### 🔧 **Backend (Rust)**
- ✅ Thêm 3 tham số màu sắc vào `audio_to_ass()`:
  - `text_color`: Màu chữ (hex: "FFFFFF")
  - `border_color`: Màu viền (hex: "000000") 
  - `highlight_color`: Màu karaoke highlight (hex: "00FFFF")

- ✅ Hàm `hex_to_ass_bgr()` chuyển đổi RGB → BGR cho ASS format
- ✅ Dynamic ASS styles với màu tùy chỉnh
- ✅ Cập nhật Tauri command với 3 tham số màu mới

### 🎨 **Frontend (Vue.js)**
- ✅ Color picker inputs cho 3 loại màu
- ✅ Video format selector (Landscape/Portrait)
- ✅ Caption position selector (Top/Center/Bottom)
- ✅ Live preview màu hex values
- ✅ Responsive UI với ASS options panel

## 🎯 **Cách sử dụng:**

### **1. Chọn file audio/video**
```javascript
// Hỗ trợ formats: MP3, WAV, M4A, AAC, OGG, FLAC, MP4, AVI, MOV, MKV, WEBM, FLV
```

### **2. Chọn định dạng ASS**
```javascript
outputFormat.value = 'ass' // Kích hoạt ASS options panel
```

### **3. Tùy chỉnh màu sắc**
```javascript
// Frontend values (hex with #)
textColor: '#FF0000'      // Đỏ
borderColor: '#000000'    // Đen
highlightColor: '#FFFF00' // Vàng

// Tự động chuyển đổi sang format cho Rust (bỏ #)
textColor: 'FF0000'
borderColor: '000000'
highlightColor: 'FFFF00'
```

### **4. Chọn video format & position**
```javascript
videoFormat: 'landscape' | 'portrait'
captionPosition: 'top' | 'center' | 'bottom'
```

### **5. Convert**
```javascript
const outputPath = await invoke('convert_audio_to_ass', {
  inputPath: mediaFile.value,
  outputAssPath: null,
  modelPath: null,
  language: 'vi',
  position: captionPosition.value,
  videoFormat: videoFormat.value,
  textColor: textColor.value.replace('#', ''),
  borderColor: borderColor.value.replace('#', ''),
  highlightColor: highlightColor.value.replace('#', '')
})
```

## 🌈 **Ví dụ màu sắc phổ biến:**

### **🎬 YouTube Style**
```javascript
textColor: '#FFFFFF'      // Trắng
borderColor: '#000000'    // Đen
highlightColor: '#FF0000' // Đỏ YouTube
```

### **🎵 TikTok Style**
```javascript
textColor: '#FFFFFF'      // Trắng
borderColor: '#000000'    // Đen  
highlightColor: '#FF0050' // Hồng TikTok
```

### **📱 Instagram Reels**
```javascript
textColor: '#FFFFFF'      // Trắng
borderColor: '#833AB4'    // Tím Instagram
highlightColor: '#F56040' // Cam Instagram
```

### **🎮 Gaming Style**
```javascript
textColor: '#00FF00'      // Xanh lá neon
borderColor: '#000000'    // Đen
highlightColor: '#FFFF00' // Vàng neon
```

## 🔧 **Technical Details:**

### **ASS Color Format**
```rust
// RGB hex: "FF0000" (red)
// ASS BGR: "&H000000FF" (red in ASS)

fn hex_to_ass_bgr(hex: &str) -> String {
    // "FF0000" -> "0000FF" (RGB to BGR)
    let r = &hex[0..2]; // "FF"  
    let g = &hex[2..4]; // "00"
    let b = &hex[4..6]; // "00"
    format!("{}{}{}", b, g, r) // "0000FF"
}
```

### **ASS Styles Generated**
```ass
[V4+ Styles]
Style: Default,Arial,58,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,-1,0,0,0,100,100,0,0,1,3.0,1.5,5,10,10,80,1
Style: Karaoke,Arial,58,&H00FFFFFF,&H0000FFFF,&H00000000,&H80000000,-1,0,0,0,100,100,0,0,1,3.0,1.5,5,10,10,80,1
```

### **Responsive Video Formats**
```rust
// Landscape (16:9) - 1920x1080
font_size: 58, margin_lr: 10, outline: 3.0

// Portrait (9:16) - 1080x1920  
font_size: 42, margin_lr: 40, outline: 2.5
```

## 🚀 **Next Steps:**

1. ✅ ~~Thêm color picker UI~~
2. ✅ ~~Backend color parameter support~~
3. 🔄 Thêm color presets (YouTube, TikTok, etc.)
4. 🔄 Font family selection
5. 🔄 Font size customization
6. 🔄 Advanced karaoke effects
7. 🔄 Real-time ASS preview

## 🎉 **Kết quả:**

Giờ đây bạn có thể:
- 🎨 Tùy chỉnh màu chữ, viền, và highlight
- 📱 Tối ưu cho video landscape/portrait
- 📍 Đặt caption ở top/center/bottom
- ⚡ Tạo ASS professional với karaoke effects
- 🌈 Phù hợp với brand colors của các platform