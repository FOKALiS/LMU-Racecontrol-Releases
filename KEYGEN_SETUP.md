# Keygen Setup – So stellst Du den Lizenz-Tier ein

## Schritt-für-Schritt

1. **In Keygen einloggen** → https://keygen.sh
2. **Links auf "Licenses" klicken**
3. **Eine License anklicken** (die Du bearbeiten willst)
4. **Runter scrollen zu "Metadata"**
5. **Auf "Add Metadata" klicken**

Dann erscheinen zwei Felder:

| Feld | Eingabe |
|------|---------|
| **Key** | `tier` |
| **Type** | `String` (auswählen) |
| **Value** | `basic` (oder `demo`, `enterprise_l`, `enterprise_xl`) |

## Beispiel für Deine Lizenzen

| Lizenz | Key | Type | Value |
|--------|-----|------|-------|
| Deine Basic-Lizenz | `tier` | String | `basic` |
| Deine Enterprise L-Lizenz | `tier` | String | `enterprise_l` |
| Deine Enterprise XL-Lizenz | `tier` | String | `enterprise_xl` |
| Demo-Lizenz | `tier` | String | `demo` |

## Wichtig
- **Key:** genau `tier` (Kleinbuchstaben)
- **Type:** `String` (nicht Integer, nicht Boolean!)
- **Value:** genau `basic` / `demo` / `enterprise_l` / `enterprise_xl`

**Fertig!** Nach dem Speichern erkennt die App beim nächsten Start den Lizenz-Tier automatisch.