"""
Shared Memory Diff für LMU_Data
Findet die Offsets, die BCUK beim Kamerawechsel beschreibt.

Verwendung:
1. LMU MUSS LAUFEN (mit aktiver Session!)
2. python scripts/sm_diff.py
3. In BCUK eine Kamera-Taste drücken
4. Enter drücken -> zeigt Änderungen
"""

import ctypes
import struct
import sys

SM_NAME = "LMU_Data"
SM_SIZE = 4096

def main():
    print("=" * 50)
    print("  LMU Shared Memory Diff")
    print("=" * 50)
    print()
    print("  1. LMU MUSS LAUFEN!")
    print("  2. BCUK muss verbunden sein")
    print()

    # Öffne Shared Memory
    kernel32 = ctypes.windll.kernel32
    handle = kernel32.OpenFileMappingW(0x000F001F, False, SM_NAME)
    
    if not handle:
        print("❌ Konnte LMU_Data Shared Memory NICHT öffnen!")
        print()
        print("   Mögliche Ursachen:")
        print("   - LMU läuft nicht? (starten und Watch-Modus aktivieren)")
        print("   - LMU Shared Memory heißt anders?")
        print("   - Berechtigungsproblem? (als Admin ausführen)")
        print()
        input("   Enter drücken zum Beenden...")
        return

    # Mappe Shared Memory
    ptr = kernel32.MapViewOfFile(handle, 0x000F001F, 0, 0, SM_SIZE)
    if not ptr:
        kernel32.CloseHandle(handle)
        print("❌ Konnte Shared Memory nicht mappen!")
        input("   Enter drücken zum Beenden...")
        return

    try:
        # Ersten Snapshot lesen
        buf1 = (ctypes.c_ubyte * SM_SIZE).from_address(ptr)
        before = bytes(buf1)
        print(f"✅ Snapshot 1: {len(before)} Bytes gelesen")
        print()
        print("   Jetzt in BCUK eine KAMERA-TASTE drücken!")
        print("   (z.B. TV Cycle oder Onboard)")
        print("   Danach hier Enter drücken...")
        sys.stdout.flush()
        input()

        # Zweiten Snapshot lesen
        buf2 = (ctypes.c_ubyte * SM_SIZE).from_address(ptr)
        after = bytes(buf2)
        print(f"✅ Snapshot 2: {len(after)} Bytes gelesen")
        print()
        print("   Suche nach Änderungen...")
        print()

        # Vergleiche Byte-für-Byte
        changes_found = False
        
        # 1-Byte Änderungen
        changes_1byte = []
        for i in range(SM_SIZE):
            if before[i] != after[i]:
                changes_1byte.append((i, before[i], after[i]))
        
        if changes_1byte:
            changes_found = True
            print(f"  === 1-Byte Änderungen ({len(changes_1byte)}) ===")
            for offset, old, new in changes_1byte[:30]:
                print(f"   0x{offset:04X}: {old:3d} -> {new:3d}")
        
        # 2-Byte (16-Bit) Änderungen  
        changes_2byte = []
        for i in range(0, SM_SIZE - 1, 2):
            old_val = struct.unpack('<H', before[i:i+2])[0]
            new_val = struct.unpack('<H', after[i:i+2])[0]
            if old_val != new_val:
                changes_2byte.append((i, old_val, new_val))
        
        if changes_2byte:
            changes_found = True
            print(f"\n  === 2-Byte Änderungen ({len(changes_2byte)}) ===")
            for offset, old, new in changes_2byte[:30]:
                delta = new - old
                print(f"   0x{offset:04X}: {old:6d} -> {new:6d} (Δ {delta:+d})")
        
        # 4-Byte (32-Bit) Änderungen
        changes_4byte = []
        for i in range(0, SM_SIZE - 3, 4):
            old_val = struct.unpack('<I', before[i:i+4])[0]
            new_val = struct.unpack('<I', after[i:i+4])[0]
            if old_val != new_val:
                changes_4byte.append((i, old_val, new_val))
        
        if changes_4byte:
            changes_found = True
            print(f"\n  === 4-Byte Änderungen ({len(changes_4byte)}) ===")
            for offset, old, new in changes_4byte[:30]:
                delta = new - old
                print(f"   0x{offset:04X}: {old:10d} -> {new:10d} (Δ {delta:+d})")
        
        if not changes_found:
            print("❌ Keine Änderungen gefunden!")
            print("   War die Kamera-Taste wirklich aktiv?")
            print("   Hat BCUK sich verbinden können?")
        
    finally:
        kernel32.UnmapViewOfFile(ptr)
        kernel32.CloseHandle(handle)
    
    print()
    input("   Enter drücken zum Beenden...")

if __name__ == "__main__":
    main()