"""Prueft ob die gebaute EXE das richtige Icon enthaelt."""
exe_path = 'src-tauri/target/release/lmu-race-control.exe'

with open(exe_path, 'rb') as f:
    data = f.read()

print(f"EXE-Groesse: {len(data)} Bytes")

# Suche nach icon.ico Referenz
if b'icon.ico' in data:
    print("icon.ico Referenz in EXE gefunden")
else:
    print("KEINE icon.ico Referenz in EXE")

# Suche nach ICO-Header (00 00 01 00)
ico_count = data.count(b'\x00\x00\x01\x00')
print(f"ICO-Header gefunden: {ico_count} mal")

# Suche nach PNG-Header (innerhalb von ICOs)
png_count = data.count(b'\x89PNG')
print(f"PNG-Header gefunden: {png_count} mal")

# Suche nach RT_ICON Group (Resourcen)
if b'RT_ICON' in data:
    print("RT_ICON Ressource gefunden")
else:
    print("KEINE RT_ICON Ressource")

# Suche nach dem Standard-Tauri-Icon (Tauri verwendet oft ein Default-Icon)
if b'TAURI' in data[:1000]:
    print("Tauri-Signatur gefunden")