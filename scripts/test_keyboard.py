t zu """
LMU Kamera-Tastatur-Test
Sendet Tastendrücke via SendInput mit KEYEVENTF_SCANCODE an LMU.

Verwendung:
1. LMU starten (Rennen, Watch-Modus)
2. LMU-Fenster in den Vordergrund bringen (ANKLICKEN!)
3. python scripts/test_keyboard.py
4. Drücke 1, 2, 3, 4, 5 für verschiedene Kameras
"""

import ctypes
import ctypes.wintypes
import time
import sys

# Win32 API
user32 = ctypes.windll.user32
kernel32 = ctypes.windll.kernel32

# Konstanten
INPUT_KEYBOARD = 1
KEYEVENTF_SCANCODE = 0x0008
KEYEVENTF_KEYUP = 0x0002
KEYEVENTF_EXTENDEDKEY = 0x0001

class KEYBDINPUT(ctypes.Structure):
    _fields_ = [
        ("wVk", ctypes.wintypes.WORD),
        ("wScan", ctypes.wintypes.WORD),
        ("dwFlags", ctypes.wintypes.DWORD),
        ("time", ctypes.wintypes.DWORD),
        ("dwExtraInfo", ctypes.c_size_t),
    ]

class INPUT(ctypes.Structure):
    _fields_ = [
        ("type", ctypes.wintypes.DWORD),
        ("padding", ctypes.c_byte * 4),
        ("ki", KEYBDINPUT),
        ("padding2", ctypes.c_byte * 16),  # 40 bytes total
    ]

def send_key(scan, extended=False):
    """Sendet einen Tastendruck (down + up) über SendInput"""
    flags_down = KEYEVENTF_SCANCODE
    flags_up = KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP
    if extended:
        flags_down |= KEYEVENTF_EXTENDEDKEY
        flags_up |= KEYEVENTF_EXTENDEDKEY
    
    # KeyDown
    input_down = INPUT()
    input_down.type = INPUT_KEYBOARD
    input_down.ki.wScan = scan
    input_down.ki.dwFlags = flags_down
    input_down.ki.time = 0
    input_down.ki.dwExtraInfo = 0
    
    # KeyUp
    input_up = INPUT()
    input_up.type = INPUT_KEYBOARD
    input_up.ki.wScan = scan
    input_up.ki.dwFlags = flags_up
    input_up.ki.time = 0
    input_up.ki.dwExtraInfo = 0
    
    inputs = (INPUT * 2)(input_down, input_up)
    result = user32.SendInput(2, inputs, ctypes.sizeof(INPUT))
    time.sleep(0.1)  # Kurze Pause zwischen Tasten
    return result

# ─── Test-Modi ──────────────────────────────────────────────────────────

def test_mode_1():
    """Test 1: Windows Virtual Key Codes (VK_*)"""
    print("\n📋 Test 1: Windows VK-Codes")
    print("   Sendet VK_INSERT=0x2D, VK_HOME=0x24, etc.")
    print("   (funktioniert bei Spielen die Windows Messages nutzen)")
    
    user32.keybd_event(0x2D, 0, 0, 0)  # VK_INSERT down
    time.sleep(0.05)
    user32.keybd_event(0x2D, 0, 2, 0)  # VK_INSERT up
    print("   ✅ VK_INSERT gesendet")
    time.sleep(1)

def test_mode_2():
    """Test 2: SendInput mit Scancodes (DirectInput-kompatibel)"""
    print("\n📋 Test 2: SendInput + Scancodes")
    print("   Sendet PS/2-kompatible Scancodes via KEYEVENTF_SCANCODE")
    
    # Kamerazuordnung
    tests = [
        ("Driving Cameras (Insert)", 0x52, True),    # Insert = E0 52
        ("Onboard Cameras (Home)", 0x47, True),        # Home = E0 47
        ("Swingman Camera (PageUp)", 0x49, True),      # PageUp = E0 49
        ("Tracking Cameras (PageDown)", 0x51, True),   # PageDown = E0 51
        ("Spectator Cameras (End)", 0x4F, True),       # End = E0 4F
    ]
    
    for name, scan, extended in tests:
        print(f"   🔄 {name} (Scan=0x{scan:02X}, extended={extended})")
        send_key(scan, extended)
        time.sleep(1.5)

def test_mode_3():
    """Test 3: LMU keyboard.json Direkt-Codes (rF2-Engine)"""
    print("\n📋 Test 3: LMU keyboard.json Codes")
    print("   Verwendet DIREKT die Zahlen aus LMU keyboard.json")
    
    # keyboard.json Werte direkt als Scancode
    tests = [
        ("Driving Cameras (210)", 210),
        ("Onboard Cameras (82)", 82),
        ("Swingman Camera (201)", 201),
        ("Tracking Cameras (209)", 209),
        ("Spectator Cameras (207)", 207),
    ]
    
    for name, code in tests:
        print(f"   🔄 {name} (Code={code} = 0x{code:02X})")
        send_key(code, extended=False)
        time.sleep(1.5)

def test_mode_4():
    """Test 4: rF2 bekannte Extended Scancodes"""
    print("\n📋 Test 4: rF2 Extended Scancodes")
    print("   Verwendet die rF2-üblichen Scancodes mit Extended-Flag")
    
    # rF2/lmu keyboard.json codes → hardware scancodes
    tests = [
        ("Driving (Insert): 210 → 0x52 ext", 0x52, True),
        ("Onboard (Home): 82 → 0x52 plain", 0x52, False),  # 82 = 0x52
        ("Swingman (PageUp): 201 → 0x49 ext", 0x49, True),
        ("Tracking (PageDown): 209 → 0x51 ext", 0x51, True),
        ("Spectator (End): 207 → 0x4F ext", 0x4F, True),
    ]
    
    for name, scan, extended in tests:
        print(f"   🔄 {name}")
        send_key(scan, extended)
        time.sleep(1.5)

def test_mode_5():
    """Test 5: SendInput + VK-Codes (z.B. F1..F6 wie in rF2)"""
    print("\n📋 Test 5: rF2 Standard-Kamera-Tasten")
    print("   rF2 Standard: F1=TV, F2=Onboard, F3=Nose, F4=Swing, F5=Track, F6=Behind")
    
    # VK_F1..VK_F6
    vk_keys = [
        ("F1 = TV", 0x70),
        ("F2 = Onboard", 0x71),
        ("F3 = Nose", 0x72),
        ("F4 = Swingman", 0x73),
        ("F5 = Tracking", 0x74),
        ("F6 = Behind", 0x75),
    ]
    
    for name, vk in vk_keys:
        print(f"   🔄 {name} (VK=0x{vk:02X})")
        # SendInput mit VK-Code (KEIN KEYEVENTF_SCANCODE)
        input_down = INPUT()
        input_down.type = INPUT_KEYBOARD
        input_down.ki.wVk = vk
        input_down.ki.dwFlags = 0  # no flags = VK mode
        input_down.ki.time = 0
        
        input_up = INPUT()
        input_up.type = INPUT_KEYBOARD
        input_up.ki.wVk = vk
        input_up.ki.dwFlags = KEYEVENTF_KEYUP
        input_up.ki.time = 0
        
        inputs = (INPUT * 2)(input_down, input_up)
        user32.SendInput(2, inputs, ctypes.sizeof(INPUT))
        time.sleep(1.5)

# ─── Hauptprogramm ──────────────────────────────────────────────────────

def main():
    print("=" * 60)
    print("  LMU KAMERA-TASTATUR-TEST")
    print("=" * 60)
    print()
    print("  WICHTIG: LMU-Fenster muss im VORDERGRUND sein!")
    print("  (Klicke auf das LMU-Fenster bevor du eine Taste drückst)")
    print()
    print("  Wähle einen Test-Modus:")
    print("  1 = Windows VK-Codes (keybd_event)")
    print("  2 = SendInput + PS/2 Scancodes (Insert, Home, PgUp, PgDn, End)")
    print("  3 = LMU keyboard.json Codes (direkt)")
    print("  4 = rF2 Extended Scancodes")
    print("  5 = rF2 Standard F1-F6 Kamera-Tasten")
    print("  Q = Beenden")
    print()
    
    while True:
        try:
            choice = input("  Deine Wahl (1-5, Q): ").strip().upper()
            
            if choice == 'Q':
                print("  Tschüss!")
                break
            elif choice == '1':
                test_mode_1()
            elif choice == '2':
                print("  ⏱️  Test 2 läuft (5 Kameras, 1.5s Pause)...")
                print("  👀 SCHAU AUFS LMU-FENSTER!")
                test_mode_2()
            elif choice == '3':
                print("  ⏱️  Test 3 läuft (5 Kameras, 1.5s Pause)...")
                print("  👀 SCHAU AUFS LMU-FENSTER!")
                test_mode_3()
            elif choice == '4':
                print("  ⏱️  Test 4 läuft (5 Kameras, 1.5s Pause)...")
                print("  👀 SCHAU AUFS LMU-FENSTER!")
                test_mode_4()
            elif choice == '5':
                print("  ⏱️  Test 5 läuft (F1-F6, 1.5s Pause)...")
                print("  👀 SCHAU AUFS LMU-FENSTER!")
                test_mode_5()
            else:
                print("  ❌ Ungültige Wahl. 1-5 oder Q.")
            
            print()
            
        except KeyboardInterrupt:
            print("\n  Abgebrochen.")
            break

if __name__ == "__main__":
    main()