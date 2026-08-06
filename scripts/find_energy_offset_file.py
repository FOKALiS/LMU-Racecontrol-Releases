"""
Findet den Offset für mVirtualEnergy in der LmuVehicleTelemetry-Struktur.
LMU muss laufen und im Rennen sein! (CMD als Administrator)

Schreibt Ergebnisse in energy_offsets.json.
"""
import ctypes
import ctypes.wintypes
import time
import sys
import json
import os

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

OUTPUT_FILE = os.path.join(os.path.dirname(__file__), "..", "energy_offsets.json")

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
    print()

    handle, ptr = open_shared_memory()
    if not ptr:
        print("❌ LMU_Data nicht gefunden. LMU läuft nicht?")
        # Ergebnis-Datei mit Fehler schreiben
        with open(OUTPUT_FILE, "w") as f:
            json.dump({"error": "LMU_Data nicht gefunden"}, f)
        print(f"  Ergebnis in {OUTPUT_FILE}")
        return

    print("✅ LMU_Data geöffnet!")

    num_vehicles = read_i32(ptr, SCORING_OFFSET + 24)
    print(f"  Fahrzeuge: {num_vehicles}")
    print()

    results = {}
    results["num_vehicles"] = num_vehicles
    results["vehicle_telemetry_size"] = VEHICLE_TELEMETRY_SIZE
    results["offsets"] = {}

    # Für jedes Fahrzeug: Werte von Offset 12 bis 400 scannen
    print(f"{'='*70}")
    print("  SCAN: Float-Werte (0.0-1.0) pro Fahrzeug")
    print(f"{'='*70}")

    for veh in range(min(num_vehicles, 32)):
        base = VEHICLE_TELEMETRY_OFFSET + veh * VEHICLE_TELEMETRY_SIZE
        slot_id = read_i32(ptr, base)
        print(f"\n  Slot {slot_id}:")

        veh_data = {}
        for off in range(12, 400, 4):
            val = read_f32(ptr, base + off)
            veh_data[hex(off)] = round(val, 6)

        # Nur Werte zwischen 0.0 und 1.0 anzeigen
        interesting = {k: v for k, v in veh_data.items() if 0.0 <= v <= 1.0}
        if interesting:
            for k, v in interesting.items():
                print(f"    Offset {k} = {v:.4f}")
        else:
            print("    Keine Werte 0.0-1.0 (Slot leer?)")

        results["offsets"][str(slot_id)] = veh_data

    # Live-Überwachung: 3 Ticks, um stabile vs. sich ändernde Werte zu finden
    print(f"\n{'='*70}")
    print("  LIVE-ÜBERWACHUNG (3 Sekunden, 1s Intervall)")
    print(f"{'='*70}")

    live_data = []
    for tick in range(3):
        tick_data = {}
        for veh in range(min(num_vehicles, 32)):
            base = VEHICLE_TELEMETRY_OFFSET + veh * VEHICLE_TELEMETRY_SIZE
            slot_id = read_i32(ptr, base)
            vals = {}
            for off in range(12, 400, 4):
                val = read_f32(ptr, base + off)
                if 0.0 <= val <= 1.0:
                    vals[hex(off)] = round(val, 4)
            if vals:
                tick_data[str(slot_id)] = vals
        live_data.append(tick_data)
        print(f"  Tick {tick+1}: {len(tick_data)} Fahrzeuge mit Float-Werten")
        time.sleep(1)

    results["live_ticks"] = live_data

    # Analyse: Welche Offsets sind KONSTANT (fuel_fraction?) und welche VARIABEL (virtual_energy?)
    print(f"\n{'='*70}")
    print("  ANALYSE: Konstante vs. variable Werte")
    print(f"{'='*70}")

    if len(live_data) >= 2:
        for slot_id_str in live_data[0]:
            print(f"\n  Slot {slot_id_str}:")
            for off_hex in live_data[0][slot_id_str]:
                v0 = live_data[0][slot_id_str].get(off_hex)
                v1 = live_data[1][slot_id_str].get(off_hex)
                v2 = live_data[2][slot_id_str].get(off_hex) if len(live_data) > 2 else v1
                if v0 is not None and v1 is not None:
                    change = max(abs(v1 - v0), abs(v2 - v1)) if v2 else abs(v1 - v0)
                    label = "VARIABEL" if change > 0.001 else "konstant"
                    print(f"    Offset {off_hex}: {v0:.4f} → {v1:.4f} → {v2:.4f}  [{label}]")

    # Ergebnisse speichern
    results["analysis"] = "search results"
    with open(OUTPUT_FILE, "w") as f:
        json.dump(results, f, indent=2)
    print(f"\n✅ Ergebnisse gespeichert in {OUTPUT_FILE}")

    UnmapViewOfFile(ptr)
    CloseHandle(handle)
    print("✅ Fertig!")

if __name__ == "__main__":
    main()