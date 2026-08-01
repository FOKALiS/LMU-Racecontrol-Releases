"""
LMU Shared Memory Reader
Liest LMU_Data Shared Memory mit den bekannten Offsets aus der SM Bridge.
Starte: python scripts/read_lmu_sm.py
"""
import ctypes
import ctypes.wintypes
import struct
import sys
import time

# Windows API
OpenFileMappingW = ctypes.windll.kernel32.OpenFileMappingW
MapViewOfFile = ctypes.windll.kernel32.MapViewOfFile
UnmapViewOfFile = ctypes.windll.kernel32.UnmapViewOfFile
CloseHandle = ctypes.windll.kernel32.CloseHandle
GetLastError = ctypes.windll.kernel32.GetLastError

FILE_MAP_READ = 0x0004
FILE_MAP_WRITE = 0x0002
FILE_MAP_ALL_ACCESS = 0x000F001F

# Bekannte Offsets aus der SM Bridge
OFFSETS = {
    "Scoring": 1632,
    "VehScoring": 2192,
    "Telemetry": 128464,
    "VehTelem": 128468,
}

STRUCT_SIZES = {
    "ScoringInfo": 548,
    "VehicleScoring": 584,
    "VehicleTelemetry": 1888,
}

def open_shared_memory(name="LMU_Data", size=4*1024*1024):
    """Öffnet den LMU Shared Memory."""
    wide_name = ctypes.create_unicode_buffer(name)
    
    # Nur READ verwenden (ALL_ACCESS gibt Error 5)
    # Versuche zuerst ALL_ACCESS
    handle = OpenFileMappingW(FILE_MAP_ALL_ACCESS, False, wide_name)
    if not handle:
        # Fallback: READ
        handle = OpenFileMappingW(FILE_MAP_READ, False, wide_name)
        err = GetLastError()
        if not handle:
            return None, None, err
    
    ptr = MapViewOfFile(handle, FILE_MAP_READ, 0, 0, size)
    if not ptr:
        err2 = GetLastError()
        CloseHandle(handle)
        return None, None, err2
    
    return handle, ptr, 0

def read_u32(ptr, offset):
    return ctypes.c_uint32.from_address(ptr + offset).value

def read_f32(ptr, offset):
    return ctypes.c_float.from_address(ptr + offset).value

def read_i32(ptr, offset):
    return ctypes.c_int32.from_address(ptr + offset).value

def read_bytes(ptr, offset, size):
    return bytes(ctypes.c_ubyte.from_address(ptr + offset + i).value for i in range(size))

def read_string(ptr, offset, max_len=64):
    """Liest einen null-terminierten UTF-16 String."""
    chars = []
    for i in range(0, max_len, 2):
        c = ctypes.c_uint16.from_address(ptr + offset + i).value
        if c == 0:
            break
        chars.append(chr(c))
    return ''.join(chars)

def dump_scoring_info(ptr, offset=1632):
    """Dump der ScoringInfo-Struktur (548 Bytes)."""
    print(f"\n{'='*60}")
    print(f"  SCORING INFO (Offset 0x{offset:04X}, {STRUCT_SIZES['ScoringInfo']} Bytes)")
    print(f"{'='*60}")
    
    # Bekannte Felder aus rFactor2 Shared Memory
    fields = [
        (0, "mVersion", "u32"),
        (4, "mSession", "u32"),
        (8, "mCurrentET", "f32"),
        (12, "mEndET", "f32"),
        (16, "mMaxLaps", "i32"),
        (20, "mLapDist", "f32"),
        (24, "mNumVehicles", "i32"),
    ]
    
    for field_offset, name, typ in fields:
        abs_offset = offset + field_offset
        if typ == "u32":
            val = read_u32(ptr, abs_offset)
        elif typ == "f32":
            val = read_f32(ptr, abs_offset)
        elif typ == "i32":
            val = read_i32(ptr, abs_offset)
        print(f"  {name:20s} = {val}")
    
    # Session-Name (String bei Offset 32)
    session_name = read_string(ptr, offset + 32, 64)
    print(f"  {'mSessionName':20s} = '{session_name}'")

def dump_vehicle_scoring(ptr, base_offset=2192, vehicle_index=0):
    """Dump eines VehicleScoring-Eintrags (584 Bytes)."""
    offset = base_offset + vehicle_index * STRUCT_SIZES["VehicleScoring"]
    
    print(f"\n{'='*60}")
    print(f"  VEHICLE SCORING [{vehicle_index}] (Offset 0x{offset:04X})")
    print(f"{'='*60}")
    
    # Bekannte Felder
    fields = [
        (0, "mDriverName", "string"),
        (64, "mTeamName", "string"),
        (128, "mCarClass", "string"),
        (192, "mCarNumber", "string"),
        (256, "mTotalLaps", "i32"),
        (260, "mSector", "i32"),
        (264, "mFinishStatus", "i32"),
        (268, "mLapDist", "f32"),
        (272, "mPathLateral", "f32"),
        (276, "mTrackEdge", "f32"),
        (280, "mBestSector1", "f32"),
        (284, "mBestSector2", "f32"),
        (288, "mBestLapTime", "f32"),
        (292, "mLastLapTime", "f32"),
        (296, "mCurrentTime", "f32"),
        (300, "mSplitTime", "f32"),
        (304, "mEventTime", "f32"),
        (308, "mPenaltyTime", "f32"),
        (312, "mSpeed", "f32"),
        (316, "mBestSpeed", "f32"),
        (320, "mNumPitstops", "i32"),
        (324, "mNumPenalties", "i32"),
        (328, "mInPit", "i32"),
        (332, "mPlace", "i32"),
        (336, "mNumCuts", "i32"),
        (340, "mTimeBehindNext", "f32"),
        (344, "mTimeBehindLeader", "f32"),
        (348, "mLapsBehindLeader", "i32"),
        (352, "mTimeBehindPrev", "f32"),
        (356, "mLapsBehindPrev", "i32"),
        (360, "mStartPos", "i32"),
        (364, "mTrackSurface", "i32"),
    ]
    
    for field_offset, name, typ in fields:
        abs_offset = offset + field_offset
        if typ == "string":
            val = read_string(ptr, abs_offset, 64)
        elif typ == "u32":
            val = read_u32(ptr, abs_offset)
        elif typ == "f32":
            val = read_f32(ptr, abs_offset)
        elif typ == "i32":
            val = read_i32(ptr, abs_offset)
        print(f"  {name:20s} = {val}")

def main():
    sys.stdout.reconfigure(encoding='utf-8', errors='replace')
    
    print("=" * 60)
    print("  LMU SHARED MEMORY READER")
    print("  (Offsets aus SM Bridge Logs)")
    print("=" * 60)
    
    handle, ptr, err = open_shared_memory()
    if not ptr:
        print(f"\n❌ Konnte LMU_Data nicht öffnen (Error: {err})")
        print("  Starte CMD als Administrator und versuche erneut!")
        print("  Stelle sicher, dass LMU läuft und du in einem Rennen bist!")
        input("\nENTER zum Beenden...")
        return
    
    print(f"\n✅ LMU_Data Shared Memory geöffnet!")
    print(f"   Scoring Offset:     0x{OFFSETS['Scoring']:04X}")
    print(f"   VehScoring Offset:  0x{OFFSETS['VehScoring']:04X}")
    print(f"   Telemetry Offset:   0x{OFFSETS['Telemetry']:04X}")
    print(f"   VehTelem Offset:    0x{OFFSETS['VehTelem']:04X}")
    
    # Scoring Info auslesen
    dump_scoring_info(ptr, OFFSETS["Scoring"])
    
    # Anzahl Fahrzeuge
    num_vehicles = read_i32(ptr, OFFSETS["Scoring"] + 24)
    print(f"\n  Anzahl Fahrzeuge: {num_vehicles}")
    
    # Erstes Fahrzeug auslesen
    if num_vehicles > 0:
        dump_vehicle_scoring(ptr, OFFSETS["VehScoring"], 0)
    
    # Live-Updates
    print(f"\n{'='*60}")
    print("  LIVE-UPDATES (alle 2 Sekunden, STRG+C zum Beenden)")
    print(f"{'='*60}")
    
    try:
        while True:
            # Session-Zeit
            current_et = read_f32(ptr, OFFSETS["Scoring"] + 8)
            num_veh = read_i32(ptr, OFFSETS["Scoring"] + 24)
            
            print(f"\n  ET={current_et:.1f}s | Fahrzeuge={num_veh}")
            
            # Alle Fahrzeuge kurz anzeigen
            for i in range(min(num_veh, 5)):
                voffset = OFFSETS["VehScoring"] + i * STRUCT_SIZES["VehicleScoring"]
                name = read_string(ptr, voffset, 32)
                car_num = read_string(ptr, voffset + 192, 8)
                speed = read_f32(ptr, voffset + 312)
                place = read_i32(ptr, voffset + 332)
                in_pit = read_i32(ptr, voffset + 328)
                pit_str = " (PIT)" if in_pit else ""
                print(f"    P{place:2d} #{car_num:4s} {name:20s} {speed:5.1f} km/h{pit_str}")
            
            sys.stdout.flush()
            time.sleep(2)
            
    except KeyboardInterrupt:
        print("\n  Beende...")
    finally:
        if ptr:
            UnmapViewOfFile(ptr)
        if handle:
            CloseHandle(handle)
        print("  Aufgeräumt. Tschüss!")

if __name__ == "__main__":
    main()