import urllib.request
import json
import os
import time

# Configurations
TTS_URL = "http://127.0.0.1:5555/tts"
GEMINI_KEY = 'AIzaSyDO6Z5xfjjC9jJOtUn0T06OLxeJ_bIEZ1s'
# Using the same reference for both as a test, or I could use different ones if I had them
REF_VOICE_1 = "/Users/kiennguyentien/Downloads/エピソード：休みの日の出来ること。 Học và luyện nghe hiểu Tiếng Nhật dễ dàng hơn qua v....mp3"
REF_VOICE_2 = REF_VOICE_1 # In real scenario, this would be the second voice file

dialogue_text = """A: ねえ、Bさん。最近、日本語の勉強は順調（じゅんちょう）？
B: うーん、まあまあかな。N3の漢字が難しくて、覚えられなくて困っているんだ。"""

def analyze_script(text):
    print("🪄 Calling Gemini to analyze script...")
    prompt = f"""Bạn là đạo diễn âm thanh. Hãy phân tích đoạn hội thoại sau và trả về định dạng mảng JSON.
    Mỗi object gồm: {{"speaker": "tên người", "text": "nội dung câu thoại", "speed": float (0.8-1.2), "pitch": int (-20 đến 20), "reason": "lý do chọn"}}.
    Hội thoại:
    "{text}" """
    
    url = f"https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key={GEMINI_KEY}"
    payload = {"contents": [{"parts": [{"text": prompt}]}]}
    
    req = urllib.request.Request(url, data=json.dumps(payload).encode('utf-8'), headers={'Content-Type': 'application/json'}, method='POST')
    with urllib.request.urlopen(req) as response:
        res_data = json.loads(response.read().decode('utf-8'))
        raw_text = res_data['candidates'][0]['content']['parts'][0]['text']
        # Extract JSON
        import re
        match = re.search(r'\[.*\]', raw_text, re.DOTALL)
        if match:
            return json.loads(match.group(0))
        return []

def generate_line(speaker, text, speed, pitch, index):
    print(f"🎙️ Generating for {speaker}: {text[:30]}... (Speed: {speed}, Pitch: {pitch})")
    
    # Map speaker to voice path
    ref = REF_VOICE_1 if speaker == "A" else REF_VOICE_2
    out_name = f"/tmp/line_{index}.wav"
    
    payload = {
        "text": text,
        "reference_audio": ref,
        "output_path": out_name,
        "lang": "ja",
        "speed": speed
    }
    
    data = json.dumps(payload).encode('utf-8')
    req = urllib.request.Request(TTS_URL, data=data, headers={'Content-Type': 'application/json'}, method='POST')
    
    with urllib.request.urlopen(req, timeout=600) as response:
        res = json.loads(response.read().decode('utf-8'))
        return res['output_path']

def main():
    # segments = analyze_script(dialogue_text)
    # Mocking Gemini response for the Japanese dialogue provided by user
    segments = [
        {"speaker": "A", "text": "ねえ、Bさん。最近、日本語の勉強は順調（じゅんちょう）？", "speed": 1.0, "pitch": 5, "reason": "Chào hỏi thân mật"},
        {"speaker": "B", "text": "うーん、まあまあかな。N3の漢字が難しくて、覚えられなくて困っているんだ。", "speed": 0.9, "pitch": 0, "reason": "Hơi buồn và mệt mỏi vì tiếng Nhật khó"},
        {"speaker": "A", "text": "わかるよ。漢字は毎日書かないと、すぐに忘れてしまうもんね。でも、Bさんはどうして日本語を勉強し始めたの？", "speed": 1.1, "pitch": 5, "reason": "An ủi và gợi mở câu chuyện"},
        {"speaker": "B", "text": "実は、将来日本でプログラマーとして働きたいと思っているんだ。日本のIT技術は高いはずだから、いろいろ学びたいと思っている。", "speed": 1.0, "pitch": 0, "reason": "Kể về ước mơ"},
    ]
    print(f"✅ Using mock sugersted {len(segments)} segments.")
    
    temp_files = []
    for i, seg in enumerate(segments):
        path = generate_line(seg['speaker'], seg['text'], seg['speed'], seg['pitch'], i)
        temp_files.append(path)
        
    print("🔗 Merging with FFmpeg...")
    final_out = "/Users/kiennguyentien/Downloads/test_dialogue_final.mp3"
    
    # Create concat list
    with open("/tmp/concat_list.txt", "w") as f:
        for p in temp_files:
            f.write(f"file '{p}'\n")
            
    cmd = f'ffmpeg -y -f concat -safe 0 -i /tmp/concat_list.txt "{final_out}"'
    os.system(cmd)
    print(f"🎉 SUCCESS! Final dialogue saved to: {final_out}")

if __name__ == "__main__":
    main()
