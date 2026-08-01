# LMU RACECONTROL – Projekt-Zusammenfassung für neuen Chat

## Projekt
**LMU RACECONTROL** – Tauri-basiertes Desktop-Tool für Le Mans Ultimate Rennkommissionen.
- **Pfad:** `C:\Users\Administrator\Documents\AI\Software Entwicklung\LMU Racecontrol\Tool\lmu-race-control`
- **Aktuelle Version:** v0.7.8 (Build 4:45 Uhr, funktioniert stabil)
- **Build-Workflow:** https://github.com/FOKALiS/LMU-Racecontrol/actions
- **Release-Versionierung:** Zweite Kommastelle erhöhen (z. B. 0.7.x → 0.8.0)
- **WICHTIG:** Nur lokale Builds & Releases – KEINE GitHub-Pushes!
- **User ist Grafik Designer, kein Coder** – Schritt-für-Schritt-Anleitung nötig

## Wichtige Pfade
- **LMU-Installation:** `C:\Program Files (x86)\Steam\steamapps\common\Le Mans Ultimate`
- **BCUK (Broadcast Control UK):** `C:\Users\Administrator\Desktop\LMU Broadcast Control\`
  - DLLs: `BroadcastControl.Lmu.dll`, `BroadcastControl.Core.dll`, `LMU_Broadcast_Control.dll`
- **LMU REST-API:** `http://localhost:6397`
- **LMU WebSocket:** `ws://localhost:6398/websocket/replaymetrics`
- **Shared Memory Name:** `LMU_Data` (aus BCUK DLLs bestätigt)

## FUNKTIONIERT ✅ (Stand v0.7.8)

### Kamera-Steuerung
- **3 Kamera-Buttons:** "Bord", "Heli", "TV" (vorher "Helmet" – matched in Rust korrekt)
- **Tastatur-Simulation (SendInput/Scancodes)** statt REST-API (REST gab 200, tat aber nichts)
- **Zoom + / Zoom -** als Dauer-Zoom (KP7/KP9, 15ms Intervall)
- Keyboard-Layout wird aus **`keyboard.json`** gelesen (LMU Tastenbelegung)
- **AKTUELLER STAND:** "Bord" matched in Rust, 1 Hintergrund-Thread statt setInterval-Flut, KP9 Scancode 0x49

### Auge-Symbol (jumpToReplay) – funktioniert im Ansatz
- **Ablauf:** R-Taste (Replay aktivieren) → 1s warten → Zeitsprung API → 500ms warten → F11 (Play) → **Zeitsprung wiederholen** (weil F11 Position auf 0:00 zurücksetzt!) → Fahrer-Fokus
- **KEINE Kamera-Änderung beim Auge-Klick!** Der Rennkommissar behält seine Kamera
- Funktioniert im **Fahrerfeld** und in den **Vorfällen**

### Replay-API – WICHTIGE ENTDECKUNG
- **Die REST-API (replaytime, replayCommand) funktioniert NUR, wenn ein WebSocket-Client auf `ws://localhost:6398/websocket/replaymetrics` verbunden ist!**
- BCUK hält diese Verbindung immer offen – deshalb funktioniert es dort
- **`src-tauri/src/lmu_ws.rs`** = WebSocket-Client, startet beim Connect-Button
- Liefert Replay-Metadaten: `currentReplayPos`, `lapNumber`, `lapTimestamps`, `gear`, `brakes`, etc.
- Zeitsprung-API: `PUT http://localhost:6397/rest/watch/replaytime/{sekunden}` (funktioniert!)
- WebSocket liefert `{"type":"replayMetrics","body":{...}}`

### Vor-/Nachlaufzeit – synchronisiert
- **Standardwerte: 20 Sekunden / 20 Sekunden** (`pre_roll_seconds` / `post_roll_seconds`)
- Slider in den **Vorfällen** speichern direkt in die Settings (via `save_settings`)
- **Einstellungen** und **Vorfälle**-Ansicht sind synchron
- Auto-Stop nach Nachlaufzeit (F6-Taste) via Timer

### Weitere funktionierende Features
- Fahrer-Fokus per Klick auf Fahrerzeile (REST-API PUT `/rest/watch/focus/{slotID}`)
- FCY-Überwachung mit roten Verstößen
- Speed-Anzeige
- Connect/Disconnect
- Lizenzsystem
- Standings-Updates per Polling (1s Intervall)

## FUNKTIONIERT NICHT ❌ / OFFEN

1. **Replay-Zeit:** Zeitsprung funktioniert, aber Nutzer muss noch genauer testen (v0.7.8 mit Doppel-Zeitsprung ist der aktuellste Fix – noch nicht ausführlich getestet)
2. **Shared Memory Offsets:** Aktuelle Offsets 0x24/0x28 sind für rFactor2, LMU braucht andere. Nicht aktiv genutzt.
3. **Kamera via Tastatur** erzwingt kurz den LMU-Fokus (Fenster in den Vordergrund) – funktioniert aber

## WICHTIGE TECHNISCHE ERKENNTNISSE

### BCUK-DLL-Analyse (Suchergebnisse)
- BCUK nutzt **dieselben REST-Pfade** wie wir:
  - `/rest/watch/replayCommand/{cmd}` (enter, replay, play, pause, toggleactive, live)
  - `/rest/watch/replaytime/{sekunden}`
  - `/rest/replay/isActive`, `/rest/replay/toggleactive`
  - `/rest/watch/focus/`
  - `/websocket/replaymetrics` (WebSocket auf Port 6398)
- **WebSocket-Client-Frames MÜSSEN maskiert sein!** (RFC 6455) – sonst: "clients may not send unmasked frames"

### REST-API Feldnamen (bestätigt)
- slotID, driverName, fullTeamName, carClass, lapsCompleted, lastLapTime, bestLapTime, pitting, carVelocity.velocity (m/s)

### LMU Tastenbelegung (keyboard.json)
- R = Replay aktivieren
- F11 = Replay Play
- F6 = Replay Stop
- KP7 / KP9 = Zoom + / Zoom -
- TV = Kamera TV, F2-Artige Tasten für Kameras (aus keyboard.json)

## WICHTIGE SKRIPTE (scripts/ Ordner)
- `test_replaytime_metrics.py` – prüft ob Zeitsprung funktioniert (WebSocket + REST gleichzeitig)
- `search_bcuk_urls.py` / `search_bcuk_commands.py` – analysiert BCUK DLLs
- `test_websocket_commands.py` – maskierte WebSocket-Befehle senden
- `dump_shared_memory.py`, `read_lmu_sm.py` – Shared Memory Analyse

## WICHTIGE RUST-DATEIEN
- `src-tauri/src/main.rs` – `jump_to_incident_replay` Command (Doppel-Zeitsprung-Logik)
- `src-tauri/src/lmu_ws.rs` – WebSocket-Client (aktiviert Replay-API)
- `src-tauri/src/keyboard.rs` – Tastatur-Simulation (SendInput/Scancodes)
- `src-tauri/src/settings.rs` – Settings (pre_roll_seconds=20, post_roll_seconds=20)
- `src-tauri/src/lmu_client.rs` – REST-Client (fetch, put, seek_replay_to)

## NÄCHSTE SCHRITTE
1. **v0.7.8 ausführlich testen** (Replay-Zeit prüfen, Kamera-Buttons, Auto-Stop)
2. Korrekte LMU Shared Memory Offsets finden (BCUK DLLs auf Desktop oder LMU-EXE) – falls benötigt
3. Verfeinern der Replay-Funktion nach Testergebnissen
4. Build als nächste Version (z.B. v0.7.9)

## BUILD-Prozess
```bash
cd "C:\Users\Administrator\Documents\AI\Software Entwicklung\LMU Racecontrol\Tool\lmu-race-control"
npx tauri build
```
- Installer: `src-tauri\target\release\bundle\nsis\LMU RACECONTROL_0.7.8_x64-setup.exe`
- MSI: `src-tauri\target\release\bundle\msi\LMU RACECONTROL_0.7.8_x64_en-US.msi`
- "A public key found, but no private key" Fehler = nur für Auto-Update, Installer funktionieren trotzdem
- Vor Build: Version in `src-tauri/Cargo.toml`, `package.json`, `src-tauri/tauri.conf.json` anpassen