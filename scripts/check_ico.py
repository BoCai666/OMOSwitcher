from PIL import Image
import os

ico_path = r"E:\AI\Programs\OMOSwitcher\src-tauri\icons\icon.ico"
img = Image.open(ico_path)
print(f"Current ICO sizes: {img.info.get('sizes', [])}")
print(f"File size: {os.path.getsize(ico_path)} bytes")
