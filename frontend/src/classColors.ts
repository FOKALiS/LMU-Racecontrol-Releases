// Farbzuordnung der Fahrzeugklassen. Bei Bedarf hier erweitern/anpassen,
// falls LMU andere Klassennamen liefert als erwartet (z.B. "GTE" statt
// "Hypercar" bei älteren Events) - einfach eine weitere Zeile ergänzen.
export function classColor(className: string): string {
  const c = className.toLowerCase();
  if (c.includes("hyper") || c.includes("lmh") || c.includes("gtp")) return "red";
  if (c.includes("lmp2")) return "blue";
  if (c.includes("lmp3")) return "purple";
  if (c.includes("lmgt3") || c.includes("gt3")) return "green";
  return "green"; // Fallback, falls eine unbekannte Klasse auftaucht
}
