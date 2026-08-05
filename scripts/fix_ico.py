"""Erstellt icon.ico mit allen Windows-Groessen (16-256px) aus icon-source.png."""
import struct
import io
import os
from PIL import Image

# Lade die Quelle
img = Image.open("src-tauri/icons/icon-source.png")

# ICO-Groessen
ico_sizes = [16, 24, 32, 48, 64, 128, 256]

# PNG-Daten fuer jede Groesse generieren
png_data_list = []
for size in ico_sizes:
    resized = img.resize((size, size), Image.LANCZOS)
    png_bytes = io.BytesIO()
    resized.save(png_bytes, format="PNG")
    png_data_list.append(png_bytes.getvalue())

# ICO-Datei schreiben
path = "src-tauri/icons/icon.ico"
with open(path, "wb") as f:
    # ICO Header: reserved=0, type=1 (ICO), count
    f.write(struct.pack("<HHH", 0, 1, len(ico_sizes)))

    # Directory entries + Image data
    offset = 6 + 16 * len(ico_sizes)
    for i, size in enumerate(ico_sizes):
        png_data = png_data_list[i]
        # Directory entry: width, height, colors, reserved, planes, bpp, size, offset
        w = 0 if size == 256 else size
        h = 0 if size == 256 else size
        f.write(struct.pack("<BBBBHHII", w, h, 0, 0, 1, 32, len(png_data), offset))
        offset += len(png_data)

    # Alle PNG-Daten schreiben
    for png_data in png_data_list:
        f.write(png_data)

print(f"icon.ico erstellt: {os.path.getsize(path)} Bytes")
print(f"Groessen: {ico_sizes}")