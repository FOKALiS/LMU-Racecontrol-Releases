"""
Findet die exakten replayCommand-Befehle in BCUK-DLLs.
Sucht nach Strings in der Nähe von /rest/watch/replayCommand/ und _vcrCommandMap.
"""
import re
import sys

sys.stdout.reconfigure(encoding='utf-8', errors='replace')

FILES = [
    r"C:\Users\Administrator\Desktop\LMU Broadcast Control\BroadcastControl.Lmu.dll",
    r"C:\Users\Administrator\Desktop\LMU Broadcast Control\BroadcastControl.Core.dll",
    r"C:\Users\Administrator\Desktop\LMU Broadcast Control\LMU_Broadcast_Control.dll",
]

# Mögliche Command-Namen (C#-Style)
CMD_KEYWORDS = [
    b"enter", b"Enter", b"ENTER",
    b"exit", b"Exit", b"EXIT",
    b"toggle", b"Toggle", b"TOGGLE",
    b"active", b"Active", b"ACTIVE",
    b"play", b"Play", b"PLAY",
    b"pause", b"Pause", b"PAUSE",
    b"live", b"Live", b"LIVE",
    b"stop", b"Stop", b"STOP",
    b"start", b"Start", b"START",
    b"realtime", b"Realtime", b"REALTIME",
    b"seek", b"Seek", b"SEEK",
    b"time", b"Time",
    b"replay", b"Replay", b"REPLAY",
    b"begin", b"Begin",
    b"end", b"End",
    b"rewind", b"Rewind",
    b"forward", b"Forward",
    b"slow", b"Slow",
    b"fast", b"Fast",
    b"speed", b"Speed",
    b"1", b"2", b"3",
]

for filepath in FILES:
    print(f"\n{'='*60}")
    print(f"📄 {filepath.split(chr(92))[-1]}")
    print(f"{'='*60}")
    try:
        with open(filepath, "rb") as f:
            data = f.read()
        
        # 1. Nach /rest/watch/replayCommand/ suchen und Kontext danach ausgeben
        print("\n  --- Kontext von /rest/watch/replayCommand/ ---")
        for m in re.finditer(rb"/rest/watch/replayCommand/", data):
            start = m.start()
            end = min(len(data), m.end() + 200)
            context = data[start:end]
            # UTF-16 oder ASCII?
            print(f"  @0x{start:06X}: {context[:200]}")
        
        # 2. Alle UTF-16 Strings (C# const strings) suchen, die klein geschrieben sind und wie Befehle aussehen
        print("\n  --- Mögliche Command-Strings ---")
        strings_utf16 = re.findall(rb"(?:[\x20-\x7e]\x00){3,}", data)
        commands = set()
        for s in strings_utf16:
            text = s.decode("utf-16-le", errors="replace").strip()
            # Befehle sind normalerweise kurz, ohne Leerzeichen, Dateinamen, Pfade
            if (1 < len(text) < 30 and 
                not " " in text and 
                not "/" in text and 
                not "\\" in text and
                not text.startswith(".") and
                not text.startswith("<") and
                not text.startswith("{") and
                not text.startswith("[") and
                not text.startswith("#") and
                not text.startswith("//") and
                not text.startswith("/*") and
                text.isascii()):
                # Nur Wörter mit typischen Command-Suffixen
                if re.search(r"(eplay|ime|ive|ause|top|lay|ind|ward|egin|nd|ctive|oggle|enter|exit)", text, re.IGNORECASE):
                    commands.add(text)
        
        if commands:
            print(f"  Gefunden ({len(commands)}):")
            for c in sorted(commands)[:60]:
                print(f"    {c}")
        else:
            print("  Keine gefunden")
        
    except Exception as e:
        print(f"  ❌ Fehler: {e}")

print(f"\n{'='*60}")
print("Fertig.")