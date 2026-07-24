## Neu

- **Shared Memory (rFactor 2/LMU)**: Direkter Zugriff auf den LMU Shared Memory (`Local\rFactor2SharedMemory`). Kamera-Wechsel und Fahrzeug-Fokus funktionieren jetzt **ohne Fenster-Fokus, ohne Tastatur-Simulation, ohne Terminal-Flash** - wie bei Broadcast Control UK, SimHub und anderen professionellen Tools
- **Connect/Disconnect Hover**: Wenn verbunden, wird beim Überfahren mit der Maus "Disconnect from Server" (rot) angezeigt - Klick trennt die Verbindung

## Behoben

- **FCY-Überwachung aktiviert**: Bei Überschreitung von Limit + 3 km/h Toleranz (z.B. 60+3=63 km/h) wird automatisch ein FCY-Verstoß-Vorfall erstellt
- **Debug-Logging**: Die API-Antwort von `/rest/watch/standings` wird jetzt ausgegeben, um die echten Feldnamen für `speed_kmh` zu identifizieren

## Geändert

- `shared_memory.rs`: Neues Modul - schreibt Kamera-Werte direkt in den LMU Shared Memory (Gruppe + Kamera-ID)
- `keyboard.rs`: Wird nur noch als Fallback verwendet, wenn Shared Memory nicht verfügbar ist
- `lmu_client.rs`: `groundSpeed` als zusätzlicher Feldname für `speed_kmh`, Debug-Logging
- `main.rs`: `set_camera` versucht zuerst Shared Memory, dann Tastatur-Fallback
- `App.tsx` + `Sidebar.tsx`: Connect/Disconnect mit Hover-Effekt
- `translations.ts`: Neue Texte `server_disconnect` / `server_disconnected`
- **Version**: 0.6.6 → 0.6.7