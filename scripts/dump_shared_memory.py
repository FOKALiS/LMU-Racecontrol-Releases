"""
LMU Shared Memory Dumper
Liest LMU_Data Shared Memory aus und zeigt Aenderungen nach einem Tastendruck.
"""
import ctypes
import ctypes.wintypes
import sys
import os

# Windows API
OpenFileMappingW = ctypes.windll.kernel32.OpenFileMappingW
MapViewOfFile = ctypes.windll.kernel32.MapViewOfFile
UnmapViewOfFile = ctypes.windll.kernel32.UnmapViewOfFile
CloseHandle = ctypes.windll.kernel32.CloseHandle

FILE_MAP_ALL_ACCESS = 0x000F001F
INVALID_HANDLE_VALUE = -1

def open_shared_memory(name="LMU_Data", size=4*1024*1024):
    wide_name = ctypes.create_unicode_buffer(name)
    handle = OpenFileMappingW(FILE_MAP_ALL_ACCESS, False, wide_name)
    if not handle or handle == INVALID_HANDLE_VALUE:
        return None, None
    ptr = MapViewOfFile(handle, FILE_MAP_ALL_ACCESS, 0, 0, size)
    if not ptr:
        CloseHandle(handle)
        return None, None
    return handle, ptr

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

def take_snapshot(ptr, size=4096):
    return bytes(ctypes.c_ubyte.from_address(ptr + i).value for i in range(size))

def main():
    sys.stdout.reconfigure(encoding='utf-8', errors='replace')
    
    print("=" * 60)
    print("LMU Shared Memory Dumper")
    print("=" * 60)
    
    handle, ptr = open_shared_memory()
    if not ptr:
        print("FEHLER: LMU_Data Shared Memory NICHT gefunden!")
        print("Stelle sicher, dass LMU laeuft und du in einem Rennen bist!")
        return
    
    print("OK - LMU_Data Shared Memory geoeffnet!")
    print()
    
    # Ersten Dump
    print("--- HEX-DUMP (erste 256 Bytes) ---")
    for i in range(0, 256, 16):
        hex_bytes = [f"{read_byte(ptr, i+j):02X}" for j in range(16)]
        print(f"  {i:04X}: {' '.join(hex_bytes)}")
    print()
    
    print("--- WERTE (erste 256 Bytes, nur nicht-Null) ---")
    for offset in range(0, 256, 4):
        u32_val = read_u32(ptr, offset)
        f32_val = read_f32(ptr, offset)
        if u32_val != 0 or abs(f32_val) > 0.001:
            print(f"  Offset 0x{offset:04X}: u32={u32_val:10d}  f32={f32_val:10.4f}")
    print()
    
    # Snapshot 1
    snap1 = take_snapshot(ptr, 4096)
    
    print("=" * 60)
    print("JETZT in Broadcast Control auf 'Nose' klicken!")
    print("Dann ENTER druecken fuer zweiten Snapshot...")
    print("=" * 60)
    input()
    
    # Snapshot 2
    snap2 = take_snapshot(ptr, 4096)
    
    # Unterschiede
    changes = []
    for i in range(min(len(snap1), len(snap2))):
        if snap1[i] != snap2[i]:
            changes.append((i, snap1[i], snap2[i]))
            if len(changes) >= 50:
                break
    
    print()
    print("=" * 60)
    print(f"AENDERUNGEN: {len(changes)} Bytes geaendert")
    print("=" * 60)
    
    if changes:
        print()
        print("Geaenderte Bytes (Offset | Alt -> Neu | u32/f32):")
        for offset, old_val, new_val in changes:
            u32_old = read_u32(ptr, offset & ~0x03)
            u32_new = read_u32(ptr, offset & ~0x03)
            f32_old = read_f32(ptr, offset & ~0x03)
            f32_new = read_f32(ptr, offset & ~0x03)
            
            line = f"  0x{offset:04X}: {old_val:3d} -> {new_val:3d}"
            if offset % 4 == 0:
                line += f"  (u32: {u32_old} -> {u32_new}, f32: {f32_old:.4f} -> {f32_new:.4f})"
            print(line)
        
        if len(changes) >= 50:
            print(f"  ... und mehr Aenderungen")
    else:
        print("Keine Aenderungen gefunden!")
        print("Wurde wirklich auf 'Nose' geklickt?")
    
    close_shared_memory(handle, ptr)
    print()
    print("Fertig!")

if __name__ == "__main__":
    main()