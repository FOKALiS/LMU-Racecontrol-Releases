"""
Sucht im LMU Shared Memory (LMU_Data) nach Fahrzeugmodell-Strings.
Ziel: Finden des Offsets für mVehicleModel.

BCUK-DLL zeigt: mVehicleModel ist ein Feld in der Shared Memory Struktur.
Die Fahrzeug-Ordner in LMU heissen z.B.:
  BMW_M4_LMGT3_2023, Ferrari_499P_2023, Aston_Martin_Vantage_AMR_2023, ...
"""
import ctypes
import ctypes.wintypes
import sys

# Windows API
OpenFileMappingW = ctypes.windll.kernel32.OpenFileMappingW
MapViewOfFile = ctypes.windll.kernel32.MapViewOfFile
UnmapViewOfFile = ctypes.windll.kernel32.UnmapViewOfFile
CloseHandle = ctypes.windll.kernel32.CloseHandle
GetLastError = ctypes.windll.kernel32.GetLastError

FILE_MAP_READ = 0x0004
FILE_MAP_ALL_ACCESS = 0x000F001F
INVALID_HANDLE_VALUE = 0

# Bekannte Fahrzeugmodell-Namen (aus LMU-Installation)
VEHICLE_MODELS = [
    "911GT3R_2024", "992S_PC_2023", "ADESS_AD25_2026",
    "Alpine_A424_2024", "Aston_Martin_Valkyrie_2025", "Aston_Martin_Vantage_AMR_2023",
    "BMW_M4_LMGT3_2023", "BMW_M_Hybrid_V8_2023",
    "Cadillac_V-lmdh_2023", "Chevrolet_C8R_LM_2023", "Corvette_Z06GT3R_2023",
    "Duqueine_D09LMP3_2026", "Ferrari_296GT3_2023", "Ferrari_488GTE_LM_2023",
    "Ferrari_499P_2023", "Ford_Mustang_GT3_2024", "Genesis_GMR001_2026",
    "Ginetta_G61Evo_2025", "Isotta_Tipo6_2024",
    "Lamborghini_Huracan_GT3_2024", "Lamborghini_SC63_2024",
    "LexusRCF_GT3_2024", "Ligier_JSP325_2025",
    "McLaren_720sGChallenge_2026", "McLaren_720sGT3Evo_2023",
    "Mercedes_AMGGT3Evo_2025", "Oreca_07_ELMS_2023", "Oreca_07_LM_2023",
    "Peugeot_9x8_2023", "Peugeot_9x8_2024",
]

# Bekannte Hersteller-Kurzformen (für vehicleFilename)
MANUFACTURER_SHORTS = [
    "BMW", "Ferrari", "Porsche", "Mercedes", "Audi", "Toyota",
    "Peugeot", "Alpine", "Cadillac", "Lamborghini", "McLaren",
    "Ford", "Chevrolet", "Aston", "Lexus", "Corvette",
    "Oreca", "Ligier", "Ginetta", "Isotta", "Genesis",
    "Duqueine", "ADESS", "Dallara", "Nissan", "Honda",
]

def open_shared_memory(name="LMU_Data", size=4*1024*1024):
    wide_name = ctypes.create_unicode_buffer(name)
    # Zuerst mit ALL_ACCESS versuchen (wie Tauri-App)
    handle = OpenFileMappingW(FILE_MAP_ALL_ACCESS, False, wide_name)
    err = GetLastError()
    if not handle or handle == INVALID_HANDLE_VALUE:
        # Fallback: READ
        handle = OpenFileMappingW(FILE_MAP_READ, False, wide_name)
        err = GetLastError()
    if not handle or handle == INVALID_HANDLE_VALUE:
        return None, None, err
    desired_access = FILE_MAP_ALL_ACCESS if handle else FILE_MAP_READ
    ptr = MapViewOfFile(handle, FILE_MAP_ALL_ACCESS, 0, 0, size)
    if not ptr:
        # Fallback: READ
        ptr = MapViewOfFile(handle, FILE_MAP_READ, 0, 0, size)
    if not ptr:
        err2 = GetLastError()
        CloseHandle(handle)
        return None, None, err2
    return handle, ptr, 0

def close_shared_memory(handle, ptr):
    if ptr:
        UnmapViewOfFile(ptr)
    if handle and handle != INVALID_HANDLE_VALUE:
        CloseHandle(handle)

def read_byte(ptr, offset):
    return ctypes.c_ubyte.from_address(ptr + offset).value

def read_u32(ptr, offset):
    return ctypes.c_uint32.from_address(ptr + offset).value

def read_f32(ptr, offset):
    return ctypes.c_float.from_address(ptr + offset).value

def read_string(ptr, offset, max_len=128):
    """Liest einen null-terminierten ASCII-String ab Offset."""
    chars = []
    for i in range(max_len):
        b = read_byte(ptr, offset + i)
        if b == 0:
            break
        if 32 <= b < 127:
            chars.append(chr(b))
        else:
            break
    return ''.join(chars)

def find_strings(ptr, size, min_len=4):
    """Findet alle ASCII-Strings im Shared Memory."""
    strings = []
    current_start = None
    current_chars = []
    
    for offset in range(size):
        b = read_byte(ptr, offset)
        if 32 <= b < 127:
            if current_start is None:
                current_start = offset
            current_chars.append(chr(b))
        else:
            if current_chars and len(current_chars) >= min_len:
                s = ''.join(current_chars)
                strings.append((current_start, s))
            current_start = None
            current_chars = []
    
    if current_chars and len(current_chars) >= min_len:
        s = ''.join(current_chars)
        strings.append((current_start, s))
    
    return strings

def main():
    sys.stdout.reconfigure(encoding='utf-8', errors='replace')
    
    print("=" * 60)
    print("LMU Shared Memory - VehicleModel Scanner")
    print("=" * 60)
    
    # Shared Memory öffnen
    handle, ptr, err = open_shared_memory()
    if not ptr:
        print(f"\n❌ LMU_Data nicht gefunden (Error: {err})")
        print("   LMU läuft? Im Rennen?")
        input("ENTER zum Beenden...")
        return
    
    print(f"\n✅ LMU_Data Shared Memory geöffnet!")
    
    # 1. ALLE Strings finden
    print(f"\n--- Suche nach Strings (min 4 Zeichen) ---")
    strings = find_strings(ptr, 4 * 1024 * 1024, min_len=4)
    print(f"   {len(strings)} Strings gefunden")
    
    # 2. Nach Fahrzeugmodell-Namen suchen
    print(f"\n--- Suche nach Fahrzeugmodell-Namen ---")
    found_models = []
    for offset, s in strings:
        for model in VEHICLE_MODELS:
            if model in s:
                found_models.append((offset, s))
                break
    
    if found_models:
        print(f"   {len(found_models)} Fahrzeugmodelle gefunden:")
        for offset, s in sorted(found_models):
            print(f"   Offset 0x{offset:06X}: '{s}'")
    else:
        print("   Keine Fahrzeugmodell-Namen gefunden")
    
    # 3. Nach Hersteller-Kurzformen suchen (aus vehicleFilename)
    print(f"\n--- Suche nach Hersteller-Kurzformen ---")
    found_manufacturers = []
    for offset, s in strings:
        for mfr in MANUFACTURER_SHORTS:
            if s == mfr or s.startswith(mfr + "_") or s.endswith("_" + mfr):
                found_manufacturers.append((offset, s))
                break
    
    if found_manufacturers:
        print(f"   {len(found_manufacturers)} Hersteller-Einträge gefunden:")
        for offset, s in sorted(found_manufacturers):
            print(f"   Offset 0x{offset:06X}: '{s}'")
    else:
        print("   Keine Hersteller-Kurzformen gefunden")
    
    # 4. Scoring-Struktur analysieren (Offset 1632)
    print(f"\n--- Scoring-Struktur (Offset 1632) ---")
    scoring_offset = 1632
    num_vehicles = read_u32(ptr, scoring_offset + 24)
    print(f"   Anzahl Fahrzeuge (Offset 1632+24): {num_vehicles}")
    
    # 5. Vehicle Telemetry Bereich nach Strings durchsuchen
    print(f"\n--- Vehicle Telemetry Bereich (Offset 128468) ---")
    vt_offset = 128468
    vt_size = 1888
    max_vehicles = min(num_vehicles, 64) if num_vehicles > 0 else 64
    
    for i in range(max_vehicles):
        base = vt_offset + i * vt_size
        slot_id = read_u32(ptr, base)
        
        # Strings im Telemetry-Block suchen
        vt_strings = []
        for off in range(0, vt_size, 4):
            s = read_string(ptr, base + off, max_len=64)
            if len(s) >= 4:
                vt_strings.append((off, s))
        
        if vt_strings:
            print(f"\n   Fahrzeug {i} (slotID={slot_id}):")
            for off, s in vt_strings[:10]:  # Max 10 Strings pro Fahrzeug
                print(f"     +0x{off:04X}: '{s}'")
            if len(vt_strings) > 10:
                print(f"     ... und {len(vt_strings)-10} weitere")
    
    # 6. Gesamten Speicher nach "VehicleModel" durchsuchen
    print(f"\n--- Suche nach 'VehicleModel' im gesamten Speicher ---")
    for offset in range(0, 4 * 1024 * 1024 - 12, 4):
        s = read_string(ptr, offset, max_len=64)
        if "VehicleModel" in s or "vehicleModel" in s:
            print(f"   Offset 0x{offset:06X}: '{s}'")
    
    close_shared_memory(handle, ptr)
    print(f"\n✅ Fertig!")
    input("ENTER zum Beenden...")

if __name__ == "__main__":
    main()