#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Coqui TTS Server - Model luôn sẵn sàng trong RAM
Chạy như daemon, nhận request qua HTTP
"""

import os
import sys
import json
import argparse
import functools
from flask import Flask, request, jsonify
import torch

# --- MASTER FIX FOR PYTORCH 2.6+ ---
original_torch_load = torch.load
@functools.wraps(original_torch_load)
def patched_torch_load(*args, **kwargs):
    kwargs['weights_only'] = False
    return original_torch_load(*args, **kwargs)

torch.load = patched_torch_load
# -----------------------------------

from TTS.api import TTS
import re
from kanjize import number2kanji

app = Flask(__name__)

def normalize_japanese_text(text):
    """Chuẩn hóa văn bản tiếng Nhật toàn diện cho TTS"""
    if not text:
        return text
    
    replacements = {
        '~': 'から', '～': 'から', '&': 'アンド', '%': 'パーセント', 
        '％': 'パーセント', '$': 'ドル', '＋': 'プラス', '+': 'プラス',
        '=': 'イコール', '＝': 'イコール', '>': 'より大きい', '<': 'より小さい',
        '#': 'シャープ', '＃': 'シャープ', '@': 'アット', '＠': 'アット'
    }
    
    for symbol, reading in replacements.items():
        text = text.replace(symbol, reading)
        
    def replace_num(match):
        try:
            num = int(match.group())
            if num > 999999999999: return match.group()
            return number2kanji(num)
        except:
            return match.group()
    
    text = re.sub(r'\d+', replace_num, text)
    text = text.replace('(', '（').replace(')', '）')
    return text

# Global model
tts_model = None
device = None

def init_model():
    global tts_model, device
    print("🚀 Đang khởi động Coqui TTS Server...")
    
    if torch.backends.mps.is_available(): device = "mps"
    elif torch.cuda.is_available(): device = "cuda"
    else: device = "cpu"
    
    print(f"🖥️  Sử dụng Device: {device}")
    tts_model = TTS("tts_models/multilingual/multi-dataset/xtts_v2")
    
    if device != "cpu":
        print(f"🎯 Đang đưa model vào {device} (Full Precision)...")
        tts_model.to(device)
    print("🚀 SERVER VERSION: 2.1 (Fixed Speed & Logs)")
    print("✅ Model đã sẵn sàng!")

@app.route('/health', methods=['GET'])
def health():
    return jsonify({"status": "ok", "model_loaded": tts_model is not None, "device": device})

@app.route('/tts', methods=['POST'])
def text_to_speech():
    try:
        data = request.json
        text = data.get('text')
        reference_audio = data.get('reference_audio')
        output_path = data.get('output_path')
        lang = data.get('lang', 'ja')
        speed = data.get('speed', 1.0)
        
        if not text or not reference_audio or not output_path:
            return jsonify({"error": "Missing required fields"}), 400
        
        if lang == 'ja':
            text = normalize_japanese_text(text)
            
        print("-" * 40)
        print(f"DEBUG - Lang: {lang}")
        print(f"DEBUG - Text: {text[:50]}...")
        print(f"DEBUG - Speed: {speed} (type: {type(speed)})")
        print("-" * 40)

        # Ép kiểu speed về float rành rọt
        try:
            speed_float = float(speed)
        except:
            speed_float = 1.0
        
        ref_files = reference_audio.split(',')
        ref_audio = ref_files[0].strip()
        
        tts_model.tts_to_file(
            text=text,
            speaker_wav=ref_audio,
            language=lang,
            file_path=output_path,
            speed=speed_float
        )
        
        return jsonify({"success": True, "output_path": output_path})
    except Exception as e:
        print(f"❌ Error: {str(e)}")
        return jsonify({"error": str(e)}), 500

if __name__ == '__main__':
    os.environ['COQUI_TOS_AGREED'] = '1'
    init_model()
    app.run(host='127.0.0.1', port=5555, debug=False, threaded=False)
