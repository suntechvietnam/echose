import torch
from functools import partial
from TTS.api import TTS
torch.load = partial(torch.load, weights_only=False)

print("Loading XTTS v2...")
tts = TTS("tts_models/multilingual/multi-dataset/xtts_v2").to("cpu")

print("Checking attributes...")
for attr in ["speakers", "speaker_names", "speaker_manager"]:
    if hasattr(tts, attr):
        val = getattr(tts, attr)
        if attr == "speaker_manager" and val:
            print(f"speaker_manager found. Speakers: {list(val.speakers.keys())[:5]}")
        else:
            print(f"{attr} found: {val[:5] if val else 'None'}")

if not hasattr(tts, "speakers") and not hasattr(tts, "speaker_manager"):
    print("No speaker attributes found in TTS object.")
