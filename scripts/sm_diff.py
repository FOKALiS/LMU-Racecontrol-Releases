"""
SM Diff – Vergleicht Bridge-Daten vor/nach Kamerawechsel.
Starte: python scripts/sm_diff.py
Dann starte BCUK und klicke auf Kamera-Buttons.
"""
import json, sys, subprocess, os, time

PYTHON_310 = r"C:\Users\Administrator\AppData\Local\Programs\Python\Python310\python.exe"

def import_websocket():
    try:
        import websocket
        return websocket
    except ImportError:
        pass
    try:
        r = subprocess.run([PYTHON_310, "-c", "import websocket; print('ok')"], capture_output=True, text=True, timeout=5)
        if r.returncode == 0:
            sp = subprocess.run([PYTHON_310, "-c", "import site; print(site.getsitepackages()[0])"], capture_output=True, text=True, timeout=5).stdout.strip()
            if sp:
                sys.path.insert(0, sp)
                import websocket
                return websocket
    except:
        pass
    print("websocket-client nicht gefunden!")
    sys.exit(1)

websocket = import_websocket()

snapshots = []
capturing = False

def on_message(ws, message):
    global snapshots, capturing
    try:
        data = json.loads(message)
        if data.get("t") == "snapshot":
            snapshots.append(data)
            
            if not capturing:
                print(f"  Snapshot #{data['seq']} empfangen ({len(snapshots)} gespeichert)")
            
            if len(snapshots) >= 5 and not capturing:
                capturing = True
                print(f"\n{'='*60}")
                print(f"  5 Snapshots gespeichert!")
                print(f"  Jetzt in BCUK auf 'TV' klicken (Kamera wechseln)!")
                print(f"  Dann ENTER drücken...")
                print(f"{'='*60}")
                input()
                
                # Noch 5 weitere Snapshots sammeln
                print("  Sammle 5 Snapshots nach Kamerawechsel...")
                time.sleep(3)
                
                # Vergleiche
                print(f"\n{'='*60}")
                print(f"  VERGLEICH: Vorher vs Nachher")
                print(f"{'='*60}")
                
                before = snapshots[:5]
                after = snapshots[-5:]
                
                # Vergleiche Scoring
                b_scoring = before[0].get("data", {}).get("scoring", {})
                a_scoring = after[0].get("data", {}).get("scoring", {})
                
                print(f"\n  Scoring Änderungen:")
                for key in set(list(b_scoring.keys()) + list(a_scoring.keys())):
                    bv = b_scoring.get(key)
                    av = a_scoring.get(key)
                    if bv != av:
                        print(f"    {key}: {bv} -> {av}")
                
                # Vergleiche Telemetrie (erste 3 Fahrzeuge)
                b_tele = before[0].get("data", {}).get("telemetry", [])
                a_tele = after[0].get("data", {}).get("telemetry", [])
                
                print(f"\n  Telemetrie Änderungen (erste 3 Fahrzeuge):")
                for i in range(min(3, len(b_tele), len(a_tele))):
                    bt = b_tele[i]
                    at = a_tele[i]
                    for key in set(list(bt.keys()) + list(at.keys())):
                        bv = bt.get(key)
                        av = at.get(key)
                        if bv != av:
                            print(f"    Fahrzeug {bt.get('id')}.{key}: {bv} -> {av}")
                
                # Vergleiche trackPos
                b_tp = before[0].get("data", {}).get("trackPos", [])
                a_tp = after[0].get("data", {}).get("trackPos", [])
                
                print(f"\n  trackPos Änderungen (erste 3 Fahrzeuge):")
                for i in range(min(3, len(b_tp), len(a_tp))):
                    bt = b_tp[i]
                    at = a_tp[i]
                    for key in set(list(bt.keys()) + list(at.keys())):
                        bv = bt.get(key)
                        av = at.get(key)
                        if bv != av:
                            print(f"    Fahrzeug {bt.get('id')}.{key}: {bv} -> {av}")
                
                print(f"\n  Fertig! Keine weiteren Kamera-spezifischen Felder gefunden.")
                ws.close()
                
    except Exception as e:
        print(f"Fehler: {e}")

def on_open(ws):
    print("✅ Verbunden! Sammle Snapshots...")

print("=" * 60)
print("  SM DIFF – Bridge-Daten vor/nach Kamerawechsel")
print("=" * 60)
print()

ws = websocket.WebSocketApp("ws://localhost:5200/sm", on_open=on_open, on_message=on_message)
ws.run_forever()