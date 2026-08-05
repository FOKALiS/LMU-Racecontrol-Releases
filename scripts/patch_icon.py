"""Patcht das Icon direkt in die EXE mit pefile (uerspringt Tauri)."""
import pefile
import struct
import os
import shutil

def patch_exe_icon(exe_path, ico_path, out_path=None):
    """Setzt das Icon einer EXE auf eine ICO-Datei."""
    if out_path is None:
        out_path = exe_path
    
    # Lese ICO-Datei
    with open(ico_path, 'rb') as f:
        ico_data = f.read()
    
    # ICO parsen
    if ico_data[:2] != b'\x00\x00' or ico_data[2:4] != b'\x01\x00':
        raise ValueError("Keine gueltige ICO-Datei")
    
    count = struct.unpack('<H', ico_data[4:6])[0]
    print(f"ICO: {count} Bilder")
    
    # Extrahiere alle Bilder
    images = []
    for i in range(count):
        entry = ico_data[6 + i*16 : 6 + (i+1)*16]
        w = entry[0]
        h = entry[1]
        if w == 0: w = 256
        if h == 0: h = 256
        size = struct.unpack('<I', entry[8:12])[0]
        offset = struct.unpack('<I', entry[12:16])[0]
        img_data = ico_data[offset:offset+size]
        images.append((w, h, img_data))
        print(f"  Bild {i}: {w}x{h} ({size} bytes)")
    
    # PE-Datei laden
    pe = pefile.PE(exe_path, fast_load=True)
    
    # RT_ICON Ressourcen finden
    icon_resources = []
    if hasattr(pe, 'DIRECTORY_ENTRY_RESOURCE'):
        for entry in pe.DIRECTORY_ENTRY_RESOURCE.entries:
            if entry.id == 3:  # RT_ICON
                for icon_entry in entry.directory.entries:
                    icon_resources.append(icon_entry)
    
    print(f"RT_ICON Ressourcen in EXE: {len(icon_resources)}")
    
    # RT_GROUP_ICON finden
    group_icon = None
    if hasattr(pe, 'DIRECTORY_ENTRY_RESOURCE'):
        for entry in pe.DIRECTORY_ENTRY_RESOURCE.entries:
            if entry.id == 14:  # RT_GROUP_ICON
                group_icon = entry
    
    print(f"RT_GROUP_ICON in EXE: {group_icon is not None}")
    
    # Da pefile das Schreiben von Ressourcen unterstuetzt, aber komplex ist,
    # verwenden wir einen einfacheren Ansatz: rcedit via npm
    # Stattdessen kopieren wir die EXE und verwenden PowerShell mit Add-Type
    print("\nVerwende alternativen Ansatz: rcedit via npm...")
    
    # npx rcedit ist verfuegbar
    import subprocess
    cmd = [
        'npx', '--yes', 'rcedit',
        exe_path,
        '--set-icon', ico_path
    ]
    result = subprocess.run(cmd, capture_output=True, text=True)
    print(f"rcedit exit: {result.returncode}")
    if result.stdout:
        print(f"stdout: {result.stdout}")
    if result.stderr:
        print(f"stderr: {result.stderr}")
    
    return result.returncode == 0

# Patch die EXE
exe = 'src-tauri/target/release/lmu-race-control.exe'
ico = 'src-tauri/icons/icon.ico'

# Backup
backup = exe + '.bak'
if os.path.exists(backup):
    os.remove(backup)
shutil.copy2(exe, backup)
print(f"Backup: {backup}")

success = patch_exe_icon(exe, ico)
if success:
    print("\nIcon erfolgreich gepatcht!")
else:
    print("\nIcon-Patch fehlgeschlagen!")