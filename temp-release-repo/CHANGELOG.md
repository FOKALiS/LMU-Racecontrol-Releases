# Changelog

Alle nennenswerten Änderungen an LMU Race Control werden hier dokumentiert.
Format angelehnt an [Keep a Changelog](https://keepachangelog.com/de/1.0.0/),
Versionierung nach [Semantic Versioning](https://semver.org/lang/de/).

## [0.9.5] - 10.08.2026 (Neue Hilfe, Signierter Auto-Update)
### Geändert
- **Hilfe komplett überarbeitet**: Ausführliche Erklärungen zu Lizenz & Aktivierung,
  Server-Verbindung & API-Key abfragen, Discord-Webhook einrichten, Einstellungen im Detail,
  Update & Version, Datenbank leeren – auf Deutsch und Englisch
- **Tauri-Signing-Key erneuert**: Alter Key durch neuen ersetzt, Auto-Update funktioniert wieder
- **Update-URL auf öffentliches Release-Repo umgestellt**: LMU-Racecontrol-Releases

### Technisch
- **Version**: 0.9.4 → 0.9.5

## [0.8.3] - 01.08.2026 (Figma MCP Integration + Player-Bar Icons)
### Hinzugefügt
- **Figma MCP Server** (`figma-developer-mcp` v0.13.2) installiert und konfiguriert
- **Figma Design "LMU Racecontrol"** importiert: 6 Screens (Home, Fahrerfeld, Vorfälle, Archiv, Einstellungen, Investigation Overlay), 14 Komponenten, Design-Tokens
- **Design-Daten** in `figma-screens/design-data-complete.json` gespeichert
- **Logos** aus Figma-Export in `figma-screens/` und `frontend/public/logo.png`

### Geändert
- **Player-Bar Icons**: Emoji-Platzhalter (⏮⏪▶⏩⏭) durch echte PNG-Icons ersetzt
- **Icons** aus `C:\Users\Administrator\Documents\AI\Software Entwicklung\LMU Racecontrol\Icons` in `frontend/public/icons/` integriert
- **CSS-Variablen** um Figma Design-Tokens erweitert (`--text-secondary`, `--text-dim-soft`, `--purple`)
- **CSS für Player-Icons** hinzugefügt (20x20 Größe, Hover-Effekte, disabled-State)
- **Sidebar-Duplikat** in Sidebar.tsx behoben
- **Logo-Höhe** auf `auto` gesetzt für korrekte Proportionen

### Technisch
- **Version**: 0.8.2 → 0.8.3

## [0.7.0] - 27.07.2026 (Tastatur-Simulation statt Camera-Helper + Turbo-Zoom)
### Geändert
- **Camera-Helper entfernt**: Der separate `camera-helper`-Prozess wurde komplett entfernt. Die Kamera-Steuerung läuft jetzt direkt über `SendInput` mit Scancodes aus der Tastenbelegung des Users – kein externer Prozess mehr nötig.
- **Kamera-Button "Bord" statt "Helmet"**: Passt zur LMU-Standard-Tastenbelegung (Insert = Bordkamera). "Helmet" wird trotzdem als Alias erkannt.
- **Zoom-Funktion**: Neue Zoom-Buttons (+ / -) neben der Kamera-Steuerung. Gedrückt halten = Dauer-Zoom via Hintergrund-Thread in Rust (kein setInterval). Funktioniert auf allen Seiten (Fahrerfeld, Vorfälle, Archiv).
- **Zoom-Geschwindigkeit**: ~500 Tastendrücke pro Sekunde (1ms KeyDown, 1ms Pause).

### Behoben
- **Zoom funktionierte nicht auf Vorfälle/Archiv-Seiten**: `onZoomStart`/`onZoomEnd` Props wurden nicht an `TopToolbar` durchgereicht. Jetzt an `VorfaelleView` und `ArchivView` übergeben.

### Technisch
- `keyboard.rs`: Komplett überarbeitet – `SendInput` mit Scancodes, kein `enigo` mehr. `zoom_start`/`zoom_stop` mit Hintergrund-Thread und AtomicBool-Flag.
- `src-tauri/camera-helper/` entfernt (über 100MB Build-Artefakte eingespart).
- `tauri.conf.json`: `resources` von `camera-helper.exe` auf leer gesetzt.

## [0.6.12] - 26.07.2026 (Cam Control + Replay-Steuerung Fix)
### Behoben
- **jumpToReplay setzt jetzt auch Kamera**: Nach dem Replay-Zeitsprung wird automatisch die TV-Kamera gesetzt (via REST-API). Vorher sprang der Replay an die richtige Zeit, aber der Nutzer sah nur die vorherige Kameraeinstellung.
- **Replay-Modus wird vor Zeitsprung aktiviert**: LMU braucht zwingend den Replay-Modus, damit Kamera-Befehle wirken. `switch_to_replay` wird jetzt vor dem Zeitsprung aufgerufen.
- **Längere Pausen zwischen Kommandos**: 200ms nach Modus-Wechsel, 500ms nach Zeitsprung – damit LMU genug Zeit hat, die Befehle zu verarbeiten.
- **focus_driver setzt jetzt auch TV-Kamera**: Nach dem Fahrer-Fokus wird die TV-Kamera aktiviert, damit der Nutzer sofort das Fahrzeug sieht.
- **Verbessertes Debug-Logging**: Alle Schritte werden jetzt mit Emoji und Zeitstempel geloggt, damit Fehler leichter nachvollziehbar sind.

## [0.6.11] - 25.07.2026 (REST-API PUT-Body Fix + Kamera-Key-Fix)
### Behoben
- **Alle PUT-Requests schlugen fehl (HTTP 400)**: Die LMU REST-API verlangt bei PUT zwingend einen leeren JSON-Body `{}` mit `Content-Type: application/json`. Ohne Body kam HTTP 400 – betroffen waren: focus, camera, replaytime, switch_to_live/replay
- **Kamera-Steuerung funktioniert jetzt zuverlässig via REST-API**: `/rest/watch/focus/TV`, `/rest/watch/focus/Onboard`, `/rest/watch/focus/Heli` u.a. sind per curl bestätigt ✅

## [0.6.7] - 24.07.2026 (Shared Memory, Connect/Disconnect, FCY +3 km/h Toleranz)
### Neu
- **Shared Memory (rFactor 2/LMU)**: Direkter Zugriff auf den LMU Shared Memory (`Local\rFactor2SharedMemory`). Kamera-Wechsel und Fahrzeug-Fokus funktionieren jetzt **ohne Fenster-Fokus, ohne Tastatur-Simulation, ohne Terminal-Flash** - wie bei Broadcast Control UK, SimHub und anderen professionellen Tools
- **Connect/Disconnect Hover**: Wenn verbunden, wird beim Überfahren mit der Maus "Disconnect from Server" (rot) angezeigt - Klick trennt die Verbindung

### Behoben
- **FCY-Überwachung aktiviert**: Bei Überschreitung von Limit + 3 km/h Toleranz (z.B. 60+3=63 km/h) wird automatisch ein FCY-Verstoß-Vorfall erstellt

## [0.6.6] - 24.07.2026 (Tastatur-Steuerung: Scancodes, AttachThreadInput für LMU-Fokus auf anderem Monitor)
### Behoben
- **Tastendrücke landen jetzt zuverlässig in LMU (nicht in der Tauri-App)**: Scancodes via `KEYEVENTF_SCANCODE` statt virtueller Tastencodes – Spiele verwenden Scancodes für ihre Tastenbelegung
- **LMU-Fokus auch über mehrere Monitore hinweg**: `AttachThreadInput` umgeht Windows-UIPI, sodass der Fokus zuverlässig auf LMU gesetzt werden kann

## [0.6.5] - 24.07.2026 (Tastatur-Steuerung neu: Win32 SendInput, kein PowerShell-Flash, Fahrerfeld-Sortierung)
### Behoben
- **Terminal-Fenster-Flash beim Fokussieren von LMU beseitigt**: PowerShell `AppActivate` durch native Win32 `FindWindowW`/`SetForegroundWindow` ersetzt – kein aufblitzendes Terminal mehr
- **Tastaturbefehle zuverlässiger**: `enigo`-Crate entfernt, stattdessen direkte Win32 `SendInput`-API mit Hintergrund-Thread-Architektur

### Geändert
- **Fahrerfeld wird jetzt nach Position sortiert** (1., 2., 3., ...) via `useMemo`

## [0.6.4] - 24.07.2026 (Icon-Größe: Logo auf Desktop/Taskleiste vergrößert)
### Behoben
- **Icon auf Desktop und Taskleiste war zu klein**: Das Logo hatte im Quellbild (`icon-source.png`) zu viel transparenten Rand. Der transparente Rand wurde entfernt und das Logo füllt jetzt fast die gesamte Icon-Fläche.

## [0.6.3] - 23.07.2026 (Icon-Fix: korrekte Windows-Icon-Generierung)
### Behoben
- **Windows-Icon (.ico) wurde nicht korrekt angezeigt**: Die `icon.ico` war fehlerhaft und zu klein. Mit dem Tauri Icon Generator neu generiert (34.690 Bytes).

## [0.6.1] - 22.07.2026 (Installer-Fix, Schriftarten lokal, Sidebar-Steuerung)
### Behoben
- **ERR_CONNECTION_REFUSED beim Start**: Dem Hauptfenster fehlte `"url": "index.html"`, sodass die installierte App versuchte, vom Dev-Server (localhost:1420) zu laden.
- **Schriftarten (Michroma/Inter) werden jetzt lokal eingebettet**: Die Schriftart-Dateien liegen als `.woff2` im `frontend/public/fonts/`-Verzeichnis und werden beim Bauen fest in die App integriert – kein Google-Fonts-Netzwerkzugriff mehr nötig. Die App funktioniert jetzt vollständig offline.

### Geändert
- **Sidebar-Steuerung** an die drei Zustände angepasst: Ohne Lizenz: nur "Software Infos"; Lizenziert, nicht verbunden: "Connect to Server" + "Software Infos"; Lizenziert + verbunden: alle Buttons

## [0.6.0] - 22.07.2026 (Kamera-Steuerung, Fahrzeug-Fokus, Splashscreen-Design)
### Hinzugefügt
- **Kamera-Steuerung per Tastatursimulation**: Die Kamera-Buttons (TV, Helmet, Front, Heck, Top, Behind) simulieren jetzt die Tastendrücke F1-F6 direkt in LMU/rFactor2
- **Fahrzeug-Fokus per Tastatursimulation**: Klick auf einen Vorfall oder Doppelklick auf einen Fahrer springt zur richtigen Replay-Position
- **Automatischer Replay-Sprung**: Der Replay-Sprung zur Vorfall-Position funktioniert jetzt zuverlässig über die LMU-REST-API

### Geändert
- **Splashscreen-Design überarbeitet**: Logo vergrößert (300px → 380px), Versionsnummer unter dem Logo platziert, gesamtes Layout optisch aufgewertet

## [0.5.4] - Unveröffentlicht (Cam Control rechtsbündig)
### Geändert
- Image Control + Cam Control werden jetzt als EINE Einheit rechtsbündig ausgerichtet

## [0.5.3] - Unveröffentlicht (Race-Control-Buttons über volle Breite)
### Geändert
- "Neuer Vorfall"/"Erledigte Vorfälle"/"Full Course Yellow" spannen sich jetzt über die volle Breite

## [0.5.2] - Unveröffentlicht (Fix: Aktivierung mit "Require Fingerprint Scope")
### Behoben
- Eure Keygen-Policy hat "Require Fingerprint Scope" aktiviert – die Geräte-Kennung wird jetzt von Anfang an immer mitgeschickt.

## [0.5.1] - Unveröffentlicht (Keygen-Konto-ID eingetragen)
### Geändert
- `KEYGEN_ACCOUNT` in `src-tauri/src/license.rs` von Platzhalter auf die echte Keygen-Konto-ID umgestellt

## [0.5.0] - Unveröffentlicht (Lizenzsystem)
### Hinzugefügt
- Lizenzpflicht: ohne gültige Lizenz sind nur Startbildschirm (mit Lizenzschlüssel-Eingabe), "Hilfe" und der Website-Link nutzbar
- Anbindung an die Keygen-License-API (https://keygen.sh): Aktivierung pro Gerät, regelmäßige Online-Nachprüfung, 14 Tage Offline-Kulanzfrist

## [0.4.2] - Unveröffentlicht (FCY-Hervorhebung, Button-Ausrichtung)
### Geändert
- Bei aktivem/ausgelöstem Full Course Yellow: gelber Rahmen um den Hauptbereich, "FULL COURSE YELLOW AKTIV"-Banner korrekt zentriert
- Buttons "Neuer Vorfall"/"Erledigte Vorfälle"/"Full Course Yellow" linksbündig mit der "Cam Control"-Zeile

## [0.4.1] - Unveröffentlicht (Grafik-Feinschliff)
### Geändert
- Sprachumschalter (DE/EN) aus dem Logo-Bereich in den Bereich "Software Infos" verschoben
- Website-Zeile in Sidebar UND Splashscreen ist jetzt klickbar
- Sekunden-Eingabefelder bei "Vorlaufzeit"/"Nachlaufzeit" verbreitert
- Tabellen-Kopfzeilen UND Datenzeilen haben jetzt durchgängig abgerundete Außenkanten
- Splashscreen-Anzeigedauer von 10 auf 5 Sekunden verkürzt

## [0.4.0] - Unveröffentlicht (Splashscreen, Auto-Update, dynamische Versionsanzeige)
### Hinzugefügt
- Splashscreen-Fenster beim Programmstart (5 Sekunden, Logo, Version, Website-Zeile)
- Eingebauter Auto-Updater: Splashscreen prüft im Hintergrund auf neue Version
- Versionsanzeige in der Sidebar ist jetzt dynamisch

## [0.3.0] - Unveröffentlicht (Feinschliff: Name, Icon, Schrift, Mehrsprachigkeit, Hilfe)
### Geändert
- App-Name überall auf "LMU RACECONTROL" vereinheitlicht
- Schriftarten Michroma/Inter werden jetzt beim Bauen automatisch heruntergeladen und fest in die App eingebaut
- App-Icon wird jetzt automatisch beim Bauen aus `src-tauri/icons/icon-source.png` erzeugt

### Hinzugefügt
- Deutsch/Englisch umschaltbar
- Neues Hilfe-Fenster (Klick auf "Hilfe" in der Sidebar)
- Automatischer, dauerhafter Installer-Download-Link (GitHub Release)

## [0.2.0] - Unveröffentlicht (Figma-Design-Umsetzung)
### Hinzugefügt
- Komplettes UI im Figma-Design umgesetzt
- Neues Datenmodell: Verursachender/Geschädigter Fahrer, Runde, Kurve, Zeitstempel, Vorfall-Art, Entscheidung, Begründung
- Explizites "Connect to Server" statt Auto-Verbindung
- Full-Course-Yellow-Workflow: Countdown-Overlay, automatische Geschwindigkeitsüberwachung
- Discord-Webhook-Benachrichtigung bei jeder Entscheidung
- Neuer "Einstellungen"-Bereich

## [0.1.0] - Unveröffentlicht (Erstgerüst)
### Hinzugefügt
- Grundgerüst als Tauri-2-App (Rust-Backend + React/TypeScript-Frontend)
- Client für die offizielle LMU REST-API (`localhost:6397`)
- Heuristische automatische Vorfall-Verdachtserkennung
- Manuelles Setzen von Vorfall-Markern
- SQLite-Persistenz aller Vorfälle
- Ein-Klick-Sprung ins LMU-Instant-Replay
- Windows-Installer (NSIS/MSI) via `cargo tauri build`