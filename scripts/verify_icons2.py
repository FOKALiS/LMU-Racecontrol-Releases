"""Extrahiert Icons aus EXE und Installer korrekt (Offsets relativ zum ICO-Start)."""
import os
from PIL import Image

def extract_ico_from_exe(exe_path, out_path):
    """Extrahiert die groesste ICO-Ressource aus einer EXE."""
    with open(exe_path, 'rb') as f:
        data = f.read()
    
    # Suche nach ICONDIR-Header: 00 00 01 00 XX XX (Reserved=0, Type=1, Count)
    ico_positions = []
    for i in range(len(data) - 6):
        if data[i] == 0 and data[i+1] == 0 and data[i+2] == 1 and data[i+3] == 0:
            count = int.from_bytes(data[i+4:i+6], 'little')
            if 1 <= count <= 20:
                ico_positions.append((i, count))
    
    print(f"ICO-Header gefunden: {len(ico_positions)}")
    if not ico_positions:
        return False
    
    # Nimm die ICO mit den meisten Bildern
    pos, count = max(ico_positions, key=lambda x: x[1])
    print(f"  ICO bei Offset {pos} mit {count} Bildern")
    
    # Lese alle Bilder (Offsets sind relativ zum ICO-Start)
    entries = []
    for j in range(count):
        entry_offset = pos + 6 + j * 16
        w = data[entry_offset]
        h = data[entry_offset + 1]
        if w == 0:
            w = 256
        if h == 0:
            h = 256
        size = int.from_bytes(data[entry_offset + 8:entry_offset + 12], 'little')
        img_offset_rel = int.from_bytes(data[entry_offset + 12:entry_offset + 16], 'little')
        img_offset = pos + img_offset_rel  # Relativ zum ICO-Start
        entries.append((w, h, size, img_offset))
    
    print(f"  Bilder: {sorted([(w, h) for w, h, _, _ in entries], key=lambda x: x[0])}")
    
    # Finde die groesste Version
    largest = max(entries, key=lambda e: e[0])
    w, h, size, img_offset = largest
    
    # Pruefe ob Daten vorhanden sind
    if img_offset + size > len(data):
        print(f"  FEHLER: Offset {img_offset} + Size {size} > Datei {len(data)}")
        return False
    
    # Extrahiere das Bild
    img_data = data[img_offset:img_offset+size]
    
    # PNG?
    if img_data[:8] == b'\x89PNG\r\n\x1a\n':
        with open(out_path, 'wb') as out:
            out.write(img_data)
        print(f"  PNG {w}x{h} extrahiert")
        return True
    
    # BMP?
    if img_data[:2] == b'BM':
        # DIB Header parsen
        dib_size = int.from_bytes(img_data[0:4], 'little')
        if dib_size >= 40:
            img_w = int.from_bytes(img_data[4:8], 'little', signed=True)
            img_h_raw = int.from_bytes(img_data[8:12], 'little', signed=True)
            img_h = abs(img_h_raw)
            bpp = int.from_bytes(img_data[14:16], 'little')
            print(f"  BMP: {img_w}x{img_h} bpp={bpp}")
            
            if bpp == 32:
                # Pixel-Daten: nach dem DIB-Header
                # Bei ICO: DIB-Header + Pixel + AND-Maske
                row_size = img_w * 4
                row_padding = (4 - (row_size % 4)) % 4
                pixel_start = 40  # DIB-Header
                img = Image.new('RGBA', (img_w, img_h))
                pixels = img.load()
                
                # BMP ist bottom-up
                for y in range(img_h - 1, -1, -1):
                    row_offset = pixel_start + (img_h - 1 - y) * (row_size + row_padding)
                    for x in range(img_w):
                        idx = row_offset + x * 4
                        if idx + 3 < len(img_data):
                            b, g, r, a = img_data[idx], img_data[idx+1], img_data[idx+2], img_data[idx+3]
                            pixels[x, y] = (r, g, b, a)
                
                img.save(out_path, 'PNG')
                print(f"  BMP {img_w}x{img_h} als PNG gespeichert")
                return True
    
    print(f"  Unbekanntes Format: {img_data[:8].hex()}")
    return False

# 1. EXE-Icon extrahieren
print("=== EXE ===")
extract_ico_from_exe(
    'src-tauri/target/release/lmu-race-control.exe',
    'C:/Users/Administrator/Desktop/verify_exe_icon.png'
)

# 2. Installer-Icon extrahieren
print("\n=== Installer ===")
extract_ico_from_exe(
    'src-tauri/target/release/bundle/nsis/LMU RACECONTROL_0.8.5_x64-setup.exe',
    'C:/Users/Administrator/Desktop/verify_installer_icon.png'
)

print("\nFertig! Bitte die beiden PNGs auf dem Desktop pruefen.")