import os
import torch
from functools import partial
print("Importing...")
from TTS.api import TTS
torch.load = partial(torch.load, weights_only=False)
print("Loading XTTS v2...")
tts = TTS("tts_models/multilingual/multi-dataset/xtts_v2").to("cpu")
print("Speakers:", tts.speakers[:5])
print("Languages:", tts.languages)
