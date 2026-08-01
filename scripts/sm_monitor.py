"""
SM Monitor – überwacht Shared Memory auf Änderungen.
Starte: python scripts/sm_monitor.py
Dann starte BCUK und klicke auf Kamera-Buttons.
"""
import ctypes, ctypes.wintypes, sys, time, os, struct

OpenFileMappingW = ctypes.windll.kernel32.OpenFileMappingW
MapViewOfFile = ctypes.windll.kernel32.MapViewOfFile
UnmapViewOfFile = ctypes.windll.kernel32.UnmapViewOfFile
CloseHandle = ctypes.windll.kernel32.CloseHandle
GetLastError = ctypes.windll.kernel32.GetLastError

FILE_MAP_READ = 0x0004
FILE_MAP_ALL_ACCESS = 0x000F001F

def open_sm(name="LMU_Data", size=4*1024*1024):
    wide = ctypes.create_unicode_buffer(name)
    h = OpenFileMappingW(FILE_MAP_READ, False, wide)
    err = GetLastError()
    if not h:
        h = OpenFileMappingW(FILE_MAP_ALL_ACCESS, False, wide)
        err = GetLastError()
        if not h:
            return None, None, err
    p = MapViewOfFile(h, FILE_MAP_READ, 0, 0, size)
    if not p:
        CloseHandle(h)
        return None, None, GetLastError()
    return h, p, 0

def read_byte(p, off):
    return ctypes.c_ubyte.from_address(p + off).value

def read_u32(p, off):
    return ctypes.c_uint32.from_address(p + off).value

def read_f32(p, off):
    return ctypes.c_float.from_address(p + off).value

def snapshot(p, size=4096):
    return bytes(read_byte(p, i) for i in range(size))

def main():
    sys.stdout.reconfigure(encoding='utf-8', errors='replace')
    
    print("=" * 60)
    print("  LMU SHARED MEMORY MONITOR")
    print("  Überwacht auf Änderungen durch BCUK")
    print("=" * 60)
    
    h, p, err = open_sm()
    if not p:
        print(f"\n❌ LMU_Data nicht gefunden (Error: {err})")
        print("  Starte CMD als Administrator!")
        input("ENTER zum Beenden...")
        return
    
    print("\n✅ LMU_Data geöffnet!")
    print("  Starte jetzt BCUK und klicke auf Kamera-Buttons")
    print("  Drücke STRG+C zum Beenden")
    print()
    
    # Erstes Snapshot
    prev = snapshot(p, 4096)
    print("  Überwache... (warte auf Änderungen)")
    
    try:
        while True:
            time.sleep(0.5)
            curr = snapshot(p, 4096)
            
            # Vergleiche
            changes = []
            for i in range(len(prev)):
                if prev[i] != curr[i]:
                    changes.append((i, prev[i], curr[i]))
            
            if changes:
                print(f"\n  🔴 {len(changes)} Bytes geändert!")
                for off, old, new in changes[:20]:
                    u32_old = read_u32(p, off & ~0x03)
                    u32_new = read_u32(p, off & ~0x03)
                    f32_old = read_f32(p, off & ~0x03)
                    f32_new = read_f32(p, off & ~0x03)
                    
                    line = f"    0x{off:04X}: {old:3d} -> {new:3d}"
                    if off % 4 == 0:
                        line += f"  (u32: {u32_old} -> {u32_new}, f32: {f32_old:.4f} -> {f32_new:.4f})"
                    print(line)
                
                if len(changes) > 20:
                    print(f"    ... und {len(changes)-20} weitere")
                
                prev = curr
                print()
    
    except KeyboardInterrupt:
        print("\n  Beende...")
    finally:
        if p: UnmapViewOfFile(p)
        if h: CloseHandle(h)

if __name__ == "__main__":
    main()