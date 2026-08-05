"""Extrahiert alle Icon-Groessen aus der gebauten EXE."""
import re

exe = 'src-tauri/target/release/lmu-race-control.exe'
with open(exe, 'rb') as f:
    data = f.read()

print(f"EXE-Groesse: {len(data)} Bytes")

# Finde alle PNG-Header
pngs = [m.start() for m in re.finditer(b'\x89PNG', data)]
print(f"PNG-Header in EXE: {len(pngs)}")

# Jede PNG-Groesse auslesen
for i, pos in enumerate(pngs):
    # PNG Header: 8 Bytes, dann IHDR Chunk
    # IHDR: 4 Bytes Length (13), 4 Bytes "IHDR", dann 4+4 Bytes width/height
    chunk_len = int.from_bytes(data[pos+8:pos+12], 'big')
    chunk_type = data[pos+12:pos+16]
    if chunk_type == b'IHDR' and chunk_len >= 13:
        w = int.from_bytes(data[pos+16:pos+20], 'big')
        h = int.from_bytes(data[pos+20:pos+24], 'big')
        print(f"  PNG {i}: {w}x{h} bei Offset {pos}, Chunk: {chunk_type}")
    else:
        print(f"  PNG {i}: Chunk {chunk_type} Laenge {chunk_len} bei Offset {pos}")
