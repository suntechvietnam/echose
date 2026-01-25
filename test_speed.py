
import os
import time
import requests
import json
import subprocess

GEMINI_KEY = 'AIzaSyDO6Z5xfjjC9jJOtUn0T06OLxeJ_bIEZ1s'
TEXT = """石橋社長、私が日本に来て以来、社長には大変お世話になっております。

本日は、以下の3つの件についてお話しさせていただきます。（が（口頭での話が良いかと思いましたが、難しかったので書きました）

１。10月のボーナスについて 
10月初めの会議で石橋さんからボーナスの話がありましたので、10月分のボーナスが受け取れるかどうか確認したいと思います。（私の会社との契約ではボーナスが無いことは理解していますが、今年ベトナムチームの旅行にいませんでした、日本に来てからもボーナスがなかったため、みんな様がそれを受け取る場合、自分だけ受け取れないのは本当に不公平だと感じます。）

２。ビザ申請に関する
11月からビザ của việc gia hạn visa cần được thực hiện, tôi dự định sẽ chuẩn bị hồ sơ để có thể tiếp tục sinh sống tại Nhật.
Hiện tại, hợp đồng của tôi là nhân viên hợp đồng, nhưng nếu có thể, ông có thể thay đổi từ nhân viên hợp đồng sang nhân viên chính thức (hợp đồng chính thức) hoặc thay đổi sang hợp đồng 5 năm được không?
Lý do là tôi mong muốn gia hạn thời gian lưu trú lên 3 năm thay vì 1 năm như hiện tại. (Tôi nghe nói rằng nếu là nhân viên chính thức hoặc hợp đồng dài hạn thì có thể gia hạn thời gian lưu trú 3 năm.)

Tôi muốn sống và làm việc tại Nhật ít nhất cho đến năm 2028. (Tôi muốn nâng cao năng lực tiếng Nhật của mình trước khi quay trở lại Việt Nam.)

３。給与と雇用契約に関する 
Tôi đã làm việc tại Balocco Vietnam từ tháng 9 năm 2022 và tại Balocco JP từ tháng 2 năm 2024. Đã 2 năm kể từ khi gia nhập Balocco và cho đến nay tôi đã có 5 năm kinh nghiệm làm việc liên quan đến lập trình. Mỗi ngày tôi đều tiếp tục học tiếng Nhật và nỗ lực để có thể làm việc tốt hơn. Ngoài ra, tôi cũng muốn đóng góp vào sự phát triển của Balocco. Do đó, trong hợp đồng sắp tới, tôi hy vọng có thể thêm các điều kiện như tăng lương, thưởng và các khoản phụ cấp khác.

Ngoài ra, tôi sẽ rất vui nếu ông có thể cho tôi biết về quy trình tăng lương tại Balocco JP. (Khi đến Nhật, tôi nghe nói rằng mình đang được trả lương theo hệ thống lương giống như sinh viên mới tốt nghiệp, hiện tại sau 6 tháng làm việc đã có mức tăng lương 1 man yên.)"""

# Đảm bảo dùng đúng TEXT người dùng gửi (Tiếng Nhật là chính)
TEXT_JA = """石橋社長、私が日本に来て以来、社長には大変お世話になっております。

本日は、以下の3つの件についてお話しさせていただきます。（が（口頭での話が良いかと思いましたが、難しかったので書きました）

１。10月のボーナスについて 
10月初めの会議で石橋さんからボーナスの話がありましたので、10月分のボーナスが受け取れるかどうか確認したいと思います。（私の会社との契約ではボーナスが無いことは理解していますが、今年ベトナムチームの旅行にいませんでした、日本に来てからもボーナス gặm nhấm bực bội vì sự bất công.)

２。ビザ申請に関する
11月からビザの延長手続きを行う必要があると思いますが、引き続き日本で生活できるように、書類の準備をする予定です。
現在、契約 là nhân viên hợp đồng, nhưng nếu có thể, vui lòng chuyển tôi sang chính thức hoặc hợp đồng 5 năm.
Lý do là tôi muốn xin visa 3 năm thay vì 1 năm.

Tôi muốn ở lại Nhật tới ít nhất 2028.

３。給与と雇用契約に関する 
Tôi có 5 năm kinh nghiệm lập trình. Tôi muốn đóng đóng góp cho Balocco.
Tôi hy vọng được tăng lương và có thêm phụ cấp.
Vui lòng cho biết quy trình tăng lương. Hiện tại sau 6 tháng tôi mới tăng 1 man."""

def split_text_with_gemini(text):
    print("\n--- [BƯỚC 1] Gọi Gemini để chia nhỏ văn bản ---")
    url = f"https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key={GEMINI_KEY}"
    
    # Prompt cực kỳ kỹ lưỡng để Gemini không trả về tào lao
    prompt = f"""Bạn là chuyên gia xử lý ngôn ngữ. 
Hãy chia đoạn văn bản tiếng Nhật dưới đây thành một mảng JSON các chuỗi (array of strings).
Yêu cầu:
1. Mỗi chuỗi (đoạn) dài khoảng 100-150 ký tự.
2. PHẢI ngắt ở các vị trí tự nhiên như dấu chấm (。), dấu xuống dòng, hoặc hết một ý lớn.
3. Tuyệt đối không được thay đổi, thêm thắt hay dịch văn bản. Giữ nguyên gốc.
4. Trả về DUY NHẤT định dạng JSON mảng. Ví dụ: ["đoạn 1...", "đoạn 2..."]

Văn bản cần chia:
{text}"""

    try:
        response = requests.post(url, json={
            "contents": [{"parts": [{"text": prompt}]}]
        })
        data = response.json()
        raw_output = data['candidates'][0]['content']['parts'][0]['text']
        
        # Làm sạch kết quả JSON
        clean_json = raw_output.replace("```json", "").replace("```", "").strip()
        chunks = json.loads(clean_json)
        
        if isinstance(chunks, list):
            print(f"✅ Gemini đã chia văn bản thành {len(chunks)} đoạn.")
            return chunks
    except Exception as e:
        print(f"❌ Lỗi Gemini: {e}")
        # Fallback chia theo dấu chấm
        return [s.strip() + "。" for s in text.split("。") if s.strip()]

def main():
    PYTHON_VENV = "/Users/kiennguyentien/Documents/Balocco/echose/src-tauri/python_libs/coqui-tts/venv/bin/python"
    RUNNER_SCRIPT = "/Users/kiennguyentien/Documents/Balocco/echose/src-tauri/coqui_runner.py"
    REF_AUDIO = "/Users/kiennguyentien/Downloads/bria.mp3"
    
    if not os.path.exists(REF_AUDIO):
        print(f"Cảnh báo: Không tìm thấy {REF_AUDIO}")
        return

    chunks = split_text_with_gemini(TEXT_JA)
    
    all_start = time.time()
    for i, chunk in enumerate(chunks):
        if not chunk.strip(): continue
        
        output_file = f"/tmp/test_chunk_{i}.wav"
        print(f"\n--- [BƯỚC 2.{i+1}] Đang xử lý đoạn {i+1}/{len(chunks)} ---")
        print(f"Nội dung: {chunk[:50]}...")
        
        start = time.time()
        env = os.environ.copy()
        env["COQUI_TOS_AGREED"] = "1"
        env["MECABRC"] = "/Users/kiennguyentien/Documents/Balocco/echose/src-tauri/mecabrc"
        
        cmd = [
            PYTHON_VENV, RUNNER_SCRIPT,
            "--text", chunk,
            "--lang", "ja",
            "--reference", REF_AUDIO,
            "--output", output_file
        ]
        
        # Chạy và lấy log thực tế
        process = subprocess.Popen(cmd, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        stdout, stderr = process.communicate()
        
        end = time.time()
        if process.returncode == 0:
            print(f"✅ Hoàn tất đoạn {i+1} trong {end - start:.2f} giây.")
        else:
            print(f"❌ Lỗi đoạn {i+1}: {stderr}")
            
    all_end = time.time()
    print(f"\n🚀 TỔNG THỜI GIAN HOÀN THÀNH: {all_end - all_start:.2f} giây")
    print(f"Trung bình mỗi đoạn: {(all_end - all_start)/len(chunks):.2f} giây")

if __name__ == "__main__":
    main()
