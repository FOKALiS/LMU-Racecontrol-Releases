"""
Dump Bridge JSON – Schreibt die komplette JSON-Struktur in eine Datei.
Führe aus: python scripts/dump_bridge_json.py
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

outfile = os.path.join(os.path.dirname(__file__), "bridge_dump.json")
snapshot_file = os.path.join(os.path.dirname(__file__), "bridge_snapshot_full.json")
dumped = False

def on_message(ws, message):
    global dumped
    try:
        data = json.loads(message)
        t = data.get("t", "")
        
        # Schreibe JEDE Nachricht in die Sammeldatei
        with open(outfile, "a") as f:
            f.write(json.dumps(data, indent=2) + "\n\n---\n\n")
        
        # Ersten vollständigen Snapshot speichern
        if t == "snapshot" and not dumped:
            with open(snapshot_file, "w") as f:
                json.dump(data, f, indent=2)
            print(f"✅ Erster Snapshot gespeichert: {snapshot_file}")
            print(f"   Alle Nachrichten: {outfile}")
            print(f"\n   WICHTIG: Jetzt in BCUK auf 'TV' klicken (Kamera wechseln)!")
            print(f"   Dann ENTER drücken...")
            input()
            print("   Beende...")
            ws.close()
            dumped = True
    except:
        pass

def on_open(ws):
    print("✅ Verbunden! Warte auf ersten Snapshot...")

ws = websocket.WebSocketApp("ws://localhost:5200/sm", on_open=on_open, on_message=on_message)
ws.run_forever()