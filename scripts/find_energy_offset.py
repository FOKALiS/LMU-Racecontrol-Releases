"""
Findet den Offset für mVirtualEnergy in der LmuVehicleTelemetry-Struktur.
LMU muss laufen und im Rennen sein! (CMD als Administrator)

Die Struktur ist 1888 Bytes groß, Offsets bekannt:
  0x000: slotID (i32)
  0x004: impactET (f32)
  0x008: impactMag (f32)

Wir suchen ab Offset 0x00C (12) nach Float-Werten zwischen 0.0 und 1.0,
die sich wie "Virtuelle Energie" verhalten (pro Fahrzeug unterschiedlich).
"""
import ctypes
import ctypes.wintypes
import time
import sys
import struct

sys.stdout.reconfigure(encoding="utf-8", errors="replace")

OpenFileMappingW = ctypes.windll.kernel32.OpenFileMappingW
MapViewOfFile = ctypes.windll.kernel32.MapViewOfFile
UnmapViewOfFile = ctypes.windll.kernel32.UnmapViewOfFile
CloseHandle = ctypes.windll.kernel32.CloseHandle

FILE_MAP_READ = 0x0004
FILE_MAP_ALL_ACCESS = 0x000F001F

SCORING_OFFSET = 1632
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

def main():
    print("=" * 70)
    print("  LMU SHARED MEMORY – VIRTUAL ENERGY OFFSET SCANNER")
    print("=" * 70)
    print("  LMU muss laufen und im Rennen sein!")
    print("  CMD als Administrator starten!")
    print()

    handle, ptr = open_shared_memory()
    if not ptr:
        print("❌ LMU_Data nicht gefunden. LMU läuft nicht?")
        input("ENTER zum Beenden...")
        return

    print("✅ LMU_Data geöffnet!")
    print()

    # Anzahl Fahrzeuge
    num_vehicles = read_i32(ptr, SCORING_OFFSET + 24)
    print(f"  Fahrzeuge: {num_vehicles}")
    print()

    # Für jedes Fahrzeug: die ersten 400 Bytes der Telemetrie dumpen
    # und nach Float-Werten zwischen 0.0 und 1.0 suchen
    print(f"{'='*70}")
    print("  SCAN: Suche nach Float-Werten (0.0–1.0) in VehicleTelemetry")
    print(f"{'='*70}")

    for veh in range(min(num_vehicles, 10)):
        base = VEHICLE_TELEMETRY_OFFSET + veh * VEHICLE_TELEMETRY_SIZE
        slot_id = read_i32(ptr, base)
        print(f"\n  Fahrzeug {veh} (slotID={slot_id}):")

        # Scanne ab Offset 12 (nach impactET/Mag) bis 400
        candidates = []
        for off in range(12, 400, 4):
            val = read_f32(ptr, base + off)
            if 0.0 <= val <= 1.0:
                candidates.append((off, val))

        if candidates:
            for off, val in candidates:
                # Kontext: zeige 4 Werte davor + danach
                ctx = []
                for j in range(max(12, off - 16), min(off + 20, 400), 4):
                    v = read_f32(ptr, base + j)
                    marker = " <--" if j == off else ""
                    ctx.append(f"+0x{j:03X}={v:.4f}{marker}")
                print(f"    {' | '.join(ctx)}")
        else:
            print("    Keine Float-Werte 0.0-1.0 gefunden (Slot leer?)")

    # Live-Überwachung: zeige Energie-Werte über 5 Sekunden
    print(f"\n{'='*70}")
    print("  LIVE-ÜBERWACHUNG (5 Sekunden)")
    print(f"{'='*70}")
    print("  Zeige alle Float-Werte 0.0-1.0 pro Fahrzeug")
    print()

    try:
        for t in range(5):
            print(f"\n  --- Tick {t+1}/5 ---")
            for veh in range(min(num_vehicles, 10)):
                base = VEHICLE_TELEMETRY_OFFSET + veh * VEHICLE_TELEMETRY_SIZE
                slot_id = read_i32(ptr, base)
                # Nur Offsets mit Werten 0.0-1.0 anzeigen
                vals = []
                for off in range(12, 400, 4):
                    val = read_f32(ptr, base + off)
                    if 0.0 <= val <= 1.0:
                        vals.append(f"+0x{off:03X}={val:.3f}")
                if vals:
                    print(f"  Slot {slot_id}: {' | '.join(vals)}")
            time.sleep(1)
    except KeyboardInterrupt:
        pass

    UnmapViewOfFile(ptr)
    CloseHandle(handle)
    print("\n✅ Fertig.")
    input("ENTER zum Beenden...")

if __name__ == "__main__":
    main()