"""Erstellt eine BMP-basierte .ico fuer maximale Windows-Kompatibilitaet."""
import struct
import io
import os
from PIL import Image

# Lade Quelle
src = "C:/Users/Administrator/Documents/AI/Software Entwicklung/LMU Racecontrol/Logo/LMU RC - Icon transparent.png"
img = Image.open(src).convert("RGBA")
print(f"Quelle: {img.size}")

# ICO-Groessen (Windows Standard)
ico_sizes = [16, 24, 32, 48, 64, 128, 256]

# BMP-Daten fuer jede Groesse generieren
bmp_data_list = []
for size in ico_sizes:
    if size == 256:
        resized = img.resize((256, 256), Image.LANCZOS)
    else:
        resized = img.resize((size, size), Image.LANCZOS)
    
    # BMP mit Alpha-Kanal (BGRA)
    w, h = resized.size
    pixels = list(resized.getdata())
    
    # BMP-DIB Header (BITMAPINFOHEADER)
    # 40 Bytes Header
    # + 4 Bytes pro Pixel (BGRA)
    # + Zeilen auf 4 Bytes aligned
    row_size = w * 4
    row_padding = (4 - (row_size % 4)) % 4
    dib_size = 40 + (row_size + row_padding) * h
    
    # AND-Maske (1 Bit pro Pixel, keine)
    and_mask_size = ((w + 31) // 32) * 4 * h
    
    header = struct.pack('<IiiHHIIiiII',
        40,              # DIB Header Size
        w,               # Width
        h * 2,           # Height (doppelt fuer ICO)
        1,               # Planes
        32,              # Bits per Pixel
        0,               # Compression (BI_RGB)
        (row_size + row_padding) * h,  # Image Size
        0, 0,            # XPels, YPels
        0,               # Colors Used
        0                # Important Colors
    )
    
    # Pixel-Daten (BGRA, bottom-up)
    pixel_data = bytearray()
    for y in range(h - 1, -1, -1):  # Bottom-up
        for x in range(w):
            r, g, b, a = pixels[y * w + x]
            pixel_data.extend([b, g, r, a])  # BGRA
        pixel_data.extend([0] * row_padding)  # Padding
    
    # AND-Maske (alles 0 = transparent)
    and_mask = bytearray(and_mask_size)
    
    bmp_data_list.append(header + bytes(pixel_data) + and_mask)

# ICO-Datei schreiben
path = "src-tauri/icons/icon.ico"
with open(path, "wb") as f:
    # ICO Header
    f.write(struct.pack("<HHH", 0, 1, len(ico_sizes)))
    
    # Directory + Data
    offset = 6 + 16 * len(ico_sizes)
    for i, size in enumerate(ico_sizes):
        bmp_data = bmp_data_list[i]
        w = 0 if size == 256 else size
        h = 0 if size == 256 else size
        f.write(struct.pack("<BBBBHHII", w, h, 0, 0, 1, 32, len(bmp_data), offset))
        offset += len(bmp_data)
    
    for bmp_data in bmp_data_list:
        f.write(bmp_data)

print(f"icon.ico erstellt: {os.path.getsize(path)} Bytes")
print(f"Groessen: {ico_sizes}")
print("Format: BMP-basiert (maximale Windows-Kompatibilitaet)")