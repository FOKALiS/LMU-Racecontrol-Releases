#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Zeigt die rohe sessionInfo JSON-Antwort von LMU an.
Damit sehen wir, welche Felder fuer die Session-Zeit verfuegbar sind.
"""
import urllib.request
import json

def rest_get(path):
    try:
        with urllib.request.urlopen(f"http://localhost:6397{path}", timeout=5) as resp:
            return resp.read().decode()
    except Exception as e:
        print(f"Fehler: {e}")
        return None

print("=" * 60)
print("LMU sessionInfo - Rohe JSON-Antwort")
print("=" * 60)

session = rest_get("/rest/watch/sessionInfo")
if session:
    print("\nRohe JSON:")
    print(session)
    print("\n--- Formatiert ---")
    try:
        data = json.loads(session)
        print(json.dumps(data, indent=2, ensure_ascii=False))
        
        # Alle Keys auflisten
        print("\n--- Alle Top-Level Keys ---")
        for key in data.keys():
            val = data[key]
            print(f"  {key}: {type(val).__name__} = {val}")
    except:
        print("Konnte JSON nicht parsen")
else:
    print("\nFEHLER: LMU laeuft nicht oder API nicht erreichbar.")
    print("Bitte LMU starten und im Rennen sein, dann dieses Skript erneut ausfuehren.")