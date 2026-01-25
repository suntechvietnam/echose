import os
import sys
import argparse
import torch
from functools import partial
from TTS.api import TTS

# FIX cho PyTorch 2.6+: Cho phép load model từ nguồn tin cậy
torch.load = partial(torch.load, weights_only=False)

def main():
    parser = argparse.ArgumentParser(description="Coqui TTS Runner for Echose")
    parser.add_argument("--text", type=str, required=True, help="Text to synthesize")
    parser.add_argument("--output", type=str, required=True, help="Output WAV file path")
    parser.add_argument("--reference", type=str, help="Reference audio file for cloning")
    parser.add_argument("--lang", type=str, default="vi", help="Language code (vi, ja, en, ...)")
    
    args = parser.parse_args()

    # Xác định thiết bị: Ưu tiên MPS (Mac M1)
    device = "mps" if torch.backends.mps.is_available() else "cpu"
    print(f"Sử dụng thiết bị: {device}")

    try:
        # Khởi tạo XTTS v2
        print("Đang khởi tạo model XTTS v2...")
        tts = TTS("tts_models/multilingual/multi-dataset/xtts_v2").to(device)

        if args.reference:
            # Hỗ trợ truyền nhiều file mẫu qua dấu phẩy
            ref_files = [f.strip() for f in args.reference.split(",")]
            # Chỉ giữ lại các file thực sự tồn tại
            ref_files = [f for f in ref_files if os.path.exists(f)]
            
            if ref_files:
                print(f"Đang clone giọng từ {len(ref_files)} file mẫu: {ref_files}")
                tts.tts_to_file(
                    text=args.text,
                    speaker_wav=ref_files, # tts_to_file nhận list file
                    language=args.lang,
                    file_path=args.output
                )
            else:
                print("❌ Không tìm thấy các file mẫu truyền vào.")
                sys.exit(1)
        else:
            print("Không có file mẫu, đang tìm giọng mặc định...")
            # XTTS v2 cần một speaker mẫu. Ta sẽ thử lấy từ danh sách hoặc dùng tên mặc định.
            speaker_name = "Claribel Dervla" 
            try:
                if hasattr(tts, 'speakers') and tts.speakers:
                    speaker_name = tts.speakers[0]
            except:
                pass
            
            print(f"Sử dụng giọng: {speaker_name}")
            tts.tts_to_file(
                text=args.text,
                speaker=speaker_name,
                language=args.lang,
                file_path=args.output
            )

        print(f"✅ Thành công! Âm thanh đã được lưu tại: {args.output}")

    except Exception as e:
        print(f"❌ Lỗi thực thi Coqui TTS: {str(e)}")
        # In thêm thông tin debug
        if 'tts' in locals() and hasattr(tts, 'languages'):
            print(f"Ngôn ngữ hỗ trợ: {tts.languages}")
        sys.exit(1)
        sys.exit(1)

if __name__ == "__main__":
    main()
