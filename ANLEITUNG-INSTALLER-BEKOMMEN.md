# Anleitung: Installer bekommen, ohne selbst zu programmieren

Diese Anleitung braucht **keine Programmierkenntnisse**. Du lädst nur Dateien
hoch und klickst ein paar Buttons. Ein Rechner von GitHub (kostenloser Dienst
von Microsoft für genau solche Zwecke) baut die fertige Installationsdatei
für dich.

---

## ⚠️ Schritt 0 (NEU, einmalig, WICHTIG): Auto-Update einrichten

Seit dieser Version hat die App eine eingebaute Auto-Update-Funktion (Update-
Prüfung im Startbildschirm, automatische Installation per Klick). Damit das
funktioniert, braucht die App einen "Sicherheitsschlüssel", der beweist, dass
ein Update wirklich von euch kommt und nicht von jemand Fremdem. **Diesen
Schritt musst du nur EINMAL machen** - danach nie wieder.

**Falls du diesen Schritt überspringst, schlägt der nächste Bau-Vorgang
(Schritt 4) mit einem Fehler fehl!**

### 0.1 Schlüsselpaar erzeugen lassen

1. Auf deiner GitHub-Repository-Seite auf den Reiter **"Actions"** klicken
2. Links in der Liste auf **"(Einmalig) Update-Signierschlüssel erzeugen"**
   klicken
3. Rechts auf **"Run workflow"** → nochmal **"Run workflow"** klicken
4. Kurz warten (ca. 1-2 Minuten), bis ein grüner Haken erscheint
5. Auf den fertigen (grünen) Durchlauf klicken, dann auf den einzigen Schritt
   darin ("Schlüsselpaar erzeugen und im Log anzeigen") - dort klappt ein
   Protokoll auf mit zwei wichtigen Textblöcken:
   - **"OEFFENTLICHER SCHLUESSEL"** - eine lange Zeichenkette
   - **"PRIVATER SCHLUESSEL"** - eine noch längere Zeichenkette

### 0.2 Öffentlichen Schlüssel eintragen

1. In deinem entpackten Projektordner die Datei `src-tauri/tauri.conf.json`
   mit einem einfachen Texteditor öffnen (z.B. Windows-Editor/Notepad,
   Rechtsklick → "Öffnen mit" → "Editor")
2. Ganz unten den Platzhaltertext `HIER_OEFFENTLICHEN_SCHLUESSEL_EINFUEGEN`
   finden und durch den kopierten **öffentlichen** Schlüssel aus 0.1 ersetzen
   (die Anführungszeichen `"..."` drumherum stehen lassen)
3. Direkt darüber: `DEIN-GITHUB-NAME/DEIN-REPO-NAME` durch deinen echten
   GitHub-Benutzernamen und Repository-Namen ersetzen, z.B. wenn deine
   Repository-Adresse `https://github.com/mweggel/lmu-race-control` lautet,
   dann trägst du `mweggel/lmu-race-control` ein
4. Datei speichern

### 0.3 Privaten Schlüssel als Geheimnis speichern

**Wichtig: Der private Schlüssel darf NIEMALS in eine Datei im Projekt oder
irgendwo öffentlich landen** - deshalb kommt er stattdessen in die
GitHub-"Secrets" (verschlüsselter, versteckter Speicher):

1. Auf GitHub im Repository: **"Settings"** (Zahnrad-Symbol oben) → links
   **"Secrets and variables"** → **"Actions"**
2. **"New repository secret"** klicken
   - Name: `TAURI_SIGNING_PRIVATE_KEY`
   - Value: den kopierten **privaten** Schlüssel aus 0.1 einfügen
   - **"Add secret"** klicken
3. Noch ein zweites Secret anlegen:
   - Name: `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`
   - Value: leer lassen (Feld einfach leer lassen, Secret trotzdem speichern)
   - **"Add secret"** klicken

Das war's - ab jetzt läuft die Update-Signierung bei jedem Bau-Vorgang
automatisch mit, ganz ohne dass du dich weiter darum kümmern musst.

---

## Vorab (optional): Eigenes App-Icon einsetzen

Falls du ein eigenes Icon für die Taskleiste verwenden willst: Speichere dein
Icon als **quadratisches PNG mit transparentem Hintergrund** (mind. 512x512px,
besser 1024x1024px) und ersetze damit die Datei
`src-tauri/icons/icon-source.png` in deinem entpackten Projektordner (gleicher
Dateiname, einfach die alte Datei überschreiben). Der Rest (alle Icon-Größen,
.ico, .icns) wird beim Bau-Vorgang automatisch daraus erzeugt.

---

## Schritt 1: Kostenlosen GitHub-Account erstellen

Falls noch nicht vorhanden: auf **https://github.com/signup** gehen, E-Mail-Adresse
und Passwort eingeben, fertig. Kostet nichts.

## Schritt 2: Neues Repository erstellen

Ein "Repository" ist einfach ein Projekt-Ordner auf GitHub. (Falls du schon
eins aus einer früheren Version hast, überspringe diesen Schritt.)

1. Oben rechts auf das **"+"**-Symbol klicken → **"New repository"**
2. Name eingeben, z.B. `lmu-race-control`
3. **"Private"** auswählen (nur ihr könnt es sehen)
4. Unten auf **"Create repository"** klicken

## Schritt 3: Projekt-Dateien hochladen

1. Die ZIP-Datei, die ich dir gegeben habe, auf deinem Rechner **entpacken**
   (Rechtsklick → "Alle extrahieren" / "Entpacken")
2. Im entpackten Ordner **`lmu-race-control`** öffnen - dort siehst du Ordner
   wie `src-tauri`, `frontend`, Dateien wie `README.md`
3. Zurück im Browser, auf deiner GitHub-Repository-Seite: Link
   **"uploading an existing file"** anklicken (steht auf der leeren
   Repository-Startseite) bzw. **"Add file" → "Upload files"**, falls schon
   Dateien vorhanden sind
4. **Alle Dateien und Ordner** aus dem `lmu-race-control`-Ordner in das
   Browser-Fenster ziehen (per Drag & Drop) - wichtig: die *Inhalte* des
   Ordners hochladen, nicht den Ordner selbst nochmal drumherum
5. Unten eine kurze Nachricht eingeben (z.B. "Version 0.4") und auf
   **"Commit changes"** klicken

Falls der Browser-Upload bei der Dateimenge streikt: Alternative ist die
kostenlose **GitHub Desktop**-App (https://desktop.github.com) - dort "Add
local repository" wählen, den entpackten Ordner auswählen, auf "Publish
repository" bzw. "Push" klicken.

## Schritt 4: Automatischen Bau-Vorgang starten

1. Oben im Repository auf den Reiter **"Actions"** klicken
2. Links auf **"Windows-Installer bauen und veröffentlichen"** klicken
3. Rechts auf **"Run workflow"** → nochmal **"Run workflow"** klicken (falls
   er nicht schon automatisch nach dem Hochladen gestartet ist)

## Schritt 5: Warten

Ein gelber Punkt zeigt "läuft noch", ein grüner Haken zeigt "fertig" (dauert
ca. 10-15 Minuten). Die Seite muss dafür nicht offen bleiben.

## Schritt 6: Installer herunterladen

1. Auf der GitHub-Repository-Seite rechts auf **"Releases"** klicken (falls
   nicht sichtbar: unter "About" auf der rechten Seitenleiste)
2. Der neueste Release (z.B. "LMU RACECONTROL v0.4.0") zeigt unter "Assets"
   die fertige `.exe`-Datei (und `.msi`) zum Herunterladen
3. Diesen Link kannst du dauerhaft an alle Rennkommissare weitergeben - er
   bleibt bestehen und läuft nicht ab

## Schritt 7: Installieren

Die `.exe`-Datei auf den Rechner des Rennkommissars kopieren und doppelklicken.
Windows zeigt eventuell eine SmartScreen-Warnung ("Unbekannter Herausgeber") -
auf **"Weitere Informationen"** → **"Trotzdem ausführen"** klicken. Das ist
normal bei selbst gebauten, internen Tools.

---

## Wie ihr künftig Updates veröffentlicht

Sobald ihr Änderungen am Tool vorgenommen habt:

1. In **drei Dateien** die Versionsnummer erhöhen (z.B. von `0.4.0` auf `0.4.1`):
   - `src-tauri/tauri.conf.json` (Feld `"version"`)
   - `src-tauri/Cargo.toml` (Feld `version = "..."`)
   - `frontend/package.json` (Feld `"version"`)
2. Geänderte Dateien wie in Schritt 3 erneut hochladen
3. Der Bau-Vorgang läuft automatisch (Schritt 4/5) und veröffentlicht einen
   neuen Release mit der neuen Versionsnummer

Alle bereits installierten Apps zeigen dann beim nächsten Start im
Startbildschirm automatisch **"UPDATE VERFÜGBAR"** an - ein Klick darauf lädt
das Update herunter, installiert es und startet die App neu.

---

## Was der Startbildschirm macht

Nach dem Doppelklick auf das App-Icon erscheint für ca. 10 Sekunden ein
Startbildschirm mit Logo, aktueller Versionsnummer und eurer Website-Zeile.
Im Hintergrund prüft die App dabei automatisch, ob eine neuere Version
veröffentlicht wurde:

- **Kein Update verfügbar**: Nach 10 Sekunden öffnet sich automatisch das
  maximierte Hauptfenster.
- **Update verfügbar**: Ein grüner Balken "UPDATE VERFÜGBAR / Version X.X.X"
  erscheint zusätzlich. Klickt der Kommissar darauf, wird das Update
  heruntergeladen, installiert, und die App startet automatisch neu - alles
  ohne manuelles Herunterladen oder Installer-Suchen.

---

## Falls etwas schiefgeht

- **Roter X-Haken statt grünem Haken bei Schritt 5**: Auf den Durchlauf
  klicken, dort steht in rot, welcher Schritt fehlgeschlagen ist. Den Text
  kannst du mir hier im Chat schicken (Screenshot reicht), dann schaue ich,
  woran es liegt. Häufigster Grund nach diesem Update: Schritt 0
  (Signierschlüssel) wurde übersprungen oder die Secrets falsch benannt.
- **Kein Freund/keine Freundin zur Hand, der/die sich auskennt**: Du kannst
  diese Anleitung + die ZIP-Datei auch an eine Person schicken, die sich ein
  kleines bisschen mit GitHub auskennt - der ganze Vorgang oben braucht
  selbst für Programmierer keine Programmierkenntnisse, nur Klicks.
