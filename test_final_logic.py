import os
import time
import requests
import subprocess

# Cấu hình
PYTHON_VENV = "/Users/kiennguyentien/Documents/Balocco/echose/src-tauri/python_libs/coqui-tts/venv/bin/python"
SERVER_SCRIPT = "/Users/kiennguyentien/Documents/Balocco/echose/src-tauri/coqui_server.py"
REF_AUDIO = "/Users/kiennguyentien/Downloads/bria.mp3"
TEXT = """この度は昇給と契約社員から正社員への転換、また賞与の対象にしていただき、本当にありがとうございます。
しかし、私のさらなる成長と今後の貢献を考慮し、今後の給与に関してもう少しご検討いただけますでしょうか。
どうぞよろしくお願いいたします。

また、Balocco JPでの昇給プロセスについて教えていただけますと幸いです。
（私は2022年9月からBalocco Vietnam、2024年2月からBalocco Japanで働いています。Baloccoに入社してから2年が経ち、現在までに5年以上のプログラミングに関する業務経験があります。毎日、日本語の勉強を続けており、より良い仕事ができるように努力しています。日本に来たとき、私の日本語がまだ未熟で、経験も少ないことを理解しました。そのため、給与は新卒の学生と同じです。）"""

def split_japanese_text(text, max_chars=120):
    sentences = []
    current = ''
    for char in text:
        current += char
        if char in ['。', '！', '？', '\n'] or (char == '）' and len(current) > 50):
            if current.strip(): sentences.append(current.strip())
            current = ''
    if current.strip(): sentences.append(current.strip())
    
    chunks = []
    current_chunk = ''
    for sentence in sentences:
        if len(sentence) > max_chars:
            if current_chunk: chunks.append(current_chunk.strip()); current_chunk = ''
            parts = sentence.split('、')
            for part in parts:
                if (len(current_chunk) + len(part)) > max_chars and current_chunk:
                    chunks.append(current_chunk.strip()); current_chunk = part
                else:
                    current_chunk += part if not current_chunk else '、' + part
            continue
        if (len(current_chunk) + len(sentence)) > max_chars and current_chunk:
            chunks.append(current_chunk.strip()); current_chunk = sentence
        else:
            current_chunk += sentence
    if current_chunk.strip(): chunks.append(current_chunk.strip())
    return chunks

def test():
    # 1. Khởi động server
    print("🚀 [TEST] Bắt đầu tự test cho anh...")
    env = os.environ.copy()
    env["COQUI_TOS_AGREED"] = "1"
    env["MECABRC"] = "/Users/kiennguyentien/Documents/Balocco/echose/src-tauri/mecabrc"
    
    server_proc = subprocess.Popen(
        [PYTHON_VENV, SERVER_SCRIPT, "--port", "5557"],
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True
    )
    
    ready = False
    for _ in range(120):
        try:
            res = requests.get("http://127.0.0.1:5557/health")
            if res.status_code == 200 and res.json().get("model_loaded"):
                print("✅ [TEST] Server đã sẵn sàng và nạp model thành công!")
                ready = True
                break
        except: pass
        time.sleep(1)
        
    if not ready:
        print("❌ [TEST] Server không khởi động được!")
        server_proc.terminate(); return

    # 2. Chia đoạn
    chunks = split_japanese_text(TEXT)
    print(f"✂️ [TEST] Đã chia văn bản thành {len(chunks)} đoạn.")
    
    # 3. Render từng đoạn
    all_start = time.time()
    for i, chunk in enumerate(chunks):
        print(f"⏳ [TEST] Rendering đoạn {i+1}/{len(chunks)} ({len(chunk)} ký tự)...", end="", flush=True)
        start = time.time()
        res = requests.post("http://127.0.0.1:5557/tts", json={
            "text": chunk,
            "reference_audio": REF_AUDIO,
            "output_path": f"/tmp/test_chunk_{i}.wav",
            "lang": "ja"
        })
        end = time.time()
        if res.status_code == 200:
            print(f" Done! ({end - start:.2f}s)")
        else:
            print(f" Failed! {res.text}")
            break
            
    total_elapsed = time.time() - all_start
    print(f"\n🎉 [HOÀN TẤT] Tổng thời gian xử lý AI: {total_elapsed:.2f} giây")
    print(f"⏱️ Trung bình: {total_elapsed/len(chunks):.2f} giây/đoạn")
    
    server_proc.terminate()

if __name__ == "__main__":
    test()
