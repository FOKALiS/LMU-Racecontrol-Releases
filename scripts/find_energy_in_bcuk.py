"""
Durchsucht BCUK-DLLs nach Energie/Hybrid-Strings.
Ziel: Finden, ob LMU "Virtuelle Energie" (Virtual Energy) preisgibt.
"""
import re
import sys

sys.stdout.reconfigure(encoding="utf-8", errors="replace")

FILES = [
    r"C:\Users\Administrator\Desktop\LMU Broadcast Control\BroadcastControl.Lmu.dll",
    r"C:\Users\Administrator\Desktop\LMU Broadcast Control\BroadcastControl.Core.dll",
    r"C:\Users\Administrator\Desktop\LMU Broadcast Control\LMU_Broadcast_Control.dll",
]

KEYWORDS = [
    b"energy", b"Energy", b"ENERGY",
    b"hybrid", b"Hybrid", b"HYBRID",
    b"virtual", b"Virtual", b"VIRTUAL",
    b"kwh", b"kWh", b"KWH",
    b"battery", b"Battery", b"BATTERY",
    b"fuel", b"Fuel", b"FUEL",
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

        # Direkt nach Keywords suchen und Kontext ausgeben
        found = set()
        for kw in KEYWORDS:
            for m in re.finditer(re.escape(kw), data):
                start = max(0, m.start() - 60)
                end = min(len(data), m.end() + 80)
                context = data[start:end]
                printable = re.findall(b"[ -~]+", context)
                for p in printable:
                    if len(p) > 3:
                        found.add(p.decode(errors="replace"))

        if found:
            print(f"\n  🔍 Gefundene Strings ({len(found)}):")
            for s in sorted(found):
                print(f"    {s}")
        else:
            print("\n  ⚠️ Keine Energie/Hybrid-Strings gefunden")

    except Exception as e:
        print(f"  ❌ Fehler: {e}")

print(f"\n{'='*60}")
print("Fertig.")