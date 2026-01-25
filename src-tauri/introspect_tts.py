import torch
from functools import partial
from TTS.api import TTS
torch.load = partial(torch.load, weights_only=False)
tts = TTS("tts_models/multilingual/multi-dataset/xtts_v2").to("cpu")
print("Attributes of TTS object:")
print(dir(tts))
