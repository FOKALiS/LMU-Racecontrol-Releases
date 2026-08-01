"""
Durchsucht BCUK-DLLs nach WebSocket/Replay-Befehlen.
"""
import re
import sys

FILES = [
    r"C:\Users\Administrator\Desktop\LMU Broadcast Control\BroadcastControl.Lmu.dll",
    r"C:\Users\Administrator\Desktop\LMU Broadcast Control\BroadcastControl.Core.dll",
    r"C:\Users\Administrator\Desktop\LMU Broadcast Control\LMU_Broadcast_Control.dll",
]

KEYWORDS = [
    b"replay", b"Replay", b"REPLAY",
    b"websocket", b"WebSocket", b"Websocket",
    b"ws://", b"wss://",
    b"6398", b"6397",
    b"time", b"Time",
    b"live", b"Live",
    b"play", b"Play",
    b"pause", b"Pause",
]

def find_strings(data, pattern=b"[ -~]{4,}"):
    return re.findall(pattern, data)

for filepath in FILES:
    print(f"\n{'='*60}")
    print(f"📄 {filepath.split(chr(92))[-1]}")
    print(f"{'='*60}")
    try:
        with open(filepath, "rb") as f:
            data = f.read()
        print(f"  Größe: {len(data):,} Bytes")
        
        # Nach Keywords suchen
        found = set()
        for kw in KEYWORDS:
            for m in re.finditer(re.escape(kw), data):
                start = max(0, m.start() - 40)
                end = min(len(data), m.end() + 60)
                context = data[start:end]
                # Nur druckbare Zeichen behalten
                printable = re.findall(b"[ -~]+", context)
                for p in printable:
                    if len(p) > 3:
                        found.add(p.decode(errors="replace"))
        
        if found:
            print(f"\n  🔍 Gefundene Strings ({len(found)}):")
            for s in sorted(found):
                print(f"    {s}")
        else:
            print("\n  ⚠️ Keine relevanten Strings gefunden")
            
    except Exception as e:
        print(f"  ❌ Fehler: {e}")

print(f"\n{'='*60}")
print("Fertig.")