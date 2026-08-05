"""Extrahiert alle Groessen aus der icon.ico und speichert sie als PNG."""
import os
from PIL import Image

SRC = "src-tauri/icons/icon.ico"
OUT_DIR = "C:/Users/Administrator/Desktop/icon_check"

os.makedirs(OUT_DIR, exist_ok=True)

ico = Image.open(SRC)
sizes = sorted(ico.info.get("sizes", []))
print(f"ICO enthaelt Groessen: {sizes}")

for i, (w, h) in enumerate(sizes):
    ico.seek(i)
    img = ico.copy()
    out = os.path.join(OUT_DIR, f"icon_{w}x{h}.png")
    img.save(out, "PNG")
    print(f"  {w}x{h} -> {out}")

print(f"Fertig! Alle Groessen in {OUT_DIR}")