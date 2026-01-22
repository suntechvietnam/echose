import argparse
import sys
import os

# Đảm bảo import được thư viện fam từ thư mục python_libs
script_dir = os.path.dirname(os.path.abspath(__file__))
lib_path = os.path.join(script_dir, "python_libs", "metavoice-src")
if lib_path not in sys.path:
    sys.path.insert(0, lib_path)

def main():
    parser = argparse.ArgumentParser(description="MetaVoice-1B TTS Runner (Mock for Echose)")
    parser.add_argument("--text", required=True, help="Text to synthesize")
    parser.add_argument("--output", required=True, help="Output MP3/WAV file path")
    parser.add_argument("--reference", help="Path to reference audio for cloning")
    
    args = parser.parse_args()
    
    # Thực hiện chuyển đổi thực tế
    try:
        from fam.llm.fast_inference import TTS
        
        print("Đang khởi tạo mô hình MetaVoice-1B (lần đầu có thể mất thời gian để tải weight)...")
        # Khởi tạo model với các tham số mặc định
        # encodec_model_path và các path khác thường được tự động tải về ~/.cache
        tts_engine = TTS()
        
        print(f"Đang tạo âm thanh cho: {args.text[:30]}...")
        
        # MetaVoice-1B trả về path tới file wav tạm thời
        output_wav_path = tts_engine.synthesise(
            text=args.text,
            spk_ref_path=args.reference if args.reference else None,
            top_p=0.95,
            guidance_scale=3.0
        )
        
        # Di chuyển hoặc convert file kết quả sang output mong muốn
        import shutil
        import subprocess
        
        # Nếu output yêu cầu mp3 mà kết quả là wav, ta dùng ffmpeg
        if args.output.endswith(".mp3"):
            subprocess.run([
                "ffmpeg", "-y", "-i", output_wav_path, 
                "-codec:a", "libmp3lame", "-q:a", "2", args.output
            ], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        else:
            shutil.copy(output_wav_path, args.output)
        
        print(f"✅ Đã tạo thành công: {args.output}")
        
    except ImportError as e:
        print(f"❌ Lỗi import: {e}", file=sys.stderr)
        import traceback
        traceback.print_exc()
        sys.exit(1)
    except Exception as e:
        print(f"❌ Lỗi thực thi MetaVoice: {e}", file=sys.stderr)
        import traceback
        traceback.print_exc()
        sys.exit(1)

if __name__ == "__main__":
    main()
