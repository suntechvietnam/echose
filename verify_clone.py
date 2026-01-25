import requests
import json
import os
import time

# 1. CẤU HÌNH
TEXT = """今回の決定は、会社の将来のために不可欠な投資であると同時に、社員の皆さまの努力に報いることができず、大変な心苦しさを感じています。

この変革を成功させ、開発中のプロダクトが市場で評価され、利益を生み出すことができた際には、その成果を必ず全員に還元することをお約束いたします。これは、会社の成長を支えてくれる皆さまへの最大の報いであると考えます。"""

# Dùng file nhạc mẫu có sẵn vì file tiếng Nhật không tìm thấy
REF_AUDIO = "/Users/kiennguyentien/Downloads/1759047676157604230-317418968924381.mp3"
OUTPUT_FILE = "/Users/kiennguyentien/Downloads/TEST_CLONE_SPEED_0.5.wav"
SPEED = 0.5  # Yêu cầu tốc độ chậm

def run_test():
    print(f"🚀 BẮT ĐẦU TEST TỰ ĐỘNG")
    print(f"📂 File mẫu: {REF_AUDIO}")
    print(f"📝 Text độ dài: {len(TEXT)} chars")
    print(f"⚡ Tốc độ: {SPEED}")

    # Check Server
    try:
        requests.get("http://localhost:5555/health", timeout=2)
        print("✅ Server Online!")
    except:
        print("❌ Server KHÔNG chạy! Vui lòng bật App lên trước.")
        return

    # Gửi lệnh Clone
    payload = {
        "text": TEXT,
        "reference_audio": REF_AUDIO,
        "output_path": OUTPUT_FILE,
        "lang": "ja",
        "speed": SPEED
    }

    start = time.time()
    try:
        print("⏳ Đang gửi request tới Server AI (sẽ mất khoảng 10-20s)...")
        res = requests.post("http://localhost:5555/tts", json=payload, timeout=600)
        
        if res.status_code == 200:
            print(f"🎉 RENDER THÀNH CÔNG!")
            print(f"📂 File saved at: {OUTPUT_FILE}")
            print(f"⏱️ Thời gian xử lý: {time.time() - start:.2f}s")
            
            # Kiểm tra file tồn tại
            if os.path.exists(OUTPUT_FILE):
                size = os.path.getsize(OUTPUT_FILE) / 1024
                print(f"📊 Dung lượng file: {size:.2f} KB")
                print("👉 HÃY MỞ FILE NÀY LÊN NGHE THỬ NGAY!")
            else:
                print("❌ Lỗi: Server báo thành công nhưng không thấy file đâu.")
        else:
            print(f"❌ Render Thất Bại: {res.text}")

    except Exception as e:
        print(f"❌ Lỗi kết nối: {e}")

if __name__ == "__main__":
    if not os.path.exists(REF_AUDIO):
        print(f"❌ Không tìm thấy file mẫu: {REF_AUDIO}")
    else:
        run_test()
