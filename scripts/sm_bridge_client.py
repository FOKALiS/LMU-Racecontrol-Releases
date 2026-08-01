"""
SM Bridge Client (WebSocket) – zeigt nur die wichtige Struktur.
"""
import json, sys, subprocess, os

PYTHON_310 = r"C:\Users\Administrator\AppData\Local\Programs\Python\Python310\python.exe"

def import_websocket():
    try:
        import websocket
        return websocket
    except ImportError:
        pass
    try:
        result = subprocess.run(
            [PYTHON_310, "-c", "import websocket; print('ok')"],
            capture_output=True, text=True, timeout=5
        )
        if result.returncode == 0:
            site_packages = subprocess.run(
                [PYTHON_310, "-c", "import site; print(site.getsitepackages()[0])"],
                capture_output=True, text=True, timeout=5
            ).stdout.strip()
            if site_packages:
                sys.path.insert(0, site_packages)
                import websocket
                return websocket
    except:
        pass
    print("websocket-client nicht gefunden!")
    sys.exit(1)

websocket = import_websocket()

def on_message(ws, message):
    try:
        data = json.loads(message)
        t = data.get("t", "")
        seq = data.get("seq", 0)
        
        if t == "snapshot":
            scoring = data.get("data", {}).get("scoring", {})
            print(f"\n[snapshot #{seq}] gamePhase={scoring.get('gamePhase')} yellow={scoring.get('yellowFlagState')}")
            # Alle Keys des Scoring-Objekts anzeigen
            for k, v in sorted(scoring.items()):
                print(f"  {k}: {v}")
            # Fahrzeugliste
            vehicles = data.get("data", {}).get("vehicles", [])
            for v in vehicles[:5]:
                print(f"    #{v.get('id','?')} {v.get('driverName','?')[:20]:20s} speed={v.get('speed',0):.1f}")
            
        elif t == "positions":
            print(f"\n[positions #{seq}] {len(data.get('data',[]))} Fahrzeuge")
            for v in data.get("data", [])[:3]:
                print(f"  id={v.get('id')} x={v.get('x',0):.1f} z={v.get('z',0):.1f} pitting={v.get('p')}")
        else:
            print(f"\n[unknown #{seq}] t={t} keys={list(data.keys())}")
    except Exception as e:
        print(f"Fehler: {e}")

def on_open(ws):
    print("✅ Verbunden! Empfange Daten... (STRG+C zum Beenden)")
    print()

ws = websocket.WebSocketApp("ws://localhost:5200/sm",
    on_open=on_open, on_message=on_message)
ws.run_forever()