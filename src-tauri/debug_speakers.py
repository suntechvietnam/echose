import torch
from functools import partial
from TTS.api import TTS
torch.load = partial(torch.load, weights_only=False)
tts = TTS("tts_models/multilingual/multi-dataset/xtts_v2").to("cpu")

print("Checking speakers...")
try:
    print(f"Type of speakers: {type(tts.speakers)}")
    print(f"Speakers list: {tts.speakers[:5]}")
except Exception as e:
    print(f"Failed to access tts.speakers: {e}")

try:
    print(f"Synthesizer speakers: {tts.synthesizer.tts_speakers[:5]}")
except Exception as e:
    print(f"Failed to access tts.synthesizer.tts_speakers: {e}")
