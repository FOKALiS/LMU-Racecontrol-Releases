#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Test der BCUK-spezifischen Replay-Befehle
"""
import urllib.request
import time

def rest_put(path):
    req = urllib.request.Request(f"http://localhost:6397{path}", method="PUT")
    req.add_header("Content-Type", "application/json")
    try:
        with urllib.request.urlopen(req, timeout=5) as resp:
            return resp.status
    except Exception as e:
        print(f"Fehler: {e}")
        return 0

def rest_get(path):
    try:
        with urllib.request.urlopen(f"http://localhost:6397{path}", timeout=5) as resp:
            return resp.read().decode()
    except Exception as e:
        print(f"Fehler: {e}")
        return None

print("=" * 60)
print("BCUK-spezifische Replay-Befehle testen")
print("=" * 60)

# 1. Prüfen ob LMU läuft
print("\n1. Prüfe LMU Verbindung...")
session = rest_get("/rest/watch/sessionInfo")
if session:
    print("OK - LMU läuft")
else:
    print("FEHLER - LMU läuft nicht auf Port 6397")
    exit(1)

# 2. PreArmReplay - BCUK-spezifischer Befehl!
print("\n2. PreArmReplay (BCUK-spezifisch)...")
status = rest_put("/rest/watch/replayCommand/PreArmReplay")
print(f"Status: {status}")
time.sleep(1)

# 3. Replay aktivieren
print("\n3. Aktiviere Replay-Modus...")
status = rest_put("/rest/watch/replayCommand/replay")
print(f"Status: {status}")
time.sleep(2)

# 4. Zeitsprung auf 100 Sekunden
print("\n4. Zeitsprung auf 100 Sekunden...")
status = rest_put("/rest/watch/replaytime/100")
print(f"Status: {status}")
time.sleep(1)

# 5. VCRCOMMAND_PLAY statt normalem play
print("\n5. VCRCOMMAND_PLAY (BCUK-spezifisch)...")
status = rest_put("/rest/watch/replayCommand/VCRCOMMAND_PLAY")
print(f"Status: {status}")
time.sleep(1)

# 6. Nochmal Zeitsprung nach Play
print("\n6. Zeitsprung wiederholen auf 100 Sekunden...")
status = rest_put("/rest/watch/replaytime/100")
print(f"Status: {status}")

print("\n" + "=" * 60)
print("BCUK-Test abgeschlossen. Schaue in LMU ob es funktioniert hat.")
print("=" * 60)