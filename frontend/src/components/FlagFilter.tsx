/// Schaltflächen zum Filtern von Vorfällen nach Flaggen-Farbe.
///
/// Die Kommissare können wählen, welche Vorfälle sie sehen möchten:
/// - ROT (Crash-Verdacht)
/// - GELB (gelbe Flagge / Pace-Anomalie)
/// - WEISS (langsames Fahrzeug)
///
/// Standardmäßig sind alle drei aktiv. Ein Klick auf eine Farbe schaltet
/// sie aus (die Vorfälle dieser Farbe werden ausgeblendet).

import type { FlagColor } from "../types";

interface Props {
  showRed: boolean;
  showYellow: boolean;
  showWhite: boolean;
  onChange: (color: FlagColor, show: boolean) => void;
}

export default function FlagFilter({ showRed, showYellow, showWhite, onChange }: Props) {
  return (
    <div className="flag-filter">
      <span className="flag-filter-label">Filter:</span>
      <button
        className={`flag-filter-btn flag-filter-red ${showRed ? "active" : ""}`}
        onClick={() => onChange("Red", !showRed)}
      >
        ● Crash
      </button>
      <button
        className={`flag-filter-btn flag-filter-yellow ${showYellow ? "active" : ""}`}
        onClick={() => onChange("Yellow", !showYellow)}
      >
        ● Gelb
      </button>
      <button
        className={`flag-filter-btn flag-filter-white ${showWhite ? "active" : ""}`}
        onClick={() => onChange("White", !showWhite)}
      >
        ● Langsam
      </button>
    </div>
  );
}