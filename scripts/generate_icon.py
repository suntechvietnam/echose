#!/usr/bin/env python3
"""Generate Windows icon.ico from icon.png"""
from PIL import Image
import sys
import os

def main():
    icon_png = "src-tauri/icons/icon.png"
    icon_ico = "src-tauri/icons/icon.ico"
    
    if not os.path.exists(icon_png):
        print(f"Error: {icon_png} not found")
        sys.exit(1)
    
    try:
        img = Image.open(icon_png)
        sizes = [(256, 256), (128, 128), (64, 64), (48, 48), (32, 32), (16, 16)]
        img.save(icon_ico, format='ICO', sizes=sizes)
        
        if os.path.exists(icon_ico):
            size = os.path.getsize(icon_ico)
            print(f"✓ Successfully created {icon_ico} ({size} bytes)")
            sys.exit(0)
        else:
            print(f"Error: {icon_ico} was not created")
            sys.exit(1)
    except Exception as e:
        print(f"Error: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()

