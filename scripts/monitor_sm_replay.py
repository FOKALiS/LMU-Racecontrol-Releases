"""
Überwacht Shared Memory 'LMU_Data' VORHER/NACHHER (reliabel mit string_at).
Starte LMU + Session, dann dieses Skript.
Dann klicke in BCUK auf einen Vorfall (Zeitsprung).
"""
import struct
import ctypes
import sys

LMU_DATA_NAME = "LMU_Data"
FILE_MAP_ALL_ACCESS = 0x000F001F
FILE_MAP_READ = 0x0004

kernel32 = ctypes.windll.kernel32

handle = kernel32.OpenFileMappingW(FILE_MAP_ALL_ACCESS, False, LMU_DATA_NAME)
if not handle:
    handle = kernel32.OpenFileMappingW(FILE_MAP_READ, False, LMU_DATA_NAME)
    if not handle:
        print("❌ Shared Memory 'LMU_Data' nicht gefunden. Läuft LMU?")
        sys.exit(1)

ptr = kernel32.MapViewOfFile(handle, FILE_MAP_ALL_ACCESS, 0, 0, 4096)
if not ptr:
    ptr = kernel32.MapViewOfFile(handle, FILE_MAP_READ, 0, 0, 4096)
    if not ptr:
        print("❌ MapViewOfFile fehlgeschlagen")
        kernel32.CloseHandle(handle)
        sys.exit(1)

print("✅ Shared Memory 'LMU_Data' geöffnet!")
print()

# Ersten Snapshot (4096 Bytes)
print("Erstelle Referenz-Snapshot (4096 Bytes)...")
old_bytes = ctypes.string_at(ptr, 4096)
print(f"Referenz erstellt: {len(old_bytes)} Bytes")
print()
print("⏳ Jetzt in BCUK auf einen Vorfall klicken (Zeitsprung),")
print("   dann ENTER drücken...")
print()

input("ENTER drücken, wenn BCUK den Zeitsprung gemacht hat...")

# Zweiten Snapshot
new_bytes = ctypes.string_at(ptr, 4096)

# Differenz suchen (4-Byte-Werte)
print("\n🔍 Suche nach Unterschieden...")
changes = []
for i in range(0, 4096, 4):
    if old_bytes[i:i+4] != new_bytes[i:i+4]:
        old_val = struct.unpack("<I", old_bytes[i:i+4])[0]
        new_val = struct.unpack("<I", new_bytes[i:i+4])[0]
        changes.append((i, old_val, new_val))

if changes:
    print(f"\n✅ {len(changes)} Änderungen gefunden!")
    print()
    for offset, old_val, new_val in changes[:30]:
        old_f = struct.unpack("<f", struct.pack("<I", old_val))[0]
        new_f = struct.unpack("<f", struct.pack("<I", new_val))[0]
        print(f"  Offset 0x{offset:04X} ({offset:5d}): {old_val:10d} ({old_f:8.1f}) → {new_val:10d} ({new_f:8.1f})")
else:
    print("\n❌ Keine Änderungen in ersten 4096 Bytes!")
    print("BCUK verwendet vielleicht Tastatur oder Windows Messages.")

# Aufräumen
kernel32.UnmapViewOfFile(ptr)
kernel32.CloseHandle(handle)
print("\nFertig.")