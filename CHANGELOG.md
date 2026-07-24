# Changelog

Alle nennenswerten Änderungen an LMU Race Control werden hier dokumentiert.
Format angelehnt an [Keep a Changelog](https://keepachangelog.com/de/1.0.0/),
Versionierung nach [Semantic Versioning](https://semver.org/lang/de/).

## [0.6.5] - 24.07.2026 (Tastatur-Steuerung neu: Win32 SendInput, kein PowerShell-Flash, Fahrerfeld-Sortierung, Kamera-Auswahl aktiv)

### Behoben
- **Terminal-Fenster-Flash beim Fokussieren von LMU beseitigt**: PowerShell `AppActivate` durch native Win32 `FindWindowW`/`SetForegroundWindow` ersetzt – kein aufblitzendes Terminal mehr
- **Tastaturbefehle zuverlässiger**: `enigo`-Crate entfernt, stattdessen direkte Win32 `SendInput`-API mit Hintergrund-Thread-Architektur
- **Async-Blockierung behoben**: `std::thread::sleep` durch `tokio::time::sleep` in `focus_driver` ersetzt

### Geändert
- **Fahrerfeld wird jetzt nach Position sortiert** (1., 2., 3., ...) via `useMemo`
- **Kamera-Auswahl wird aktiv dargestellt**: Zentraler `selectedCam`-State in `App.tsx`, view-übergreifend an alle Views weitergegeben
- **`enigo`-Abhängigkeit entfernt**: `Cargo.toml` bereinigt
- **`keyboard.rs` komplett neu geschrieben**: Win32-API, Hintergrund-Thread, Eingabe-Puffer-Flush
- **Version**: 0.6.4 → 0.6.5

## [0.6.4] - 24.07.2026 (Icon-Größe: Logo auf Desktop/Taskleiste vergrößert)

### Behoben
- **Icon auf Desktop und Taskleiste war zu klein**: Das Logo hatte im Quellbild (`icon-source.png`) zu viel transparenten Rand. Dadurch wurde es bei der Skalierung auf kleine Icon-Größen (32x32) winzig dargestellt. Der transparente Rand wurde entfernt und das Logo füllt jetzt fast die gesamte Icon-Fläche.

### Geändert
- **Version**: 0.6.3 → 0.6.4

## [0.6.3] - 23.07.2026 (Icon-Fix: korrekte Windows-Icon-Generierung)

### Behoben
- **Windows-Icon (.ico) wurde nicht korrekt angezeigt**: Die `icon.ico` war fehlerhaft und zu klein für eine korrekte Anzeige in Desktop und Taskleiste. Mit dem Tauri Icon Generator (`npx @tauri-apps/cli icon`) aus der `icon-source.png` neu generiert (34.690 Bytes).
- **Alle Icons** (32x32.png, 128x128.png, 128x128@2x.png, icon.icns, icon.ico) wurden aus der Quelle neu erzeugt.

### Geändert
- **Version**: 0.6.2 → 0.6.3

## [0.6.1] - 22.07.2026 (Installer-Fix, Schriftarten lokal, Sidebar-Steuerung)

### Behoben
- **ERR_CONNECTION_REFUSED beim Start**: Dem Hauptfenster fehlte `"url": "index.html"`,
  sodass die installierte App versuchte, vom Dev-Server (localhost:1420) zu laden.
  Die App startet jetzt sofort, ohne Internetverbindung.
- **Schriftarten (Michroma/Inter) werden jetzt lokal eingebettet**: Die
  Schriftart-Dateien liegen als `.woff2` im `frontend/public/fonts/`-Verzeichnis
  und werden beim Bauen fest in die App integriert – kein Google-Fonts-Netzwerkzugriff
  mehr nötig. Die App funktioniert jetzt vollständig offline.

### Geändert
- **Sidebar-Steuerung** an die drei Zustände angepasst:
  - Ohne Lizenz: nur "Software Infos" (Sprache, Hilfe, Website, Footer)
  - Lizenziert, nicht verbunden: "Connect to Server" + "Software Infos"
  - Lizenziert + verbunden: alle Buttons (Fahrerfeld, Vorfälle, Archiv, FCY)
- **`beforeBuildCommand`** auf `npm --prefix frontend run build` umgestellt
  (löst Sonderzeichen-Probleme mit Umlauten im Pfad)
- **Version**: 0.6.0 → 0.6.1

## [0.6.0] - 22.07.2026 (Kamera-Steuerung, Fahrzeug-Fokus, Splashscreen-Design)

### Hinzugefügt
- **Kamera-Steuerung per Tastatursimulation**: Die Kamera-Buttons (TV, Helmet,
  Front, Heck, Top, Behind) simulieren jetzt die Tastendrücke F1-F6 direkt in
  LMU/rFactor2 - funktioniert, weil die LMU-REST-API keinen Kamera-Endpunkt
  bietet
- **Fahrzeug-Fokus per Tastatursimulation**: Klick auf einen Vorfall oder
  Doppelklick auf einen Fahrer springt zur richtigen Replay-Position, schaltet
  auf TV-Kamera und fokussiert das Fahrzeug via Strg+F + Fahrzeugnummer + Enter
- **Automatischer Replay-Sprung**: Der Replay-Sprung zur Vorfall-Position
  funktioniert jetzt zuverlässig über die LMU-REST-API

### Geändert
- **Splashscreen-Design überarbeitet**: Logo vergrößert (300px → 380px),
  Versionsnummer unter dem Logo platziert, gesamtes Layout optisch
  aufgewertet
- **Backend**: Tastatursimulation von `windows`-crate auf `enigo`-crate
  umgestellt (löst Versionskonflikte mit Tauri 2)
- **Version**: 0.5.4 → 0.6.0 (alle Versionsnummern aktualisiert)

### Technisch
- `keyboard.rs`: Neues Modul für Tastatursimulation mit `enigo`-crate
- `main.rs`: `set_camera` und `focus_driver` nutzen jetzt die
  Tastatursimulation statt fehlschlagender REST-API-Aufrufe
- `Cargo.toml`: `enigo = "0.2"` ersetzt `windows = "0.58"`

## [0.5.4] - Unveröffentlicht (Cam Control rechtsbündig)

### Geändert
- Image Control + Cam Control werden jetzt als EINE Einheit rechtsbündig
  ausgerichtet (gleiche Breite wie die Race-Control-Zeile darunter) - Image
  Control behält dabei seine Größe und rutscht nur als Ganzes mit nach
  rechts, Cam Control endet jetzt exakt am wahren rechten Rand statt mit
  Leerraum danach

## [0.5.3] - Unveröffentlicht (Race-Control-Buttons über volle Breite)

### Geändert
- "Neuer Vorfall"/"Erledigte Vorfälle"/"Full Course Yellow" bei Vorfälle
  spannen sich jetzt über die volle Breite von Image Control + Cam Control
  zusammen (Zeile darüber), Buttons gleichmäßig gestreckt statt nur
  natürlich rechtsbündig gepackt

## [0.5.2] - Unveröffentlicht (Fix: Aktivierung mit "Require Fingerprint Scope")

### Behoben
- Eure Keygen-Policy hat "Require Fingerprint Scope" aktiviert (jede
  Prüfung MUSS eine Geräte-Kennung mitschicken). Der erste Aktivierungs-
  Aufruf lief bisher bewusst OHNE diese Kennung, was mit dieser
  Policy-Einstellung sofort fehlgeschlagen wäre. Ab jetzt wird die
  Geräte-Kennung von Anfang an immer mitgeschickt.

## [0.5.1] - Unveröffentlicht (Keygen-Konto-ID eingetragen)

### Geändert
- `KEYGEN_ACCOUNT` in `src-tauri/src/license.rs` von Platzhalter auf die
  echte Keygen-Konto-ID umgestellt - Lizenzprüfung ist damit erstmals
  tatsächlich funktionsfähig (vorausgesetzt, die Policy in Keygen ist wie
  besprochen konfiguriert: Node-locked, Authentifizierungsstrategie
  License/Mixed)

## [0.5.0] - Unveröffentlicht (Lizenzsystem)

### Hinzugefügt
- Lizenzpflicht: ohne gültige Lizenz sind nur Startbildschirm (mit
  Lizenzschlüssel-Eingabe), "Hilfe" und der Website-Link nutzbar - alle
  anderen Funktionen (Fahrerfeld, Vorfälle, Archiv, Einstellungen,
  Connect to Server, FCY) sind gesperrt
- Anbindung an die Keygen-License-API (https://keygen.sh): Aktivierung pro
  Gerät, regelmäßige Online-Nachprüfung, 14 Tage Offline-Kulanzfrist
  (damit ein Renn-Wochenende mit schlechtem Internet niemanden aussperrt)
- Empfohlener Vertriebsweg: bestehender Wix-Shop bleibt Verkaufsstelle,
  Wix-Automation erzeugt bei Bestelleingang automatisch einen
  Lizenzschlüssel über die Keygen-API (Einrichtung folgt als separater
  Schritt, sobald das Keygen-Konto angelegt ist)

### Wichtig - vor dem nächsten Bau-Vorgang
- In `src-tauri/src/license.rs` den Platzhalter `KEYGEN_ACCOUNT` durch den
  echten Keygen-Account-Slug ersetzen - ohne diesen Schritt schlägt jede
  Lizenzprüfung fehl

## [0.4.2] - Unveröffentlicht (FCY-Hervorhebung, Button-Ausrichtung)

### Geändert
- Bei aktivem/ausgelöstem Full Course Yellow: gelber Rahmen um den
  Hauptbereich, "FULL COURSE YELLOW AKTIV"-Banner jetzt korrekt über dem
  Hauptbereich zentriert (statt über das gesamte Fenster inkl. Sidebar)
- Buttons "Neuer Vorfall"/"Erledigte Vorfälle"/"Full Course Yellow" bei
  Vorfälle sind jetzt linksbündig mit der "Cam Control"-Zeile darüber
  ausgerichtet
- Update-Strategie: Repository bleibt vorerst privat, Auto-Update-Erkennung
  greift automatisch, sobald das Projekt später auf "Public" gestellt wird -
  bis dahin Updates weiterhin manuell über die Releases-Seite herunterladen

## [0.4.1] - Unveröffentlicht (Grafik-Feinschliff)

### Geändert
- Sprachumschalter (DE/EN) aus dem Logo-Bereich in den Bereich "Software Infos"
  verschoben (war zu eng, wurde abgeschnitten)
- Website-Zeile (www.lmu-racecontrol.gg) in Sidebar UND Splashscreen ist jetzt
  klickbar und öffnet die Seite im Standardbrowser
- Sekunden-Eingabefelder bei "Vorlaufzeit"/"Nachlaufzeit" verbreitert, damit
  die Pfeile zum Ändern nicht mehr auf der Zahl kleben
- Tabellen-Kopfzeilen UND Datenzeilen haben jetzt durchgängig abgerundete
  Außenkanten (vorher unsichtbar durch eine CSS-Eigenschaft, die Rundungen
  an Tabellenzellen blockiert hat)
- Text auf der Startseite ("Willkommen bei...") deutlich verkleinert
- Splashscreen-Anzeigedauer von 10 auf 5 Sekunden verkürzt

## [0.4.0] - Unveröffentlicht (Splashscreen, Auto-Update, dynamische Versionsanzeige)

### Hinzugefügt
- Splashscreen-Fenster beim Programmstart (10 Sekunden, Logo, Version,
  Website-Zeile), danach automatischer Wechsel ins maximierte Hauptfenster
- Eingebauter Auto-Updater: Splashscreen prüft im Hintergrund auf neue
  Version; falls verfügbar, lädt ein Klick auf den grünen Balken das Update
  herunter, installiert es und startet die App neu
- Bau-Workflow nutzt jetzt die offizielle `tauri-apps/tauri-action`: signiert
  Updates kryptografisch, erstellt automatisch einen versionierten
  GitHub-Release (z.B. "v0.4.0") inkl. der für den Updater nötigen
  `latest.json` - ersetzt den bisherigen provisorischen "latest"-Release
- Neuer einmaliger Workflow `generate-update-key.yml` zum Erzeugen des
  Signierschlüsselpaars (siehe ANLEITUNG-INSTALLER-BEKOMMEN.md, Schritt 0)
- Versionsanzeige in der Sidebar ist jetzt dynamisch (liest die echte
  App-Version aus, statt fest im Text zu stehen)

### Wichtig
- Vor dem nächsten Bau-Vorgang muss einmalig das Signierschlüsselpaar erzeugt
  und eingerichtet werden (Anleitung, Schritt 0) - sonst schlägt der Bau fehl
- Künftige Updates veröffentlichen: Versionsnummer in drei Dateien erhöhen
  (`tauri.conf.json`, `Cargo.toml`, `package.json`), dann wie gewohnt hochladen

## [0.3.0] - Unveröffentlicht (Feinschliff: Name, Icon, Schrift, Mehrsprachigkeit, Hilfe)

### Geändert
- App-Name überall auf "LMU RACECONTROL" vereinheitlicht (Startmenü, Fenstertitel,
  Deinstallations-Eintrag)
- Schriftarten Michroma/Inter werden jetzt beim Bauen automatisch heruntergeladen
  und FEST in die App eingebaut (vorher: Google-Fonts-Link wurde von der
  Sicherheitsrichtlinie der App blockiert, deshalb erschien Arial statt Michroma)
- App-Icon wird jetzt automatisch beim Bauen aus `src-tauri/icons/icon-source.png`
  erzeugt (alle Größen/Formate) - einfach dieses eine Bild ersetzen, um das
  Icon zu ändern

### Hinzugefügt
- Deutsch/Englisch umschaltbar: automatische Erkennung nach Systemsprache beim
  ersten Start, dauerhaft merkbarer Umschalter (DE/EN) oben in der Sidebar
- Neues Hilfe-Fenster (Klick auf "Hilfe" in der Sidebar) mit Kurzübersicht zur
  Bedienung - Text frei editierbar in `frontend/src/content/helpContent.ts`
- Automatischer, dauerhafter Installer-Download-Link (GitHub Release) zusätzlich
  zum bisherigen 90-Tage-Artifact

## [0.2.0] - Unveröffentlicht (Figma-Design-Umsetzung)

### Hinzugefügt
- Komplettes UI im Figma-Design umgesetzt: Sidebar mit Logo/Navigation,
  Fahrerfeld, Vorfälle, Archiv, Investigation-Modal
- Neues Datenmodell: Verursachender/Geschädigter Fahrer, Runde, Kurve,
  Zeitstempel, Vorfall-Art, Entscheidung, Begründung
- Explizites "Connect to Server" statt Auto-Verbindung beim Start
- Full-Course-Yellow-Workflow: Countdown-Overlay, danach automatische
  Geschwindigkeitsüberwachung mit automatischer Vorfall-Markierung bei
  Verstößen gegen das konfigurierte Tempolimit
- Rot/Gelb/Weiß-Statuspunkte im Fahrerfeld (Crash-Verdacht / auffällige
  Pace-Anomalie / langsames Fahrzeug) - siehe "Bekannte Lücken"
- Discord-Webhook-Benachrichtigung bei jeder Entscheidung der Kommission
- Neuer "Einstellungen"-Bereich (nicht im Mockup enthalten, aber
  notwendig): Vorfall-Kategorien, Entscheidungs-Optionen, Discord-Webhook,
  FCY-Parameter - lokal pro Installation gespeichert
- App-Icons aus dem echten LMU-Racecontrol-Logo generiert

### Bekannt fehlend / zu verifizieren
- Feld für Live-Geschwindigkeit (`speed_kmh`, für FCY-Überwachung) und
  Fahrzeugmodell (`car_model`) noch nicht gegen echte LMU-Instanz verifiziert
- Gelb/Weiß-Marker sind Heuristiken (Pace-Anomalie / langsames Fahrzeug im
  Feldvergleich), kein bestätigtes Flaggen-Feld in der REST-API gefunden
- "Zeitstempel"/Rundenbezug basiert auf verstrichener Echtzeit seit
  "Connect to Server", nicht auf einem bestätigten Session-Zeit-Feld

## [0.1.0] - Unveröffentlicht (Erstgerüst)

### Hinzugefügt
- Grundgerüst als Tauri-2-App (Rust-Backend + React/TypeScript-Frontend)
- Client für die offizielle LMU REST-API (`localhost:6397`):
  Live-Timing (`/rest/watch/standings`), Sessioninfo, Replay-Zeitsprung
  (`/rest/watch/replaytime/{s}`), Kamerafokus (`/rest/watch/focus/{slot}`)
- Heuristische automatische Vorfall-Verdachtserkennung (Rundenzeit- und
  Positionsanomalien)
- Manuelles Setzen von Vorfall-Markern durch den Kommissar
- SQLite-Persistenz aller Vorfälle inkl. Status-Workflow
  (Verdachtsfall → In Prüfung → Keine Maßnahme / Strafe verhängt)
- Ein-Klick-Sprung ins LMU-Instant-Replay mit konfigurierbarem Pre-/Post-Roll
- Windows-Installer (NSIS/MSI) via `cargo tauri build`

### Bekannt fehlend / zu verifizieren
- Exaktes JSON-Feld für die "seit Session-/Replaybeginn verstrichene Zeit"
  in `/rest/watch/sessionInfo` ist noch nicht gegen eine echte laufende
  LMU-Instanz verifiziert (siehe README, Abschnitt "Bekannte Lücken")
- Kein bestätigter REST-Endpunkt für Schaden/Kontakt pro gegnerischem
  Fahrzeug gefunden - automatische Erkennung arbeitet daher mit
  Pace-/Positionsanomalien statt direkter Kollisionserkennung
