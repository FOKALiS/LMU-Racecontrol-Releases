"""Prueft alle Icon-Dateien und erstellt ICO mit allen Windows-Skalierungsgroessen."""
from PIL import Image
import os

# 1. ICO analysieren
ico = Image.open('src-tauri/icons/icon.ico')
print('=== ICON.ICO Analyse ===')
print(f'Datei: {os.path.getsize("src-tauri/icons/icon.ico")} Bytes')
print(f'Enthaltene Groessen: {sorted(ico.info.get("sizes", []))}')
print()

# 2. PNGs analysieren
for name in ['32x32.png', '128x128.png', '128x128@2x.png', 'icon.png']:
    path = f'src-tauri/icons/{name}'
    if os.path.exists(path):
        img = Image.open(path)
        print(f'{name}: {img.size[0]}x{img.size[1]}, {os.path.getsize(path)} Bytes')
print()

# 3. ICO mit allen Windows-Skalierungsgroessen neu erstellen
ico_sizes = [16, 20, 24, 32, 40, 48, 64, 96, 128, 256]
img = Image.open('C:/Users/Administrator/Documents/AI/Software Entwicklung/LMU Racecontrol/Logo/LMU RC - Icon transparent.png').convert('RGBA')
img.save('src-tauri/icons/icon.ico', format='ICO', sizes=[(s, s) for s in ico_sizes], append_images=[img.resize((s, s), Image.LANCZOS) for s in ico_sizes[1:]])
print(f'ICO neu erstellt: {os.path.getsize("src-tauri/icons/icon.ico")} Bytes')
print(f'Groessen: {sorted(Image.open("src-tauri/icons/icon.ico").info.get("sizes", []))}')