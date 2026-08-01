"""Testet die Named Pipe zum Camera Helper"""
import sys
import json
import time

PIPE = r"\\.\pipe\LMU_CameraHelper"

def send_command(cmd_dict):
    """Sendet JSON-Kommando an die Pipe und wartet auf Antwort."""
    try:
        import win32pipe
        import win32file
        import pywintypes
    except ImportError:
        # Fallback: ctypes
        import ctypes
        from ctypes import wintypes
        
        kernel32 = ctypes.windll.kernel32
        
        PIPE_ACCESS_DUPLEX = 0x3
        PIPE_READMODE_MESSAGE = 0x2
        PIPE_WAIT = 0x0
        PIPE_UNLIMITED_INSTANCES = 255
        BUFFER_SIZE = 4096
        
        # Pipe öffnen
        handle = kernel32.CreateFileW(
            PIPE,
            0xC0000000,  # GENERIC_READ | GENERIC_WRITE
            0,
            None,
            3,  # OPEN_EXISTING
            0,
            None
        )
        
        if handle == -1 or handle is None:
            print(f"❌ Pipe nicht verfügbar (Error: {ctypes.GetLastError()})")
            return None
        
        # Kommando senden
        cmd_bytes = json.dumps(cmd_dict).encode('utf-8')
        written = wintypes.DWORD(0)
        success = kernel32.WriteFile(
            handle,
            cmd_bytes,
            len(cmd_bytes),
            ctypes.byref(written),
            None
        )
        
        if not success:
            print(f"❌ WriteFile fehlgeschlagen (Error: {ctypes.GetLastError()})")
            kernel32.CloseHandle(handle)
            return None
        
        print(f"✅ Gesendet: {json.dumps(cmd_dict)}")
        
        # Antwort lesen
        buf = ctypes.create_string_buffer(BUFFER_SIZE)
        read = wintypes.DWORD(0)
        success = kernel32.ReadFile(
            handle,
            buf,
            BUFFER_SIZE,
            ctypes.byref(read),
            None
        )
        
        response = buf.raw[:read.value].decode('utf-8', errors='ignore').strip('\x00').strip()
        print(f"✅ Antwort: {response}")
        
        kernel32.CloseHandle(handle)
        return response

if __name__ == '__main__':
    print("=" * 50)
    print("Camera Helper Pipe Test")
    print("=" * 50)
    
    if len(sys.argv) > 1:
        cmd = sys.argv[1]
        vk = int(sys.argv[2]) if len(sys.argv) > 2 else 45
        send_command({"cmd": cmd, "vk": vk})
    else:
        print("\nTeste: TV-Kamera (Insert = 0x2D = 45)")
        send_command({"cmd": "key", "vk": 45})
        time.sleep(0.5)
        
        print("\nTeste: Exit")
        send_command({"cmd": "exit"})