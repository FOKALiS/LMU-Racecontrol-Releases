"""
Dump der VehicleTelemetry-Struktur aus dem LMU Shared Memory.
Ziel: Finden der Offsets für impactET, impactMag, impactPos.
"""
import ctypes
import ctypes.wintypes
import time
import sys

OpenFileMappingW = ctypes.windll.kernel32.OpenFileMappingW
MapViewOfFile = ctypes.windll.kernel32.MapViewOfFile
UnmapViewOfFile = ctypes.windll.kernel32.UnmapViewOfFile
CloseHandle = ctypes.windll.kernel32.CloseHandle

FILE_MAP_READ = 0x0004
FILE_MAP_ALL_ACCESS = 0x000F001F

# Offsets aus read_lmu_sm.py
SCORING_OFFSET = 1632
VEHICLE_SCORING_OFFSET = 2192
VEHICLE_SCORING_SIZE = 584
TELEMETRY_OFFSET = 128464
VEHICLE_TELEMETRY_OFFSET = 128468
VEHICLE_TELEMETRY_SIZE = 1888

def open_shared_memory(name="LMU_Data", size=8*1024*1024):
    wide_name = ctypes.create_unicode_buffer(name)
    handle = OpenFileMappingW(FILE_MAP_ALL_ACCESS, False, wide_name)
    if not handle:
        handle = OpenFileMappingW(FILE_MAP_READ, False, wide_name)
        if not handle:
            return None, None
    ptr = MapViewOfFile(handle, FILE_MAP_READ, 0, 0, size)
    if not ptr:
        CloseHandle(handle)
        return None, None
    return handle, ptr

def read_f32(ptr, offset):
    return ctypes.c_float.from_address(ptr + offset).value

def read_i32(ptr, offset):
    return ctypes.c_int32.from_address(ptr + offset).value

def read_u32(ptr, offset):
    return ctypes.c_uint32.from_address(ptr + offset).value

def read_string(ptr, offset, max_len=64):
    chars = []
    for i in range(0, max_len, 2):
        c = ctypes.c_uint16.from_address(ptr + offset + i).value
        if c == 0:
            break
        chars.append(chr(c))
    return ''.join(chars)

def main():
    sys.stdout.reconfigure(encoding='utf-8', errors='replace')
    
    print("=" * 70)
    print("  LMU SHARED MEMORY – VEHICLE TELEMETRY DUMP")
    print("=" * 70)
    
    handle, ptr = open_shared_memory()
    if not ptr:
        print("\n❌ Konnte LMU_Data nicht öffnen!")
        print("  Starte CMD als Administrator!")
        input("ENTER zum Beenden...")
        return
    
    print("\n✅ LMU_Data geöffnet!")
    
    # 1. Scoring Info auslesen
    num_vehicles = read_i32(ptr, SCORING_OFFSET + 24)
    print(f"\n  Anzahl Fahrzeuge (Scoring): {num_vehicles}")
    
    # 2. List der Fahrzeuge mit Namen
    print(f"\n{'='*70}")
    print("  FAHRZEUGE")
    print(f"{'='*70}")
    
    for i in range(min(num_vehicles, 30)):
        voffset = VEHICLE_SCORING_OFFSET + i * VEHICLE_SCORING_SIZE
        name = read_string(ptr, voffset, 32)
        car_num = read_string(ptr, voffset + 192, 8)
        speed = read_f32(ptr, voffset + 312)
        place = read_i32(ptr, voffset + 332)
        print(f"  #{i}: P{place} #{car_num} {name} speed={speed:.1f}")
    
    # 3. Telemetry-Header prüfen
    print(f"\n{'='*70}")
    print("  TELEMETRY HEADER (Offset 0x{TELEMETRY_OFFSET:X})")
    print(f"{'='*70}")
    
    # Dump der ersten 64 Bytes des Telemetry-Headers
    for i in range(0, 64, 4):
        val = read_i32(ptr, TELEMETRY_OFFSET + i)
        val_f = read_f32(ptr, TELEMETRY_OFFSET + i)
        print(f"  +0x{i:02X}: i32={val:10d}  f32={val_f:+.6f}")
    
    # 4. VehicleTelemetry dumpen (erstes Fahrzeug)
    print(f"\n{'='*70}")
    print("  VEHICLE TELEMETRY [0] (Offset 0x{VEHICLE_TELEMETRY_OFFSET:X})")
    print(f"  Strukturgröße: {VEHICLE_TELEMETRY_SIZE} Bytes")
    print(f"{'='*70}")
    
    base = VEHICLE_TELEMETRY_OFFSET + 0 * VEHICLE_TELEMETRY_SIZE
    
    # Erste 200 Bytes komplett dumpen
    print(f"\n  --- Erste 200 Bytes ---")
    for i in range(0, 200, 4):
        val_i = read_i32(ptr, base + i)
        val_f = read_f32(ptr, base + i)
        print(f"  +0x{i:03X}: i32={val_i:10d}  f32={val_f:+.6f}")
    
    # 5. Live-Überwachung: Scanne nach impactMag > 0
    print(f"\n{'='*70}")
    print("  LIVE-ÜBERWACHUNG (Impact-Suche, 30 Sekunden)")
    print(f"{'='*70}")
    print("  Provoziere einen Crash in LMU!")
    print()
    
    try:
        for t in range(15):
            # Scanne alle Fahrzeuge nach impactMag > 0
            found = False
            for veh in range(min(num_vehicles, 30)):
                veh_base = VEHICLE_TELEMETRY_OFFSET + veh * VEHICLE_TELEMETRY_SIZE
                # Prüfe verschiedene Offsets für impactMag
                for off in [0, 4, 8, 12, 16, 20, 24, 28, 32, 36, 40, 48, 60, 64, 68, 72, 76, 80, 84, 88, 92, 96, 100]:
                    mag = read_f32(ptr, veh_base + off)
                    if abs(mag) > 0.5:
                        found = True
                        print(f"  🚨 VEHKEL {veh}: impactMag={mag:.3f} an Offset 0x{veh_base+off:X}")
                        # Zeige den Kontext
                        print(f"     Kontext (0x{veh_base:X}):")
                        for j in range(max(0, off-8), min(off+12, 200), 4):
                            v = read_f32(ptr, veh_base + j)
                            vi = read_i32(ptr, veh_base + j)
                            marker = " ← impactMag" if j == off else ""
                            print(f"     +0x{j:03X}: f32={v:+.6f} i32={vi}{marker}")
            if not found:
                print(f"  [{t+1}/15] Keine Einschläge erkannt (warte 2s)...")
            time.sleep(2)
    except KeyboardInterrupt:
        print("\n  Abbruch")
    
    if ptr:
        UnmapViewOfFile(ptr)
    if handle:
        CloseHandle(handle)
    print("\n  Aufgeräumt.")

if __name__ == "__main__":
    main()