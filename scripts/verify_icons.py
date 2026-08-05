"""Extrahiert Icons aus EXE und Installer und speichert sie als PNG zum Vergleich."""
import struct
import os
from PIL import Image

def extract_ico_from_exe(exe_path, out_path):
    """Extrahiert die groesste ICO-Ressource aus einer EXE."""
    with open(exe_path, 'rb') as f:
        data = f.read()
    
    # Finde alle ICO-Header (00 00 01 00)
    # Die ICO-Ressourcen sind in PE-Dateien als RT_ICON eingebettet
    # Suche nach ICONDIR-Header
    ico_positions = []
    for i in range(len(data) - 4):
        if data[i:i+2] == b'\x00\x00' and data[i+2:i+4] == b'\x01\x00':
            # Pruefe ob an dieser Position ein ICO-Header ist
            count = int.from_bytes(data[i+4:i+6], 'little')
            if 1 <= count <= 20:
                ico_positions.append((i, count))
    
    print(f"ICO-Header gefunden in {exe_path}: {len(ico_positions)}")
    
    # Nimm die groesste ICO (letzte Position)
    if not ico_positions:
        return False
    
    pos, count = ico_positions[-1]
    print(f"  Letzte ICO: {count} Bilder bei Offset {pos}")
    
    # Lese alle Bilder
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
        img_offset = int.from_bytes(data[entry_offset + 12:entry_offset + 16], 'little')
        entries.append((w, h, size, img_offset))
    
    print(f"  Bilder: {[(w, h) for w, h, _, _ in entries]}")
    
    # Finde die groesste Version
    largest = max(entries, key=lambda e: e[0])
    w, h, size, img_offset = largest
    
    # Wenn es ein PNG ist, extrahiere direkt
    if data[img_offset:img_offset+8] == b'\x89PNG\r\n\x1a\n':
        png_data = data[img_offset:img_offset+size]
        with open(out_path, 'wb') as out:
            out.write(png_data)
        print(f"  PNG {w}x{h} extrahiert: {out_path}")
        return True
    
    # Wenn es BMP ist, konvertiere zu PNG
    if data[img_offset:img_offset+2] == b'BM':
        # BMP-Daten
        bmp_data = data[img_offset:img_offset+size]
        # DIB Header: 40 Bytes
        dib_size = int.from_bytes(bmp_data[0:4], 'little')
        if dib_size >= 40:
            img_w = int.from_bytes(bmp_data[4:8], 'little', signed=True)
            img_h = int.from_bytes(bmp_data[8:12], 'little', signed=True)
            bpp = int.from_bytes(bmp_data[14:16], 'little')
            print(f"  BMP: {img_w}x{img_h} bpp={bpp}")
            
            # Fuer 32-bit BMP mit Alpha
            if bpp == 32:
                rows = []
                row_size = img_w * 4
                pixel_data_start = img_offset + len(bmp_data) - (row_size * img_h)
                # Simpler: extrahiere die Pixel aus dem BMP
                # BMP ist bottom-up
                for row in range(img_h):
                    row_data = []
                    for col in range(img_w):
                        idx = pixel_data_start + row * row_size + col * 4
                        if idx + 3 < len(data):
                            b, g, r, a = data[idx], data[idx+1], data[idx+2], data[idx+3]
                            row_data.append((r, g, b, a))
                    rows.append(row_data)
                
                # BMP ist bottom-up, also umdrehen
                rows.reverse()
                
                img = Image.new('RGBA', (img_w, img_h))
                pixels = img.load()
                for y in range(img_h):
                    for x in range(img_w):
                        pixels[x, y] = rows[y][x]
                
                img.save(out_path, 'PNG')
                print(f"  BMP {img_w}x{img_h} zu PNG konvertiert: {out_path}")
                return True
    
    print("  Konnte kein Icon extrahieren")
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