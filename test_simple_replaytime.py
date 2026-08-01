#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Einfacher Test der replaytime API ohne Emojis
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
print("Einfacher replaytime Test")
print("=" * 60)

# 1. Prüfen ob LMU läuft
print("\n1. Prüfe LMU Verbindung...")
session = rest_get("/rest/watch/sessionInfo")
if session:
    print("OK - LMU läuft")
else:
    print("FEHLER - LMU läuft nicht auf Port 6397")
    exit(1)

# 2. Replay aktivieren
print("\n2. Aktiviere Replay-Modus...")
status = rest_put("/rest/watch/replayCommand/replay")
print(f"Status: {status}")
time.sleep(2)

# 3. Zeitsprung auf 100 Sekunden
print("\n3. Zeitsprung auf 100 Sekunden...")
status = rest_put("/rest/watch/replaytime/100")
print(f"Status: {status}")

# 4. Zeitsprung auf 200 Sekunden
print("\n4. Zeitsprung auf 200 Sekunden...")
status = rest_put("/rest/watch/replaytime/200")
print(f"Status: {status}")

# 5. Zeitsprung auf 50.5 Sekunden (mit Dezimalstelle)
print("\n5. Zeitsprung auf 50.5 Sekunden...")
status = rest_put("/rest/watch/replaytime/50.5")
print(f"Status: {status}")

print("\n" + "=" * 60)
print("Test abgeschlossen. Schaue in LMU ob die Zeitsprünge funktioniert haben.")
print("=" * 60)