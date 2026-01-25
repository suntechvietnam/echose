import os
import sys
import torch
from functools import partial

print("Step 1: Importing TTS...")
from TTS.api import TTS

print("Step 2: Security patch...")
torch.load = partial(torch.load, weights_only=False)

try:
    print("Step 3: Initializing model (Glow-TTS - small)...")
    model_name = "tts_models/en/ljspeech/glow-tts"
    tts = TTS(model_name).to("cpu")
    
    print("Step 4: Model loaded. Listing speakers...")
    if hasattr(tts, 'speakers') and tts.speakers:
        print(f"Available speakers: {tts.speakers[:5]}... (Total: {len(tts.speakers)})")
    else:
        print("No speakers found in attributes.")
        
    print("Step 5: Synthesizing simple text...")
    tts.tts_to_file(
        text="Hello world test",
        speaker=tts.speakers[0],
        language="en",
        file_path="/tmp/debug_out.wav"
    )
    print("Step 6: Success! Check /tmp/debug_out.wav")

except Exception as e:
    print(f"ERROR: {str(e)}")
    import traceback
    traceback.print_exc()
