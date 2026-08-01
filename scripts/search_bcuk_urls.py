"""
Extrahiert ALLE URL-Pfade aus BCUK-DLLs – fokussiert auf /rest/ und /watch/.
"""
import re
import sys

sys.stdout.reconfigure(encoding='utf-8', errors='replace')

FILES = [
    r"C:\Users\Administrator\Desktop\LMU Broadcast Control\BroadcastControl.Lmu.dll",
    r"C:\Users\Administrator\Desktop\LMU Broadcast Control\BroadcastControl.Core.dll",
    r"C:\Users\Administrator\Desktop\LMU Broadcast Control\LMU_Broadcast_Control.dll",
]

for filepath in FILES:
    print(f"\n{'='*60}")
    print(f"📄 {filepath.split(chr(92))[-1]}")
    print(f"{'='*60}")
    try:
        with open(filepath, "rb") as f:
            data = f.read()
        
        # Nur UTF-16 Strings (C# speichert Strings als UTF-16 im .NET)
        strings_utf16 = re.findall(rb"(?:[\x20-\x7e]\x00){3,}", data)
        
        paths = set()
        for s in strings_utf16:
            text = s.decode("utf-16-le", errors="replace").strip()
            # Nur wenn es wie ein Pfad aussieht
            if text.startswith("/") and ("rest" in text or "watch" in text or "replay" in text or "focus" in text or "time" in text or "live" in text or "camera" in text):
                paths.add(text)
            # Auch URLs
            if "://" in text and ("rest" in text or "watch" in text):
                paths.add(text)
        
        if paths:
            print(f"\n  🔍 Gefundene Pfade ({len(paths)}):")
            for p in sorted(paths):
                print(f"    {p}")
        else:
            print("\n  ⚠️ Keine Pfade gefunden – suche in ASCII...")
            strings_ascii = re.findall(rb"[\x20-\x7e]{8,}", data)
            for s in strings_ascii:
                text = s.decode(errors="replace")
                if text.startswith("/rest") or text.startswith("/watch") or "replaytime" in text or "replayCommand" in text:
                    paths.add(text)
            if paths:
                print(f"  Gefunden ({len(paths)}):")
                for p in sorted(paths):
                    print(f"    {p}")
            else:
                print("  ⚠️ Auch keine ASCII-Pfade")
        
    except Exception as e:
        print(f"  ❌ Fehler: {e}")

print(f"\n{'='*60}")
print("Fertig.")