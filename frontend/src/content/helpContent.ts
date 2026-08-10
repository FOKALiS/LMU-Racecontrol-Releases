// ============================================================================
// HILFE-TEXT DER APP - HIER ÄNDERN
// ============================================================================
// Diese Datei zeigt den Inhalt, der erscheint, wenn jemand in der App auf
// "Hilfe" klickt. Du kannst hier jederzeit Texte ändern, ohne sonst etwas
// am Code zu verstehen:
//
//  - Jeder Abschnitt hat eine "heading" (Überschrift) und "paragraphs"
//    (eine Liste von Absätzen - für einen neuen Absatz einfach eine neue
//    Zeile in eckigen Klammern mit Anführungszeichen hinzufügen).
//  - Es gibt eine deutsche Version (helpContentDe) und eine englische
//    Version (helpContentEn) - beide bitte inhaltlich synchron halten.
//  - Wichtig: Text IMMER in doppelten Anführungszeichen " " lassen, und
//    nach jedem Absatz ein Komma nicht vergessen.
//  - Nach dem Ändern: Datei speichern, dann wie gewohnt über GitHub neu
//    hochladen (siehe ANLEITUNG-INSTALLER-BEKOMMEN.md) - der neue Text
//    erscheint automatisch in der nächsten gebauten Version.
// ============================================================================

export interface HelpSection {
  heading: string;
  paragraphs: string[];
}

export const helpContentDe: HelpSection[] = [
  {
    heading: "Über dieses Tool",
    paragraphs: [
      "LMU RACECONTROL ist das professionelle Race-Control-Tool für Le Mans Ultimate. Es unterstützt die Rennkommission dabei, Vorfälle während eines Rennens live zu erkennen, zu dokumentieren und zu entscheiden – alles an einem zentralen Arbeitsplatz.",
      "Das Tool verbindet sich als Zuschauer (Spectator) mit eurem LMU-Server und wertet die Live-Telemetrie in Echtzeit aus.",
      "Das Tool ist in verschiedene Lizenz-Stufen eingeteilt: Basic (einzelner Kommissar), Enterprise L (bis zu 3 Kommissare mit Server-Anbindung) und Enterprise XL (bis zu 5 Kommissare mit Server-Anbindung).",
    ],
  },
  {
    heading: "Erste Schritte – Verbindung herstellen",
    paragraphs: [
      "1. Starte Le Mans Ultimate und joine als Zuschauer (Spectator) auf dem gewünschten Server.",
      '2. Klicke in der App links in der Menüleiste auf "Connect to Server".',
      "3. Sobald die Verbindung steht, wechselt der Button auf grün und du kannst ins \"Fahrerfeld\" wechseln.",
      "4. Beim ersten Start wirst du nach deinem Lizenzschlüssel gefragt. Gib diesen ein und die Aktivierung erfolgt automatisch. Danach kannst du das Tool sofort nutzen.",
    ],
  },
  {
    heading: "Lizenz & Aktivierung (ausführlich)",
    paragraphs: [
      "LMU RACECONTROL benötigt eine gültige Lizenz. Deinen Lizenzschlüssel erhältst du nach dem Kauf per E-Mail von uns.",
      "Beim ersten Start öffnet sich ein Fenster, in dem du deinen Lizenzschlüssel (z. B. ABC123-DEF456-GHI789) eingibst. Die Aktivierung erfolgt automatisch und online – es wird eine Internetverbindung benötigt.",
      "Nach erfolgreicher Aktivierung wird dein Lizenz-Tier angezeigt (Basic, Enterprise L oder Enterprise XL) und du kannst das Tool in vollem Umfang nutzen.",
      "Deine Lizenz ist an deinen Rechner (Fingerprint) gebunden. Du kannst die Lizenz auf bis zu 3 (Enterprise L) bzw. 5 (Enterprise XL) verschiedenen Rechnern aktivieren.",
      "Bei einem Rechnerwechsel: Gehe in den Einstellungen auf \"Lizenz deaktivieren\" – dadurch wird dein alter Rechner freigegeben, und du kannst die Lizenz auf dem neuen Rechner mit dem selben Schlüssel erneut aktivieren.",
      "Hinweis: Die Deaktivierung setzt die lokale Lizenz zurück. Danach erscheint beim nächsten Start wieder der Aktivierungsdialog, wo du deinen Schlüssel erneut eingeben kannst.",
    ],
  },
  {
    heading: "Lizenz-Tier und Server-Funktionen",
    paragraphs: [
      "Basic: Du nutzt das Tool lokal ohne Server-Anbindung. Alle Vorfälle werden nur auf deinem Rechner gespeichert.",
      "Enterprise L (bis zu 3 User) und Enterprise XL (bis zu 5 User): Du erhältst Zugang zum LMU RACECONTROL Server.",
      "Mit Server-Anbindung können mehrere Kommissare gleichzeitig arbeiten. Vorfälle werden zentral auf dem Server gespeichert und sind für alle sichtbar.",
      "Wenn du einen Enterprise-Tier besitzt, erscheinen in den Einstellungen die Server-Felder (Server-URL und API-Key). Dort stellst du die Verbindung zu deinem Server her.",
    ],
  },
  {
    heading: "Server-Verbindung & API-Key abfragen",
    paragraphs: [
      "Wenn du einen Enterprise-Tier besitzt, kannst du in den Einstellungen unter \"Server Verbindung\" deine Server-URL eingeben (z. B. https://dein-server.com:8443).",
      "Klicke dann auf \"API-Key abfragen\" – die App holt sich automatisch den passenden API-Key zu deinem Lizenzschlüssel vom Server.",
      "Nach erfolgreicher Abfrage wird der API-Key eingetragen, und die Verbindung zu deinem Server steht.",
      "Der API-Key ist eine eindeutige ID, die deinem Team (Tenant) zugeordnet ist. Er wird benötigt, um Vorfälle zwischen den Kommissaren zu synchronisieren.",
      "Sollte die Abfrage fehlschlagen, überprüfe bitte die Server-URL und deine Internetverbindung. Bei anhaltenden Problemen kontaktiere den Administrator deines Servers.",
    ],
  },
  {
    heading: "Das Fahrerfeld",
    paragraphs: [
      "Das Fahrerfeld zeigt die Live-Timing-Daten aller Fahrzeuge in Echtzeit – Position, Klasse, Fahrername, Team, Fahrzeug, Geschwindigkeit, Status und schnellste Runde.",
      "Ein farbiger Punkt in der Spalte \"Vorfall\" zeigt einen automatisch erkannten Verdachtsfall an:",
      "• Rot = möglicher Crash\n• Gelb = Auffälligkeit\n• Weiß = ungewöhnlich langsames Fahrzeug",
      'Klicke bei einem markierten Fahrzeug auf "Investigate", um den Vorfall zu prüfen und zu entscheiden.',
      "Auf der Vorfall-Seite siehst du alle offenen Vorfälle. Über den Button \"Neuer Vorfall\" kannst du auch manuell einen Vorfall anlegen.",
    ],
  },
  {
    heading: "Vorfälle prüfen und entscheiden",
    paragraphs: [
      'Im Investigation-Fenster wählst du den verursachenden Fahrer (Fahrzeug A) und ggf. den geschädigten Fahrer (Fahrzeug B) aus. Ergänze Runde, Kurve und Zeitstempel, wähle die Vorfall-Art und die Entscheidung der Kommission aus und trage eine Begründung ein.',
      'Mit "Entscheidung absenden" wird der Vorfall archiviert. Er erscheint dann im Archiv-Bereich der Vorfall-Seite.',
      'Falls in den Einstellungen eine Discord-Webhook-URL konfiguriert ist, wird die Entscheidung automatisch in eurem Discord-Kanal gepostet.',
      "Wenn du eine Server-Anbindung hast, werden alle Vorfälle automatisch mit dem Server synchronisiert. So sehen alle Kommissare die gleichen Daten.",
      "Jeder Vorfall wird automatisch 26 Stunden gespeichert und danach entfernt. Archivierte Vorfälle mit Entscheidung bleiben bis dahin erhalten.",
    ],
  },
  {
    heading: "Discord-Webhook einrichten",
    paragraphs: [
      "Du kannst Vorfälle automatisch in deinen Discord-Kanal posten lassen. Dazu benötigst du eine Webhook-URL.",
      "So erstellst du einen Webhook in Discord:\n1. Gehe in Discord zu deinem Server\n2. Klicke auf das Zahnrad (Server-Einstellungen)\n3. Wähle \"Integrationen\" → \"Webhooks\"\n4. Klicke auf \"Neuen Webhook\"\n5. Gib einen Namen ein (z. B. \"LMU Race Control\")\n6. Wähle den gewünschten Textkanal aus\n7. Kopiere die Webhook-URL",
      "Die Webhook-URL trägst du in den Einstellungen unter \"Discord Webhook-URL\" ein. Ab dann werden alle getätigten Entscheidungen automatisch in Discord gepostet.",
      "Du kannst die Webhook-URL jederzeit ändern oder löschen, wenn du keine Benachrichtigungen mehr erhalten möchtest.",
    ],
  },
  {
    heading: "Einstellungen im Detail",
    paragraphs: [
      "Unter Einstellungen legst du alle wichtigen Parameter für dein Tool fest:",
      "• Vorfall-Kategorien: Hier definierst du die Arten von Vorfällen, die im Investigation-Fenster als Dropdown erscheinen (z. B. Kollision, Reifenstapel, Überholen unter Gelb, ...).",
      "• Entscheidungs-Optionen: Hier legst du die möglichen Entscheidungen fest (z. B. Verwarnung, Zeitstrafe, Durchfahrtsstrafe, ...).",
      "• FCY-Countdown: Sekunden bis zur aktiven FCY-Phase.",
      "• FCY-Tempolimit: Geschwindigkeit, die in der FCY-Phase nicht überschritten werden darf (in km/h).",
      "• Vorlaufzeit / Nachlaufzeit im Replay: Wie viele Sekunden vor bzw. nach einem Vorfall das Replay starten soll.",
      "• Discord Webhook-URL: Für automatische Benachrichtigungen (siehe separaten Abschnitt).",
      "• LMU-Pfad: Der Ordner, in dem Le Mans Ultimate installiert ist. Die Tastenbelegung wird automatisch ausgelesen.",
    ],
  },
  {
    heading: "Replay & Kamera-Steuerung",
    paragraphs: [
      "Über die Replay-Steuerung kannst du Vorfälle direkt im Replay ansehen:",
      "• Vorlaufzeit / Nachlaufzeit – Replay startet vor bzw. läuft nach dem Vorfall weiter",
      "• F7 – Rückwärts\n• F8 – Schnell Zurück\n• F9 – Vorspulen\n• F10 – Slow-Motion\n• F11 – Play/Pause",
      "Mit der Kamera-Steuerung wechselst du zwischen TV-, Bord- und Heck-Kamera und kannst per Zoom + / Zoom - hinein- und herauszoomen.",
      "Der Fahrer-Fokus setzt den Kamerafokus auf ein bestimmtes Fahrzeug. Du kannst ihn über einen Klick auf das Fahrzeug im Fahrerfeld oder über die Investigation-Seite auslösen.",
    ],
  },
  {
    heading: "Full Course Yellow (FCY)",
    paragraphs: [
      "Der gelbe FCY-Button startet einen Countdown. Nach Ablauf gilt die aktive FCY-Phase: Fahrzeuge, die das eingestellte Tempolimit überschreiten, werden automatisch als Vorfall markiert.",
      "Wichtig: Die App kann als Zuschauer kein echtes FCY auf dem Server auslösen – die Ansage an die Fahrer muss weiterhin über euren gewohnten Kanal erfolgen (z.B. Server-Nachricht oder Funk).",
      "Sobald die FCY-Phase beendet ist, klickst du auf den roten Button, um sie wieder zu deaktivieren.",
    ],
  },
  {
    heading: "Update & Version",
    paragraphs: [
      "LMU RACECONTROL sucht beim Start automatisch nach neuen Versionen im Internet.",
      "Wenn ein Update verfügbar ist, erscheint ein Hinweis und du kannst es direkt herunterladen und installieren – deine Einstellungen und deine Lizenz bleiben dabei erhalten.",
      "Du kannst die aktuelle Versionsnummer im Splashscreen beim Start ablesen.",
      "Falls du keinen Update-Hinweis erhältst, kannst du auch manuell auf unserer GitHub-Releases-Seite nach neuen Versionen suchen.",
    ],
  },
  {
    heading: "Datenbank leeren (Danger Zone)",
    paragraphs: [
      "In den Einstellungen gibt es unten den Bereich \"Datenbank leeren\". Damit werden ALLE Vorfälle (offene + archivierte) unwiderruflich gelöscht.",
      "Wenn du eine Server-Anbindung hast, werden auch die Vorfälle auf dem Server gelöscht.",
      "Sei vorsichtig: Diese Aktion kann nicht rückgängig gemacht werden. Verwende sie nur, wenn du wirklich alle Daten entfernen möchtest, z. B. für einen kompletten Neustart.",
    ],
  },
  {
    heading: "Tastenkürzel im Überblick",
    paragraphs: [
      "• F6 – Replay Stop (zum Anhalten nach Ablauf der Replay-Zeit)\n• F7 – Rückwärts abspielen\n• F8 – Schnell Zurück\n• F9 – Vorspulen\n• F10 – Slow-Motion\n• F11 – Play/Pause\n• R – Replay öffnen/Replay-Modus aktivieren",
    ],
  },
  {
    heading: "Hilfe & Support",
    paragraphs: [
      "Bei Fragen, Problemen oder Wünschen kontaktiere uns bitte über unsere Website: www.lmu-racecontrol.com",
      "Du möchtest deine Lizenz erweitern oder hast Fragen zu den Lizenz-Stufen? Auch dafür findest du alle Informationen auf unserer Website.",
    ],
  },
];

export const helpContentEn: HelpSection[] = [
  {
    heading: "About This Tool",
    paragraphs: [
      "LMU RACECONTROL is the professional race control tool for Le Mans Ultimate. It helps the race stewards detect, document, and decide on incidents live during a race – all from one central workstation.",
      "The tool connects to your LMU server as a spectator and analyzes live telemetry in real time.",
      "The tool is available in different license tiers: Basic (single steward), Enterprise L (up to 3 stewards with server sync), and Enterprise XL (up to 5 stewards with server sync).",
    ],
  },
  {
    heading: "Getting Started – Connecting",
    paragraphs: [
      "1. Start Le Mans Ultimate and join the desired server as a spectator.",
      '2. Click "Connect to Server" in the menu on the left in the app.',
      "3. Once connected, the button turns green and you can switch to \"Driver Field\".",
      "4. On first launch you will be prompted for your license key. Enter it and activation happens automatically. After that you can use the tool immediately.",
    ],
  },
  {
    heading: "License & Activation (detailed)",
    paragraphs: [
      "LMU RACECONTROL requires a valid license. You will receive your license key by email after purchase.",
      "On first launch, a dialog opens where you enter your license key (e.g. ABC123-DEF456-GHI789). Activation happens automatically and online – an internet connection is required.",
      "After successful activation, your license tier (Basic, Enterprise L, or Enterprise XL) is shown and you can use the tool in full.",
      "Your license is tied to your computer (fingerprint). You can activate the license on up to 3 (Enterprise L) or 5 (Enterprise XL) different computers.",
      "If you switch computers: Go to Settings and click \"Deactivate License\" – this frees up your old computer, and you can activate the license on the new machine using the same key.",
      "Note: Deactivation resets the local license. On next start, the activation dialog will appear again for you to re-enter your key.",
    ],
  },
  {
    heading: "License Tier & Server Features",
    paragraphs: [
      "Basic: You use the tool locally without server connectivity. All incidents are stored only on your computer.",
      "Enterprise L (up to 3 users) and Enterprise XL (up to 5 users): You get access to the LMU RACECONTROL Server.",
      "With server connectivity, multiple stewards can work simultaneously. Incidents are stored centrally on the server and visible to all.",
      "If you have an Enterprise tier, the server fields (Server URL and API Key) appear in Settings. There you set up the connection to your server.",
    ],
  },
  {
    heading: "Server Connection & Fetching API Key",
    paragraphs: [
      "If you have an Enterprise tier, you can enter your server URL in Settings under \"Server Connection\" (e.g. https://your-server.com:8443).",
      "Then click \"Fetch API Key\" – the app automatically retrieves the matching API key for your license key from the server.",
      "After successful retrieval, the API key is filled in and your server connection is established.",
      "The API key is a unique ID assigned to your team (tenant). It is required to synchronize incidents between stewards.",
      "If the fetch fails, please check the server URL and your internet connection. If problems persist, contact your server administrator.",
    ],
  },
  {
    heading: "The Driver Field",
    paragraphs: [
      "The Driver Field shows live timing data for all cars in real time – position, class, driver name, team, car, speed, status, and fastest lap.",
      "A colored dot in the \"Incident\" column indicates an automatically detected suspicion:",
      "• Red = possible crash\n• Yellow = anomaly\n• White = unusually slow car",
      'Click "Investigate" on a flagged car to review and decide the incident.',
      "On the Incidents page you can see all open incidents. Use \"New Incident\" to manually log an incident.",
    ],
  },
  {
    heading: "Reviewing and Deciding Incidents",
    paragraphs: [
      'In the Investigation window, select the causing driver (Vehicle A) and, if applicable, the affected driver (Vehicle B). Fill in lap, corner and timestamp, choose the incident type and the stewards\' decision, and add your reasoning.',
      '"Submit Decision" archives the incident. It then appears in the Archived section of the Incidents page.',
      'If a Discord webhook URL is configured in Settings, the decision is automatically posted to your Discord channel.',
      "If you have a server connection enabled, all incidents are automatically synced with the server. All stewards see the same data.",
      "Each incident is automatically stored for 26 hours and then removed. Archived incidents with a decision remain until then.",
    ],
  },
  {
    heading: "Setting up Discord Webhook",
    paragraphs: [
      "You can have incidents automatically posted to your Discord channel. You need a webhook URL.",
      "How to create a webhook in Discord:\n1. Go to your Discord server\n2. Click the gear icon (Server Settings)\n3. Select \"Integrations\" → \"Webhooks\"\n4. Click \"New Webhook\"\n5. Give it a name (e.g. \"LMU Race Control\")\n6. Select the desired text channel\n7. Copy the webhook URL",
      "Paste the webhook URL into Settings under \"Discord Webhook URL\". From then on, all submitted decisions will be automatically posted to Discord.",
      "You can change or remove the webhook URL at any time if you no longer want notifications.",
    ],
  },
  {
    heading: "Settings in Detail",
    paragraphs: [
      "In Settings you can configure all important parameters:",
      "• Incident Categories: Define the incident types that appear as dropdown in the Investigation window (e.g. Collision, Cutting the track, Overtaking under yellow, ...).",
      "• Decision Options: Define the possible decisions (e.g. Warning, Time Penalty, Drive-Through, ...).",
      "• FCY Countdown: Seconds until the active FCY phase begins.",
      "• FCY Speed Limit: Maximum speed allowed during FCY phase (in km/h).",
      "• Pre-roll / Post-roll in Replay: How many seconds before / after an incident the replay should start.",
      "• Discord Webhook URL: For automatic notifications (see separate section).",
      "• LMU Path: The folder where Le Mans Ultimate is installed. The key mapping is read automatically.",
    ],
  },
  {
    heading: "Replay & Camera Control",
    paragraphs: [
      "Use the replay controls to review incidents directly in the replay:",
      "• Pre-roll / Post-roll – replay starts before / continues after the incident",
      "• F7 – Reverse\n• F8 – Rewind Fast\n• F9 – Forward\n• F10 – Slow-Motion\n• F11 – Play/Pause",
      "With the camera controls you can switch between TV, onboard, and rear cameras and zoom in / out.",
      "Driver focus sets the camera focus on a specific car. You can trigger it by clicking a car in the Driver Field or from the Investigation page.",
    ],
  },
  {
    heading: "Full Course Yellow (FCY)",
    paragraphs: [
      "The yellow FCY button starts a countdown. Once it ends, the active FCY phase begins: cars exceeding the configured speed limit are automatically flagged as an incident.",
      "Important: as a spectator, the app cannot trigger a real FCY on the server itself – announcing it to drivers still has to happen through your usual channel (e.g. server message or radio).",
      "Once the FCY phase is over, click the red button to deactivate it again.",
    ],
  },
  {
    heading: "Updates & Version",
    paragraphs: [
      "LMU RACECONTROL automatically checks for updates on startup.",
      "When an update is available, a notification appears and you can download and install it directly – your settings and license are preserved.",
      "The current version number is shown in the splashscreen on startup.",
      "If you don't receive an update notification, you can also manually check for new versions on our GitHub Releases page.",
    ],
  },
  {
    heading: "Clear Database (Danger Zone)",
    paragraphs: [
      "At the bottom of Settings there is a \"Clear Database\" section. This will permanently delete ALL incidents (open + archived) on your local machine.",
      "If you have a server connection enabled, the incidents on the server will also be deleted.",
      "Be careful: This action cannot be undone. Only use it when you really want to remove all data, e.g. for a complete fresh start.",
    ],
  },
  {
    heading: "Keyboard Shortcuts Overview",
    paragraphs: [
      "• F6 – Replay Stop (to stop after the replay timer expires)\n• F7 – Reverse Playback\n• F8 – Rewind Fast\n• F9 – Forward\n• F10 – Slow-Motion\n• F11 – Play/Pause\n• R – Open Replay / Activate Replay Mode",
    ],
  },
  {
    heading: "Help & Support",
    paragraphs: [
      "If you have questions, problems, or suggestions, please contact us through our website: www.lmu-racecontrol.com",
      "Want to upgrade your license or have questions about the license tiers? You can find all information on our website as well.",
    ],
  },
];