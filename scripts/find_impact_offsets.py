"""
Findet die Shared Memory Offsets für impactET, impactMag, impactPos
durch Vergleich mit der SM Bridge JSON-Dump.

Die SM Bridge liest den LMU Shared Memory und streamt ihn als JSON.
Der Bridge-Dump zeigt die Feld-Reihenfolge. Daraus berechnen wir die Offsets.

VehicleTelemetry-Struktur (aus read_lmu_sm.py):
  Telemetry Header Offset: 0x1F5D0 (128464)
  VehTelem Array Offset:  0x1F5D4 (128468)
  VehicleTelemetry Size:   1888 Bytes pro Fahrzeug
"""
import ctypes
import ctypes.wintypes
import time
import struct

# Windows API
OpenFileMappingW = ctypes.windll.kernel32.OpenFileMappingW
MapViewOfFile = ctypes.windll.kernel32.MapViewOfFile
UnmapViewOfFile = ctypes.windll.kernel32.UnmapViewOfFile
CloseHandle = ctypes.windll.kernel32.CloseHandle

FILE_MAP_READ = 0x0004
FILE_MAP_ALL_ACCESS = 0x000F001F

# Offsets aus read_lmu_sm.py
TELEMETRY_HEADER_OFFSET = 128464  # 0x1F5D0
VEHICLE_TELEMETRY_OFFSET = 128468 # 0x1F5D4
VEHICLE_TELEMETRY_SIZE = 1888     # Bytes pro Fahrzeug

def open_shared_memory(name="LMU_Data", size=4*1024*1024):
    """Öffnet den LMU Shared Memory."""
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

def main():
    print("=" * 70)
    print("  LMU SHARED MEMORY – IMPACT OFFSET FINDER")
    print("=" * 70)
    print()
    print("  Starte LMU und provoziere einen Crash,")
    print("  während dieses Skript läuft!")
    print()

    handle, ptr = open_shared_memory()
    if not ptr:
        print("❌ Konnte LMU_Data nicht öffnen!")
        print("  Starte CMD als Administrator!")
        print("  Stelle sicher, dass LMU läuft!")
        input("ENTER zum Beenden...")
        return

    print("✅ LMU_Data geöffnet!")
    print()

    # Lese Anzahl Fahrzeuge aus dem Telemetry-Header
    # Annahme: Header bei 0x1F5D0 enthält NumVehicles als int32
    num_vehicles = read_i32(ptr, TELEMETRY_HEADER_OFFSET)
    print(f"  Telemetry NumVehicles (Offset 0x{TELEMETRY_HEADER_OFFSET:X}): {num_vehicles}")

    if num_vehicles <= 0 or num_vehicles > 100:
        print("  ⚠️ Ungültige Anzahl – versuche andere Header-Offsets...")
        # Probiere verschiedene Offsets
        for off in range(TELEMETRY_HEADER_OFFSET - 100, TELEMETRY_HEADER_OFFSET + 100, 4):
            v = read_i32(ptr, off)
            if 1 <= v <= 60:
                print(f"  → Möglicher NumVehicles an Offset 0x{off:X}: {v}")
    
    print()
    print("  🔍 SCANNE NACH IMPACT-DATEN...")
    print()

    # Scanne den VehicleTelemetry-Bereich nach impactMag > 0
    # Probiere verschiedene Offsets innerhalb der 1888 Bytes
    try:
        for scan_round in range(10):
            print(f"\n  --- Scan {scan_round + 1} ---")
            
            for veh in range(min(num_vehicles, 30)):
                base = VEHICLE_TELEMETRY_OFFSET + veh * VEHICLE_TELEMETRY_SIZE
                
                # Lese impactET (float) bei Offset 4, impactMag (float) bei Offset 8
                impact_et = read_f32(ptr, base + 4)
                impact_mag = read_f32(ptr, base + 8)
                
                if impact_mag > 0.01 or impact_et > 0.01:
                    print(f"  🚨 FAHRZEUG {veh+1}: impactET={impact_et:.3f}s impactMag={impact_mag:.3f}")
                    print(f"     Base Offset: 0x{base:X}")
                    
                    # Dump der ersten 100 Bytes der Telemetrie
                    print(f"     Bytes [0-100]:")
                    for i in range(0, 100, 4):
                        val_f32 = read_f32(ptr, base + i)
                        val_i32 = read_i32(ptr, base + i)
                        print(f"     0x{base+i:06X} | f32={val_f32:+.6f} | i32={val_i32}")
            
            print("  Keine Einschläge erkannt – warte 3s...")
            time.sleep(3)
            
    except KeyboardInterrupt:
        print("\n  Abbruch durch Benutzer")
    finally:
        if ptr:
            UnmapViewOfFile(ptr)
        if handle:
            CloseHandle(handle)
        print("  Aufgeräumt.")

if __name__ == "__main__":
    main()