# LMU RACECONTROL – Projekt-Zusammenfassung für neuen Chat

## Projekt
**LMU RACECONTROL** – Tauri-basiertes Desktop-Tool für Le Mans Ultimate Rennkommissionen.
- **Pfad:** `C:\Users\Administrator\Documents\AI\Software Entwicklung\LMU Racecontrol\Tool\lmu-race-control`
- **Aktuelle Version:** v0.8.5
- **Build-Workflow:** https://github.com/FOKALiS/LMU-Racecontrol/actions
- **Release-Versionierung:** Zweite Kommastelle erhöhen (z. B. 0.8.x → 0.8.6)
- **WICHTIG:** Nur lokale Builds & Releases – KEINE GitHub-Pushes!
- **User ist Grafik Designer, kein Coder** – Schritt-für-Schritt-Anleitung nötig
- **Letzter Commit:** `c4b20d8eb4a745358a13c93a1db54e8a1a357a8b`

## Wichtige Pfade
- **LMU-Installation:** `C:\Program Files (x86)\Steam\steamapps\common\Le Mans Ultimate`
- **BCUK (Broadcast Control UK):** `C:\Users\Administrator\Desktop\LMU Broadcast Control\`
  - DLLs: `BroadcastControl.Lmu.dll`, `BroadcastControl.Core.dll`, `LMU_Broadcast_Control.dll`
- **LMU REST-API:** `http://localhost:6397`
- **LMU WebSocket:** `ws://localhost:6398/websocket/replaymetrics`
- **Shared Memory Name:** `LMU_Data` (aus BCUK DLLs bestätigt)
- **Figma-Screenshots:** `figma-screens/` (01-splashscreen.png bis 06-einstellungen.png)
- **Logo-Quellen:**
  - `C:\Users\Administrator\Documents\AI\Software Entwicklung\LMU Racecontrol\Logo\LMU RC Logo - hell.png` (1024x486, breites Banner)
  - `C:\Users\Administrator\Documents\AI\Software Entwicklung\LMU Racecontrol\Logo\LMU RC - Icon transparent.png` (1024x1024, Flaggen-Symbol)

## FUNKTIONIERT ✅ (Stand v0.8.5)

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

## ICON-PROBLEM GELÖST ✅ (v0.8.5, Chat vom 04.08.2026)

### Ursache (2 Probleme kombiniert)
1. **Fehlende Windows-Skalierungsgrößen** – Die ICO hatte nur 7 Größen (16, 24, 32, 48, 64, 128, 256). Windows braucht aber auch **20, 40 und 96** für DPI-Skalierungen (100%, 125%, 150%, 175%, 200%).
2. **Windows Icon-Cache** – Selbst mit der richtigen ICO zeigte Windows das alte, verpixelte Icon aus dem Cache. Erst ein **Neustart** hat den Cache geleert.

### Lösung
- **`icon.ico`** – Neu erstellt mit **10 Größen**: 16, 20, 24, 32, 40, 48, 64, 96, 128, 256
- **Alle PNGs** (`32x32.png`, `128x128.png`, `128x128@2x.png`, `icon.png`) – Mit dem LMU RC Logo generiert
- **`tauri.conf.json`** – `icon.ico` an erster Stelle der Icon-Liste, `installerIcon` gesetzt
- **`build.rs`** – `rerun-if-changed` für Icons hinzugefügt

### Wichtige Erkenntnisse zum Icon-Problem
- **`app.windows[].icon` existiert NICHT in Tauri v2** – "Additional properties are not allowed ('icon' was unexpected)"
- **`app.set_icon()` existiert NICHT auf `tauri::App`** – nur `Image::new_owned(rgba, w, h)` für Laufzeit-Icons
- **PIL ICO-Erstellung:** `img.save('icon.ico', format='ICO', sizes=[(s,s) for s in sizes], append_images=[...])` – Standard-PIL-Format
- **Windows liest `System.Drawing.Icon` immer als 32x32** – das ist normal, prüft die tatsächlichen Größen mit PIL: `Image.open('icon.ico').info.get('sizes')`
- **BMP-basierte ICO (372KB)** vs. **PIL-ICO (60-75KB):** Die BMP-ICO wurde nicht verwendet, PIL-ICO mit 10 Größen hat funktioniert
- **Icon-Cache:** `%LocalAppData%\IconCache.db` und `%LocalAppData%\Microsoft\Windows\Explorer\iconcache_*.db` – nach Installation **PC neu starten** zum Leeren!

### Verifikation
- **EXE:** 8.443.904 Bytes, 10 PNGs, 40 BMPs
- **ICO:** 75.655 Bytes, 10 Größen: 16, 20, 24, 32, 40, 48, 64, 96, 128, 256
- **Ergebnis:** Desktop-Icon + Taskleisten-Icon nach Neustart **SCHARF** ✅

## WICHTIGE SKRIPTE (scripts/ Ordner)
- `test_replaytime_metrics.py` – prüft ob Zeitsprung funktioniert (WebSocket + REST gleichzeitig)
- `search_bcuk_urls.py` / `search_bcuk_commands.py` – analysiert BCUK DLLs
- `test_websocket_commands.py` – maskierte WebSocket-Befehle senden
- `dump_shared_memory.py`, `read_lmu_sm.py` – Shared Memory Analyse
- `generate-icons.py` – generiert alle Icons + Logo aus den Quellbildern
- `generate_all_icons.py` – generiert Icons + BMP-basierte ICO (für Tests)
- `check_icons_final.py` – prüft Icon-Größen + erstellt ICO mit 10 Windows-Größen
- `create_bmp_ico.py` – erstellt BMP-basierte ICO (372KB, Windows-nativ)
- `patch_icon.py` – versucht Icon direkt in EXE zu patchen (rcedit, nicht erfolgreich)
- `verify_icons.py` / `verify_icons2.py` – verifiziert Icon-Größen in ICO/EXE

## WICHTIGE RUST-DATEIEN
- `src-tauri/src/main.rs` – `jump_to_incident_replay` Command (Doppel-Zeitsprung-Logik)
- `src-tauri/src/lmu_ws.rs` – WebSocket-Client (aktiviert Replay-API)
- `src-tauri/src/keyboard.rs` – Tastatur-Simulation (SendInput/Scancodes)
- `src-tauri/src/settings.rs` – Settings (pre_roll_seconds=20, post_roll_seconds=20)
- `src-tauri/src/lmu_client.rs` – REST-Client (fetch, put, seek_replay_to)

## WICHTIGE FRONTEND-DATEIEN
- `frontend/src/App.tsx` – Haupt-App, Routing, View-Steuerung
- `frontend/src/components/Sidebar.tsx` – Sidebar (Logo, Language Toggle, Server, Navigation, FCY, Footer)
- `frontend/src/views/HomeView.tsx` – Disconnected Startseite (Logo, Willkommen, Lizenz-Eingabe)
- `frontend/src/views/FahrerfeldView.tsx` – Fahrerfeld-Ansicht
- `frontend/src/views/VorfaelleView.tsx` – Vorfälle-Ansicht
- `frontend/src/views/ArchivView.tsx` – Archiv-Ansicht
- `frontend/src/views/EinstellungenView.tsx` – Einstellungen (Discord Webhook, FCY, Vorlauf/Nachlauf, Danger Zone)
- `frontend/src/splashscreen.tsx` – Splashscreen (Logo, Version, Update-Check)
- `frontend/src/styles.css` – Alle Styles (~2200 Zeilen)
- `frontend/src/i18n/translations.ts` – Übersetzungen (DE/EN)

## LETZTE ÄNDERUNGEN (Chat v0.8.5)

### 1. Doppelte Lizenznummer entfernt (`frontend/src/views/HomeView.tsx`)
- Die Zeile `<div className="home-license-hint">123a-234b-345YZ</div>` unter dem Eingabefeld wurde gelöscht
- Der Placeholder `123a-234b-345YZ` im Input-Feld bleibt erhalten

### 2. Placeholder_SB in der Sidebar (`frontend/src/components/Sidebar.tsx` + `styles.css`)
- Ein Container `sidebar-placeholder` mit `min-height: 393px` und `flex: 1` wurde zwischen Server Section und Software Infos Section eingefügt (nur im disconnected Zustand, wenn `!licensed`)
- **CSS:** `.sidebar-placeholder { flex: 1; min-height: 393px; display: flex; ... }`

### 3. Einstellungen-Layout (`frontend/src/views/EinstellungenView.tsx` + `styles.css`)
- Discord Webhook auf volle Breite
- FCY-/Vorlauf-Nachlauf nebeneinander
- Neue CSS-Klassen: `settings-row`, `settings-block-half`, `settings-block-full`

### 4. ConfirmModal-Titel (`frontend/src/views/EinstellungenView.tsx` + `styles.css`)
- Titel verkleinert auf h2, 20px, `white-space: nowrap`

### 5. Sidebar-Abstand (`styles.css`)
- `sidebar-section-infos` und `sidebar-section-server` haben `margin-top: 38px` für 48px Abstand zum Footer

### 6. onFocusDriver (`frontend/src/views/VorfaelleView.tsx` + `App.tsx`)
- `onFocusDriver` Prop an VorfaelleView übergeben (für Fokus-Funktion)

### 7. Logo/Icons aktualisiert
- Logo: `LMU RC Logo - hell.png` (1024x486, breites Banner)
- Icons: `LMU RC - Icon transparent.png` (1024x1024, Flaggen-Symbol)
- Icons generiert via `scripts/generate-icons.py`

### 8. Icon-Problem gelöst (Desktop + Taskleiste unscharf → SCHARF)
- ICO mit 10 Windows-Skalierungsgrößen erstellt (16, 20, 24, 32, 40, 48, 64, 96, 128, 256)
- `icon.ico` an erste Stelle in `tauri.conf.json` Icon-Liste gesetzt
- `installerIcon` auf `icon.ico` gesetzt
- `build.rs` mit `rerun-if-changed` für Icons
- **Lösung war: PC neu starten nach Installation** (Icon-Cache leeren!)

## NÄCHSTE SCHRITTE
1. **v0.8.5 ausführlich testen** (Placeholder_SB, Disconnected-Layout, Einstellungen, Icon-Qualität)
2. Weitere UI-Anpassungen nach Figma-Design besprechen
3. Build als nächste Version (z.B. v0.8.6)

## BUILD-Prozess
```bash
cd "C:\Users\Administrator\Documents\AI\Software Entwicklung\LMU Racecontrol\Tool\lmu-race-control"
cargo tauri build
```
- Installer: `src-tauri\target\release\bundle\nsis\LMU RACECONTROL_0.8.5_x64-setup.exe`
- MSI: `src-tauri\target\release\bundle\msi\LMU RACECONTROL_0.8.5_x64_en-US.msi`
- "A public key found, but no private key" Fehler = nur für Auto-Update, Installer funktionieren trotzdem
- Vor Build: Version in `src-tauri/Cargo.toml`, `package.json`, `src-tauri/tauri.conf.json` anpassen