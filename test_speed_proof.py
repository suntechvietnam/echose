import requests
import time
import os
import wave
import contextlib

def get_duration(fname):
    with contextlib.closing(wave.open(fname, 'r')) as f:
        frames = f.getnframes()
        rate = f.getframerate()
        return frames / float(rate)

def test_speed(speed, output_path):
    url = "http://localhost:5555/tts"
    payload = {
        "text": "こんにちは、これはスピードテストです。ゆっくり話していますか？",
        "reference_audio": "/Users/kiennguyentien/Downloads/bria.mp3",
        "output_path": output_path,
        "lang": "ja",
        "speed": speed
    }
    
    print(f"Testing Speed: {speed}...")
    try:
        start = time.time()
        res = requests.post(url, json=payload)
        print(f"Status: {res.status_code}")
        if res.status_code == 200:
            dur = get_duration(output_path)
            print(f"✅ Generated Audio Duration: {dur:.2f}s")
            return dur
        else:
            print(f"❌ Failed: {res.text}")
            return 0
    except Exception as e:
        print(f"❌ Connection Error: {e}")
        return 0

# Check if server is up
try:
    requests.get("http://localhost:5555/health")
except:
    print("⚠️ Server not running. Please wait for App to start...")
    exit(1)

print("🔍 PROOF OF VERIFICATION (BẰNG CHỨNG XÁC THỰC)\n")

# Test 1: Normal Speed (1.0)
dur_normal = test_speed(1.0, "/tmp/proof_speed_1.0.wav")

# Test 2: Slow Speed (0.7)
dur_slow = test_speed(0.7, "/tmp/proof_speed_0.7.wav")

print("\n--- KẾT QUẢ SO SÁNH ---")
print(f"⏱️ Normal (1.0): {dur_normal:.2f}s")
print(f"⏱️ Slow (0.7):   {dur_slow:.2f}s")

if dur_slow > dur_normal:
    print("🎉 KHẲNG ĐỊNH: Tốc độ ĐÃ THAY ĐỔI. 0.7 dài hơn 1.0!")
else:
    print("❌ THẤT BẠI: Tốc độ không đổi.")
