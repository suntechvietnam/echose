#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import json

def split_japanese_text(text, max_chars=120):
    """
    Chia văn bản tiếng Nhật thành các đoạn nhỏ
    """
    # Chia theo dấu chấm tiếng Nhật và xuống dòng
    sentences = []
    current = ""
    
    for char in text:
        current += char
        if char in ['。', '！', '？', '\n'] or (char == '）' and len(current) > 50):
            if current.strip():
                sentences.append(current.strip())
            current = ""
    
    if current.strip():
        sentences.append(current.strip())
    
    # Gộp các câu ngắn thành chunks
    chunks = []
    current_chunk = ""
    
    for sentence in sentences:
        # Nếu câu quá dài, chia nhỏ hơn nữa
        if len(sentence) > max_chars:
            # Lưu chunk hiện tại nếu có
            if current_chunk:
                chunks.append(current_chunk.strip())
                current_chunk = ""
            
            # Chia câu dài theo dấu phẩy hoặc khoảng trắng
            parts = sentence.split('、')
            for part in parts:
                if len(current_chunk) + len(part) > max_chars and current_chunk:
                    chunks.append(current_chunk.strip())
                    current_chunk = part
                else:
                    current_chunk += part if not current_chunk else '、' + part
            continue
        
        # Thêm câu vào chunk hiện tại
        if len(current_chunk) + len(sentence) > max_chars and current_chunk:
            chunks.append(current_chunk.strip())
            current_chunk = sentence
        else:
            current_chunk += ("\n" if sentence.startswith("１") or sentence.startswith("２") or sentence.startswith("３") else "") + sentence
    
    if current_chunk.strip():
        chunks.append(current_chunk.strip())
    
    return chunks

# Đọc văn bản từ file
with open('test_text.txt', 'r', encoding='utf-8') as f:
    text = f.read()

print("=" * 80)
print("📝 VĂN BẢN GỐC")
print("=" * 80)
print(f"Độ dài: {len(text)} ký tự\n")

# Chia đoạn
chunks = split_japanese_text(text)

print("=" * 80)
print(f"✂️ KẾT QUẢ CHIA ĐOẠN: {len(chunks)} đoạn")
print("=" * 80)

for i, chunk in enumerate(chunks, 1):
    print(f"\n--- ĐOẠN {i}/{len(chunks)} ({len(chunk)} ký tự) ---")
    print(chunk)
    print()

# Tính toán thời gian ước tính
estimated_time = len(chunks) * 25  # 25 giây/đoạn
print("=" * 80)
print("⏱️ ƯỚC TÍNH THỜI GIAN XỬ LÝ")
print("=" * 80)
print(f"Số đoạn: {len(chunks)}")
print(f"Thời gian mỗi đoạn: ~25 giây (bao gồm load model)")
print(f"Tổng thời gian: ~{estimated_time} giây ({estimated_time//60} phút {estimated_time%60} giây)")
print()

# Xuất ra JSON để dùng trong app
output = {
    "total_chars": len(text),
    "num_chunks": len(chunks),
    "chunks": chunks,
    "estimated_seconds": estimated_time
}

with open('split_result.json', 'w', encoding='utf-8') as f:
    json.dump(output, f, ensure_ascii=False, indent=2)

print("✅ Đã lưu kết quả vào split_result.json")
