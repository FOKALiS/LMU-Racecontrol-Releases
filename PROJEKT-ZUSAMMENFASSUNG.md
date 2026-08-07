# LMU RACECONTROL – Projekt-Zusammenfassung für neuen Chat

## Projekt
**LMU RACECONTROL** – Tauri-basiertes Desktop-Tool für Le Mans Ultimate Rennkommissionen.
- **Pfad:** `C:\Users\Administrator\Documents\AI\Software Entwicklung\LMU Racecontrol\Tool\lmu-race-control`
- **Aktuelle Version:** v0.9.4
- **Build-Workflow:** https://github.com/FOKALiS/LMU-Racecontrol/actions
- **Release-Versionierung:** Zweite Kommastelle erhöhen (z. B. 0.9.0 → 0.9.1)
- **User ist Grafik Designer, kein Code** – Schritt-für-Schritt-Anleitung nötig
- **Letzter Commit:** `2929eda` (v0.9.4 – API-Key Lookup + Server-Bereinigung)
- **GitHub:** `origin/main` auf dem aktuellsten Stand, Arbeitsverzeichnis sauber

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

## FUNKTIONIERT ✅ (Stand v0.9.1)

### Kamera-Steuerung
- **3 Kamera-Buttons:** "Bord", "TV", "Heck"
- **Tastatur-Simulation (SendInput/Scancodes)** statt REST-API (REST gab 200, tat aber nichts)
- **Zoom + / Zoom -** als Dauer-Zoom (über keyboard.json konfigurierbar)
- Keyboard-Layout wird aus **`keyboard.json`** gelesen (LMU Tastenbelegung)
- **Onboard Cameras** wird ebenfalls aus keyboard.json ausgelesen und angezeigt
- **Reload-Button** in Einstellungen lädt Tastenbelegung neu (OnceLock → RwLock gefixt)
- **Datei-Browser (📂)** zur Auswahl des LMU-Installationspfads

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
- Auto-Stop nach Nachlaufzeit (F11-Taste) via Timer

### Vorfall-Erkennung (v0.8.8, verbessert in v0.9.1)
- **ROT (Crash):** Impact >3.0g (Shared Memory), Rundenzeit >30% zum eigenen Schnitt, Stillstand >0.5 km/h und <10 km/h
- **GELB (Auffälligkeit):** Rundenzeit >15%, Positionsverlust ≥3 ohne Boxenstopp, FCY-Verstoß
- **WEISS (dauerhaft langsam):** >30 Sekunden unter 50 km/h (Timer-basiert)
- **Impact-Schwelle gesenkt:** 5.0 → 3.0g (sensibler)
- Cooldown: 30 Sekunden pro Fahrzeug
- **MIN_STOPPED_SPEED_KMH = 0.5** – Fahrzeuge mit exakt 0.0 km/h (z.B. Pipo Derani) werden ignoriert (v0.9.0)
- **Weiße Flagge** prüft ebenfalls `> MIN_STOPPED_SPEED_KMH` (v0.9.1)

### Session-Buttons (v0.8.8)
- Session-Buttons in FahrerfeldView + VorfaelleView sind **reine Info-Anzeigen** (nicht klickbar)
- `session?.session_type` aus LMU bestimmt den active State (Practice / Qualifying / Race)
- Gleiche Höhe (36px) + Breite (flex:1) wie Filter-Buttons

### FCY (Full Course Yellow)
- **Countdown** (10 Sekunden, konfigurierbar) → dann **Active**
- Während Active: Prüft alle Fahrzeuge auf **> 60 km/h + 3 km/h Toleranz = 63 km/h**
- Jeder Verstoß wird **einmal pro Fahrzeug und FCY-Phase** gemeldet
- Verstöße werden als GELB-Vorfälle mit Typ "FCY-Verstoß" gespeichert

### Splashscreen (v0.8.7)
- Copyright: "FOKALiS - Film & Medienagentur"

### Weitere funktionierende Features
- Fahrer-Fokus per Klick auf Fahrerzeile (REST-API PUT `/rest/watch/focus/{slotID}`)
- Speed-Anzeige
- Connect/Disconnect
- Lizenzsystem (Online-Aktivierung)
- Standings-Updates per Polling (1s Intervall)
- Connect-Button deaktiviert wenn nicht lizenziert (v0.8.7)
- Switch to Live via 86400s-Trick
- Player-Bar (Play, Vor, Zurück, Slow, Rewind) via SendInput
- Discord-Webhook (formatiert mit Fahrer, Verwarn-/Strafpunkte, "N.A." bei leerer Kurve)
- Einstellungen-Überschrift linksbündig
- Ordnersymbol für LMU-Pfad-Auswahl in Einstellungen
- Archiv: Entscheidungs-Badge öffnet InvestigationModal
- Fenster-Titel: "LMU RACECONTROL – Das Tool für Rennkommissare"

## FUNKTIONIERT NICHT ❌ / OFFEN

1. **Shared Memory Impact-Daten:** `read_impact_data()` ist codiert, aber LMU-Offsets sind geschätzt (aus SM Bridge Logs). Python-Tests schlagen fehl (Error 5 – Admin/User Session Problem). Die Tauri-App hat als normaler User potenziell Zugriff.
2. **Manufacturers-API:** LMU liefert Hersteller-Daten pro Fahrzeug – müssen noch in FahrerfeldView eingebunden werden.
3. **VE (Verbale Entscheidung):** Noch nicht implementiert.
4. **Tabellen-Zellen-Ausrichtung:** Soll noch angepasst werden.
5. **Mehrbenutzer-fähig:** Aktuell nur lokale SQLite-DB – kein gemeinsamer Zugriff für mehrere Stewards.

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
- Die Tastenbelegung wird aus `{Installation}/UserData/player/keyboard.json` gelesen
- Format: `{"Input": {"Aktion": DIK_Scancode, ...}, "Type": "Keyboard"}`
- DIK-Scancodes werden in Windows-Scancodes (+ extended-Flag) übersetzt
- **Relevante Actions:** "Tracking Cameras", "Driving Cameras", "Onboard Cameras", "Swingman Camera", "Swingman Zoom In/Out", "Instant Replay", "Replay Play/Stop/Slowmotion/Fast Forward/Fast Rewind/Reverse"
- Standard-Presets dienen als Fallback, wenn keyboard.json nicht lesbar ist

### Tastatur-Simulation (keyboard.rs)
- Verwendet `SendInput` mit Scancodes (KEIN externer Helper nötig)
- `OnceLock` → **RwLock** umgestellt (v0.8.8), damit Reload mehrfach funktioniert
- Kamera-Wechsel holt LMU kurz in den Vordergrund, sendet 1x Scancode und geht zurück
- Dauer-Zoom sendet Taste alle 15ms via Hintergrund-Thread
- Hold-Tasten (F7, F8, F9, F10) drücken KEYDOWN und halten bis Stop

## ICON-PROBLEM GELÖST ✅ (v0.8.5)

### Ursache (2 Probleme kombiniert)
1. **Fehlende Windows-Skalierungsgrößen** – Die ICO hatte nur 7 Größen (16, 24, 32, 48, 64, 128, 256). Windows braucht aber auch **20, 40 und 96** für DPI-Skalierungen (100%, 125%, 150%, 175%, 200%).
2. **Windows Icon-Cache** – Selbst mit der richtigen ICO zeigte Windows das alte, verpixelte Icon aus dem Cache. Erst ein **Neustart** hat den Cache geleert.

### Lösung
- **`icon.ico`** – Neu erstellt mit **10 Größen**: 16, 20, 24, 32, 40, 48, 64, 96, 128, 256
- **Alle PNGs** (`32x32.png`, `128x128.png`, `128x128@2x.png`, `icon.png`) – Mit dem LMU RC Logo generiert
- **`tauri.conf.json`** – `icon.ico` an erster Stelle der Icon-Liste, `installerIcon` gesetzt
- **`build.rs`** – `rerun-if-changed` für Icons hinzugefügt

## WICHTIGE SKRIPTE (scripts/ Ordner)
- `test_replaytime_metrics.py` – prüft ob Zeitsprung funktioniert (WebSocket + REST gleichzeitig)
- `search_bcuk_urls.py` / `search_bcuk_commands.py` – analysiert BCUK DLLs
- `test_websocket_commands.py` – maskierte WebSocket-Befehle senden
- `dump_shared_memory.py`, `read_lmu_sm.py` – Shared Memory Analyse
- `generate-icons.py` – generiert alle Icons + Logo aus den Quellbildern
- `check_icons_final.py` – prüft Icon-Größen + erstellt ICO mit 10 Windows-Größen

## WICHTIGE RUST-DATEIEN
- `src-tauri/src/main.rs` – `jump_to_incident_replay` Command (Doppel-Zeitsprung-Logik), Commands
- `src-tauri/src/lmu_ws.rs` – WebSocket-Client (aktiviert Replay-API)
- `src-tauri/src/keyboard.rs` – Tastatur-Simulation (SendInput/Scancodes, RwLock-basierte Config)
- `src-tauri/src/keyboard_config.rs` – Liest keyboard.json, übersetzt DIK→Windows-Scancodes
- `src-tauri/src/settings.rs` – Settings (pre_roll_seconds=20, post_roll_seconds=20, lmu_install_path)
- `src-tauri/src/lmu_client.rs` – REST-Client (fetch, put, seek_replay_to)
- `src-tauri/src/incidents.rs` – Vorfall-Erkennung (ROT/GELB/WEISS), Heuristik
- `src-tauri/src/shared_memory.rs` – LMU Shared Memory Zugriff (LMU_Data, Impact-Offsets)
- `src-tauri/src/fcy.rs` – FCY-Statusmaschine (Idle → Countdown → Active)
- `src-tauri/src/db.rs` – SQLite-Datenbank (Incidents, Settings, License)
- `src-tauri/src/discord.rs` – Discord-Webhook-Versand
- `src-tauri/src/settings.rs` – Settings-Speicher (JSON-Datei)
- `src-tauri/src/license.rs` – Lizenzsystem (Online-Aktivierung)

## WICHTIGE FRONTEND-DATEIEN
- `frontend/src/App.tsx` – Haupt-App, Routing, View-Steuerung, session-Prop für Views
- `frontend/src/components/Sidebar.tsx` – Sidebar (Logo, Language Toggle, Server, Navigation, FCY, Footer, Placeholder_SB)
- `frontend/src/views/HomeView.tsx` – Disconnected Startseite (Logo, Willkommen)
- `frontend/src/views/FahrerfeldView.tsx` – Fahrerfeld-Ansicht (Session-Info, Player, Filter, Tabelle)
- `frontend/src/views/VorfaelleView.tsx` – Vorfälle-Ansicht (Session-Info, Player, Filter, Vor/Nachlauf, Tabelle)
- `frontend/src/views/ArchivView.tsx` – Archiv-Ansicht
- `frontend/src/views/EinstellungenView.tsx` – Einstellungen (Discord Webhook, FCY, Vorlauf/Nachlauf, LMU-Pfad mit Datei-Browser, Tastenbelegung, Danger Zone)
- `frontend/src/splashscreen.tsx` – Splashscreen (Logo, Version, Update-Check)
- `frontend/src/styles.css` – Alle Styles (~2200 Zeilen)
- `frontend/src/i18n/translations.ts` – Übersetzungen (DE/EN)

## CHANGELOG – Änderungen pro Version

### v0.9.4 (07.08.2026)
- **Server-Endpunkt `POST /api/lookup-api-key`:** Validiert License-Key live bei Keygen, legt automatisch Tenant + API-Key mit korrektem Tier an (Enterprise L/XL/HOWE)
- **Button "API-Key abfragen":** In Einstellungen → Server Verbindung – fragt API-Key zum License-Key ab (nur bei lizenzierter Enterprise)
- **Version in Lizenz-Überschrift:** "Lizenz Informationen – Version: Enterprise XL (Server)"
- **Server-URL-Feld entfernt:** Muss vom User nicht änderbar sein
- **Server `DELETE /api/incidents`:** Löscht nur Incidents des eigenen Tenants (team-sicher)
- **Server `purge_old_incidents`:** Auto-Bereinigung alter Vorfälle (>26h) alle 30 Minuten
- **`clear_all_incidents`:** Löscht jetzt auch Server-Vorfälle des eigenen Tenants (kein fremdes Team)
- **Server-EXE + App-EXE gebaut:** v0.9.4

### v0.9.1 (06.08.2026)
- **Weiße Flagge Fix:** Auch die weiße Flagge (langsames Fahrzeug) prüft jetzt `> MIN_STOPPED_SPEED_KMH (0.5 km/h)`, damit Fahrzeuge mit exakt 0.0 km/h (Pipo Derani) nicht fälschlich als "langsam" gemeldet werden
- **Commit:** `52d1e2b` – gepusht auf `origin/main`

### v0.9.0 (06.08.2026)
- **Pipo Derani Fix:** `MIN_STOPPED_SPEED_KMH = 0.5` eingeführt – Fahrzeuge mit 0.0 km/h werden nicht mehr als Stillstand erkannt
- **Discord-Webhook:** Formatierung überarbeitet (Fahrer, Verwarn-/Strafpunkte, "N.A." bei leerer Kurve)
- **Einstellungen:** Überschrift linksbündig (text-align: center → left)
- **Ordnersymbol (📂)** für LMU-Pfad-Auswahl wieder eingebaut
- **Archiv:** Entscheidungs-Badge öffnet jetzt InvestigationModal
- **Fenster-Titel:** "LMU RACECONTROL – Das Tool für Rennkommissare"
- **Builds:** NSIS + MSI auf GitHub Releases hochgeladen

### v0.8.8 (05.08.2026)
- **Vorfall-Erkennung überarbeitet:**
  - ROT: Impact >3.0g, Rundenzeit >30%, Stillstand <10 km/h
  - GELB: Rundenzeit >15%, Positionsverlust ≥3, FCY-Verstoß
  - WEISS (NEU): >30s unter 50 km/h (Timer-basiert)
- **Session-Buttons:** Nicht mehr klickbar, active State aus LMU (`session?.session_type`)
- **Keyboard-Config gefixt:**
  - Vollständige Scancode-Tabelle (fehlende Tasten ergänzt)
  - "Onboard Cameras" unterstützt
  - OnceLock → RwLock (Reload funktioniert jetzt mehrfach)
- **Datei-Browser (📂)** für LMU-Installationspfad in Einstellungen
- `@tauri-apps/plugin-dialog` installiert

### v0.8.7 (05.08.2026)
- Timer: replay_pause() via F11 nach Vorlauf+Nachlauf
- LIVE-Button: onSwitchToLive?.() in allen 3 Views
- Splashscreen: Copyright "FOKALiS - Film & Medienagentur"
- Connect-Button deaktiviert wenn nicht lizenziert
- Session-Buttons/Player-Bar/Filter-Tabs als eigene CSS-Klassen

### v0.8.6–0.8.7 (Zwischen-Versionen)
- Wartezeiten in jump_to_incident_replay optimiert
- Incident-Erkennung gefixt (Impact, Stillstand, Feld-basierte Rundenzeit)
- switch_to_live auf 86400s-Trick zurückgesetzt
- Fenstertitel "Tool für Rennkommissare"

## NÄCHSTE SCHRITTE (Offene Punkte)

1. **Manufacturers einbinden** – Hersteller-Daten aus LMU-API in FahrerfeldView anzeigen
2. **VE (Verbale Entscheidung) einbinden** – Neue Funktion/Feature für mündliche Urteile
3. **Tabellen-Zellen-Ausrichtung** – Zellen-Inhalte in den Tabellen korrekt ausrichten
4. **Discord Webhook testen** – Webhook-Funktionalität prüfen und ggf. korrigieren
5. **Shared Memory Impact-Daten testen** – Wenn LMU läuft, ob `impact_mag` Daten liefert
6. **Replay-Zeit ausführlich testen** – Doppel-Zeitsprung in der Praxis prüfen

## 🚀 GROSSES ZIEL: Server-Architektur für Enterprise (geplant ab 07.08.2026)

### Problem
Mehrere Rennkommissare sitzen verteilt (Dresden, Bayreuth, Ulm, ...) – jeder hat aktuell seine eigene lokale SQLite-DB. Keiner sieht die Vorfälle der anderen.

### Infrastruktur
- **V-Server von ZAP:** Intel Xeon E5-2650 v2 @ 2.60GHz, 32 GB RAM, Windows Server 2019
- **Build 17763.8146** – läuft 24/7 im Rechenzentrum

### Lösungs-Architektur (Monorepo)

```
lmu-race-control/          ← EIN Repository (das bestehende)
├── src-tauri/             ← Das Tool (Client) – bleibt wie es ist
│   ├── src/
│   │   ├── main.rs        ← Client-Logik
│   │   ├── db.rs          ← Wird erweitert (LocalDb + RemoteDb)
│   │   └── ...
├── frontend/              ← Die UI – bleibt wie sie ist
├── server/                ← NEU: Die Server-Komponente (Enterprise)
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs        ← REST-API (Axum)
│       ├── db.rs          ← Zentrale DB (SQLite, gleiches Schema wie Client)
│       └── ...
```

### Phasen

#### Phase 1: Server-Backend (server/ Ordner)
- **Technologie:** Rust mit Axum + SQLite (sqlx)
- **API-Endpunkte:**
  - `POST /api/incidents` – Vorfall melden
  - `GET /api/incidents` – alle Vorfälle abrufen
  - `PATCH /api/incidents/:id` – Entscheidung/Notiz setzen
  - `POST /api/incidents/:id/decision` – Entscheidung setzen
  - `GET /api/standings` – aktuelle Wertung (optional)
- **Mandanten-Verwaltung:** Jeder Kunde = ein Tenant mit API-Key
- **Authentifizierung:** `Authorization: Bearer <api_key>`
- **Windows-Dienst:** Läuft als .exe auf dem V-Server

#### Phase 2: Tool-Umbau (Client)
- **`ConnectionMode`-Enum:** `Local` oder `Server { url, api_key }`
- **`Db`-Trait:** `LocalDb` (bestehendes SQLite) + `RemoteDb` (ruft REST-API auf)
- **Settings-Erweiterung:** Neuer Tab "Server" mit URL + API-Key-Feld
- **Feature-Flags:** Basic-Kunden nutzen nur LocalDb, Enterprise-Kunden schalten Server frei

#### Phase 3: Vertriebsmodell
- **LMU RACECONTROL Basic** (lokal, kostenlos/einmalig) – für 1-2 Stewards an einem Rechner
- **LMU RACECONTROL Enterprise** (Server, Monatsabo) – für mehrere Stewards remote

### Feature-Grenzen
| Feature | Basic | Enterprise |
|---------|-------|------------|
| Vorfall-Erkennung | ✅ | ✅ |
| Discord-Webhook | ✅ | ✅ |
| FCY-Überwachung | ✅ | ✅ |
| Kamera-Steuerung | ✅ | ✅ |
| Replay-Steuerung | ✅ | ✅ |
| Server-Anbindung | ❌ | ✅ |
| Team-Übersicht | ❌ | ✅ |
| Mandanten-Verwaltung | ❌ | ✅ (nur Admin) |

## BUILD-Prozess
```bash
cd "C:\Users\Administrator\Documents\AI\Software Entwicklung\LMU Racecontrol\Tool\lmu-race-control"
cargo tauri build
```
- Installer: `src-tauri\target\release\bundle\nsis\LMU RACECONTROL_0.9.0_x64-setup.exe`
- MSI: `src-tauri\target\release\bundle\msi\LMU RACECONTROL_0.9.0_x64_en-US.msi`
- "A public key found, but no private key" Fehler = nur für Auto-Update, Installer funktionieren trotzdem
- Vor Build: Version in `src-tauri/Cargo.toml`, `package.json`, `src-tauri/tauri.conf.json` anpassen
- Server-Build: `cargo build --release -p lmu-server` (separate .exe)