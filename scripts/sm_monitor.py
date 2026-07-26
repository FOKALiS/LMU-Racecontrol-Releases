import sys, os, struct

print("=" * 60)
print("  LMU Shared Memory Monitor v2")
print("=" * 60)
print()
sys.stdout.flush()

try:
    import ctypes
    import ctypes.wintypes
    print("✅ ctypes geladen")
    sys.stdout.flush()
except Exception as e:
    print(f"❌ ctypes Fehler: {e}")
    input("Enter zum Beenden...")
    sys.exit(1)

SM_NAMES = ["LMU_Data", "LMU_SharedMemory", "rFactor2_Data", 
            "rFactor2_SharedMemory", "Local\\LMU_Data"]
SM_SIZE = 4096

kernel32 = ctypes.windll.kernel32

for name in SM_NAMES:
    print(f"  Suche Shared Memory: '{name}'...")
    sys.stdout.flush()
    try:
        handle = kernel32.OpenFileMappingW(0x000F001F, False, name)
        if handle:
            print(f"  ✅ GEFUNDEN: '{name}' (Handle: {handle})")
            sys.stdout.flush()
            
            ptr = kernel32.MapViewOfFile(handle, 0x000F001F, 0, 0, SM_SIZE)
            if not ptr:
                print(f"  ❌ MapViewOfFile fehlgeschlagen für '{name}'")
                kernel32.CloseHandle(handle)
                continue
            
            buf = (ctypes.c_ubyte * SM_SIZE).from_address(ptr)
            data = bytes(buf)
            print(f"  ✅ Shared Memory gelesen: {len(data)} Bytes")
            print()
            print("  Hex-Dump (erste 256 Bytes):")
            print()
            for row in range(0, 256, 16):
                hex_part = " ".join(f"{data[row+i]:02X}" for i in range(16))
                ascii_part = "".join(chr(data[row+i]) if 32 <= data[row+i] < 127 else "." for i in range(16))
                print(f"  {row:04X}: {hex_part}  |{ascii_part}|")
            sys.stdout.flush()
            
            # Speichere Snapshot
            before = data
            
            print()
            print("  Jetzt in BCUK eine KAMERA-TASTE drücken!")
            print("  Danach hier Enter drücken...")
            sys.stdout.flush()
            input()
            
            # Zweiter Snapshot
            after = bytes(buf)
            
            # Vergleiche
            changes = []
            for i in range(SM_SIZE):
                if before[i] != after[i]:
                    changes.append((i, before[i], after[i]))
            
            if not changes:
                print("❌ Keine Änderungen gefunden!")
                print("  Wurdest du in BCUK eine Kamera-Taste gedrückt?")
            else:
                print(f"\n✅ {len(changes)} Bytes geändert!")
                print()
                for offset, old, new in changes[:40]:
                    delta = new - old
                    print(f"  0x{offset:04X}: {old:3d} -> {new:3d} (Δ {delta:+d})")
                if len(changes) > 40:
                    print(f"  ... und {len(changes) - 40} weitere Änderungen")
            
            kernel32.UnmapViewOfFile(ptr)
            kernel32.CloseHandle(handle)
            break
        else:
            print(f"  ❌ Nicht gefunden")
            sys.stdout.flush()
    except Exception as e:
        print(f"  ❌ Fehler bei '{name}': {e}")
        sys.stdout.flush()
else:
    print()
    print("❌ KEIN Shared Memory gefunden!")
    print()
    print("  Mögliche Ursachen:")
    print("  1. LMU läuft nicht (starten + Watch-Modus aktivieren)")
    print("  2. Keine Berechtigung (als Admin ausführen)")
    print("  3. BCUK hat den Speicher noch nicht initialisiert")

print()
input("  Enter zum Beenden...")