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
      "LMU RACECONTROL unterstützt die Rennkommission bei Le Mans Ultimate dabei, Vorfälle während eines Rennens live zu erkennen, zu dokumentieren und zu entscheiden.",
    ],
  },
  {
    heading: "Verbindung herstellen",
    paragraphs: [
      "Starte Le Mans Ultimate und joine als Zuschauer (Spectator) auf dem gewünschten Server.",
      'Klicke danach in der App links auf "Connect to Server". Sobald die Verbindung steht, wechselt der Button auf grün und du kannst ins "Fahrerfeld" wechseln.',
    ],
  },
  {
    heading: "Fahrerfeld",
    paragraphs: [
      "Zeigt die Live-Timing-Daten aller Fahrzeuge. Ein farbiger Punkt in der Spalte \"Vorfall\" zeigt einen automatisch erkannten Verdachtsfall an: Rot = möglicher Crash, Gelb = Auffälligkeit, Weiß = ungewöhnlich langsames Fahrzeug.",
      'Klicke bei einem markierten Fahrzeug auf "Investigate", um den Vorfall zu prüfen und zu entscheiden.',
    ],
  },
  {
    heading: "Vorfälle prüfen und entscheiden",
    paragraphs: [
      'Im Investigation-Fenster wählst du den verursachenden und ggf. geschädigten Fahrer aus, ergänzt Runde, Kurve und Zeitstempel, wählst die Vorfall-Art und die Entscheidung der Kommission aus und trägst eine Begründung ein.',
      'Mit "Entscheidung absenden" wird der Vorfall archiviert und - falls in den Einstellungen konfiguriert - automatisch in eurem Discord-Kanal gepostet.',
      'Über "Neuer Vorfall" auf der Vorfälle-Seite kannst du auch manuell einen Vorfall anlegen, der nicht automatisch erkannt wurde.',
    ],
  },
  {
    heading: "Full Course Yellow (FCY)",
    paragraphs: [
      "Der gelbe FCY-Button startet einen Countdown. Nach Ablauf gilt die aktive FCY-Phase: Fahrzeuge, die das eingestellte Tempolimit überschreiten, werden automatisch als Vorfall markiert.",
      "Wichtig: Die App kann als Zuschauer kein echtes FCY auf dem Server auslösen - die Ansage an die Fahrer muss weiterhin über euren gewohnten Kanal erfolgen (z.B. Server-Nachricht oder Funk).",
    ],
  },
  {
    heading: "Einstellungen",
    paragraphs: [
      "Unter Einstellungen legt ihr eure eigenen Vorfall-Kategorien und Entscheidungs-Optionen fest, die im Investigation-Fenster als Dropdown erscheinen, sowie die Discord-Webhook-URL und die FCY-Parameter.",
      "Diese Einstellungen gelten nur für den jeweiligen Rechner und müssen auf jedem Kommissars-PC einmal eingerichtet werden.",
    ],
  },
];

export const helpContentEn: HelpSection[] = [
  {
    heading: "About This Tool",
    paragraphs: [
      "LMU RACECONTROL helps the race stewards for Le Mans Ultimate detect, document, and decide on incidents live during a race.",
    ],
  },
  {
    heading: "Connecting",
    paragraphs: [
      "Start Le Mans Ultimate and join the desired server as a spectator.",
      'Then click "Connect to Server" on the left in the app. Once connected, the button turns green and you can switch to "Driver Field".',
    ],
  },
  {
    heading: "Driver Field",
    paragraphs: [
      'Shows live timing data for all cars. A colored dot in the "Incident" column indicates an automatically detected suspicion: red = possible crash, yellow = anomaly, white = unusually slow car.',
      'Click "Investigate" on a flagged car to review and decide the incident.',
    ],
  },
  {
    heading: "Reviewing and Deciding Incidents",
    paragraphs: [
      "In the Investigation window, select the causing driver and, if applicable, the affected driver, fill in lap, corner and timestamp, choose the incident type and the stewards' decision, and add your reasoning.",
      '"Submit Decision" archives the incident and - if configured in Settings - automatically posts it to your Discord channel.',
      'Use "New Incident" on the Incidents page to manually log an incident that wasn\'t detected automatically.',
    ],
  },
  {
    heading: "Full Course Yellow (FCY)",
    paragraphs: [
      "The yellow FCY button starts a countdown. Once it ends, the active FCY phase begins: cars exceeding the configured speed limit are automatically flagged as an incident.",
      "Important: as a spectator, the app cannot trigger a real FCY on the server itself - announcing it to drivers still has to happen through your usual channel (e.g. server message or radio).",
    ],
  },
  {
    heading: "Settings",
    paragraphs: [
      "Under Settings you define your own incident categories and decision options, which appear as dropdowns in the Investigation window, as well as the Discord webhook URL and FCY parameters.",
      "These settings only apply to the computer they're set on and need to be configured once on every steward's PC.",
    ],
  },
];
