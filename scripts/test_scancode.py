"""Testet Scancode-Simulation an LMU via Camera Helper"""
import sys
import json
import ctypes
from ctypes import wintypes

PIPE = r"\\.\pipe\LMU_CameraHelper"
kernel32 = ctypes.windll.kernel32

def send_command(cmd_dict):
    cmd_bytes = json.dumps(cmd_dict).encode('utf-8')
    handle = kernel32.CreateFileW(PIPE, 0xC0000000, 0, None, 3, 0, None)
    if handle == -1 or handle is None:
        print(f"❌ Pipe nicht verfügbar")
        return
    written = wintypes.DWORD(0)
    success = kernel32.WriteFile(handle, cmd_bytes, len(cmd_bytes), ctypes.byref(written), None)
    if success:
        print(f"✅ Gesendet: {json.dumps(cmd_dict)}")
    buf = ctypes.create_string_buffer(4096)
    read = wintypes.DWORD(0)
    kernel32.ReadFile(handle, buf, 4096, ctypes.byref(read), None)
    response = buf.raw[:read.value].decode('utf-8', errors='ignore').strip('\x00').strip()
    print(f"✅ Antwort: {response}")
    kernel32.CloseHandle(handle)

if __name__ == '__main__':
    # Scancodes: PageDown=0xE051 (Extended), aber einfacher: 0x51
    # Insert=0xE052 (Extended), Home=0xE047, End=0xE04F, PageUp=0xE049
    # Ohne Extended: PageDown=0x51, Insert=0x52, Home=0x47, End=0x4F
    
    print("Teste Scancode-Modus:")
    print("1. PageDown (Scancode 0x51)")
    send_command({"cmd": "scancode", "scan": 0x51, "extended": False})
    print("2. Insert (Scancode 0x52)")
    send_command({"cmd": "scancode", "scan": 0x52, "extended": False})
    print("3. Exit")
    send_command({"cmd": "exit"})