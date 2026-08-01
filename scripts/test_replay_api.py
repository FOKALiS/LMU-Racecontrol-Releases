"""
Testet die LMU REST-API auf Port 6398 (BCUK-Port).
ACHTUNG: BCUK läuft bereits auf Port 6398 – wir lesen NUR, schreiben nichts!
Starte mit LMU + BCUK in einer Session.
"""
import urllib.request
import urllib.error
import json

def get_json(path, port):
    url = f"http://localhost:{port}{path}"
    try:
        with urllib.request.urlopen(url, timeout=5) as resp:
            return json.loads(resp.read().decode())
    except Exception as e:
        return None

print("=" * 60)
print("LMU REST-API – Port-Vergleich 6397 vs 6398")
print("=" * 60)

# sessionInfo auf BEIDEN Ports lesen
print("\n--- sessionInfo auf Port 6397 (unsere API) ---")
info_6397 = get_json("/rest/watch/sessionInfo", 6397)
if info_6397:
    print(f"  inRealtime: {info_6397.get('inRealtime')}")
    print(f"  session: {info_6397.get('session')}")
    print(f"  currentEventTime: {info_6397.get('currentEventTime')}")
    print(f"  gamePhase: {info_6397.get('gamePhase')}")
else:
    print("  ❌ Keine Antwort")

print("\n--- sessionInfo auf Port 6398 (BCUK-Port) ---")
info_6398 = get_json("/rest/watch/sessionInfo", 6398)
if info_6398:
    print(f"  inRealtime: {info_6398.get('inRealtime')}")
    print(f"  session: {info_6398.get('session')}")
    print(f"  currentEventTime: {info_6398.get('currentEventTime')}")
    print(f"  gamePhase: {info_6398.get('gamePhase')}")
else:
    print("  ❌ Keine Antwort")

# JETZT: In BCUK auf Replay klicken, dann ENTER drücken
input("\n⏳ Jetzt in BCUK auf einen Vorfall klicken (Replay starten), dann ENTER drücken...")

print("\n--- sessionInfo auf Port 6397 NACH Replay ---")
info_6397b = get_json("/rest/watch/sessionInfo", 6397)
if info_6397b:
    print(f"  inRealtime: {info_6397b.get('inRealtime')}")
    print(f"  currentEventTime: {info_6397b.get('currentEventTime')}")

print("\n--- sessionInfo auf Port 6398 NACH Replay ---")
info_6398b = get_json("/rest/watch/sessionInfo", 6398)
if info_6398b:
    print(f"  inRealtime: {info_6398b.get('inRealtime')}")
    print(f"  currentEventTime: {info_6398b.get('currentEventTime')}")

print("\n" + "=" * 60)
print("Fertig.")
print("=" * 60)