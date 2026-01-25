import urllib.request
import json
import os

url = "http://127.0.0.1:5555/tts"
ref_audio = "/Users/kiennguyentien/Downloads/エピソード：休みの日の出来ること。 Học và luyện nghe hiểu Tiếng Nhật dễ dàng hơn qua v....mp3"
output_path = "/Users/kiennguyentien/Downloads/test_voice_clone_slow.wav"

text = """今回の決定は、会社の将来のために不可欠な投資であると同時に、社員の皆さまの努力に報いることができず、大変な心苦しさを感じています。

この変革を成功させ、開発中のプロダクトが市場で評価され、利益を生み出すことができた際には、その成果を必ず全員に還元することをお約束いたします。これは、会社の成長を支えてくれる皆さまへの最大の報いであると考えます。"""

payload = {
    "text": text,
    "reference_audio": ref_audio,
    "output_path": output_path,
    "lang": "ja",
    "speed": 0.5
}

print(f"Sending request to {url}...")
data = json.dumps(payload).encode('utf-8')
req = urllib.request.Request(url, data=data, headers={'Content-Type': 'application/json'}, method='POST')

try:
    with urllib.request.urlopen(req, timeout=600) as response:
        print(f"Status Code: {response.getcode()}")
        resp_data = json.loads(response.read().decode('utf-8'))
        print(f"Response: {resp_data}")
        
        saved_path = resp_data.get("output_path")
        if saved_path:
            print(f"Original output saved to: {saved_path}")
            
            # Post-processing with FFmpeg for speed
            speed = 0.5
            if abs(speed - 1.0) > 0.01:
                print(f"Applying FFmpeg atempo filter for speed {speed}...")
                final_path = output_path.replace(".wav", f"_speed_{speed}.wav")
                cmd = f'ffmpeg -y -i "{saved_path}" -filter:a "atempo={speed}" "{final_path}"'
                print(f"Running: {cmd}")
                os.system(cmd)
                print(f"✅ Final Corrected Audio: {final_path}")
            else:
                print(f"✅ Final Audio: {saved_path}")

except Exception as e:
    print(f"Error: {e}")
