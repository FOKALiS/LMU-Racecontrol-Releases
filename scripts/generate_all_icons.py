"""Kompletter Icon-Generator: Erstellt alle Icons mit LMU RC Logo + BMP-basierte ICO."""
import os
from PIL import Image

BASE = "src-tauri/icons"
SRC = "C:/Users/Administrator/Documents/AI/Software Entwicklung/LMU Racecontrol/Logo/LMU RC - Icon transparent.png"
LOGO = "C:/Users/Administrator/Documents/AI/Software Entwicklung/LMU Racecontrol/Logo/LMU RC Logo - hell.png"

# Lade Quelle
img = Image.open(SRC).convert("RGBA")
print(f"Quelle: {img.size}")

# 1. PNGs generieren
sizes = {
    "32x32.png": 32,
    "64x64.png": 64,
    "128x128.png": 128,
    "128x128@2x.png": 256,
    "icon.png": 512,
}
for name, size in sizes.items():
    path = f"{BASE}/{name}"
    resized = img.resize((size, size), Image.LANCZOS)
    resized.save(path, "PNG")
    print(f"  {name}: {size}x{size}")

# 2. BMP-basierte ICO generieren (372KB, Windows-nativ)
ico_sizes = [16, 24, 32, 48, 64, 128, 256]
import struct

bmp_data_list = []
for size in ico_sizes:
    resized = img.resize((size, size), Image.LANCZOS)
    w, h = resized.size
    pixels = list(resized.getdata())
    
    row_size = w * 4
    row_padding = (4 - (row_size % 4)) % 4
    
    header = struct.pack('<IiiHHIIiiII',
        40, w, h * 2, 1, 32, 0,
        (row_size + row_padding) * h, 0, 0, 0, 0)
    
    pixel_data = bytearray()
    for y in range(h - 1, -1, -1):
        for x in range(w):
            r, g, b, a = pixels[y * w + x]
            pixel_data.extend([b, g, r, a])
        pixel_data.extend([0] * row_padding)
    
    and_mask_size = ((w + 31) // 32) * 4 * h
    bmp_data_list.append(header + bytes(pixel_data) + bytearray(and_mask_size))

ico_path = f"{BASE}/icon.ico"
with open(ico_path, "wb") as f:
    f.write(struct.pack("<HHH", 0, 1, len(ico_sizes)))
    offset = 6 + 16 * len(ico_sizes)
    for i, size in enumerate(ico_sizes):
        bmp_data = bmp_data_list[i]
        w = 0 if size == 256 else size
        h = 0 if size == 256 else size
        f.write(struct.pack("<BBBBHHII", w, h, 0, 0, 1, 32, len(bmp_data), offset))
        offset += len(bmp_data)
    for bmp_data in bmp_data_list:
        f.write(bmp_data)

print(f"  icon.ico: {os.path.getsize(ico_path)} Bytes (BMP-basiert)")

# 3. Logo kopieren
logo = Image.open(LOGO).convert("RGBA")
logo.save("frontend/public/logo.png", "PNG")
print(f"  logo.png: {logo.size}")

print("\nFERTIG! Alle Icons mit LMU RC Logo wurden generiert.")