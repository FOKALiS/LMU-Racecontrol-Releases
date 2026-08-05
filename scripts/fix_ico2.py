"""Erstellt icon.ico mit allen Windows-Groessen (16-256px) - BMP-basiert fuer maximale Kompatibilitaet."""
import os
from PIL import Image

# Lade die Quelle (1024x1024)
img = Image.open("src-tauri/icons/icon-source.png")
print(f"Quelle: {img.size}, Mode: {img.mode}")

# ICO-Groessen
ico_sizes = [16, 24, 32, 48, 64, 128, 256]

# Alle Groessen als RGBA vorbereiten
images = []
for size in ico_sizes:
    if size == 256:
        # Windows erwartet 0 fuer 256px in ICO, PIL macht das automatisch
        resized = img.resize((256, 256), Image.LANCZOS)
    else:
        resized = img.resize((size, size), Image.LANCZOS)
    # In RGBA konvertieren (falls nicht schon)
    if resized.mode != "RGBA":
        resized = resized.convert("RGBA")
    images.append(resized)

# PIL ICO speichern - das erste Bild definiert die Basis-Groesse
# und append_images ergaenzen die restlichen Groessen
path = "src-tauri/icons/icon.ico"
# WICHTIG: PIL braucht das Bild MIT Alpha-Kanal und die Groessen als sizes-Liste
base = images[-1]  # 256x256 als Basis
base.save(
    path,
    format="ICO",
    sizes=[(s, s) for s in ico_sizes],
    append_images=images[:-1],
)

# Verifizieren
with Image.open(path) as check:
    print(f"Erstellt: {os.path.getsize(path)} Bytes")
    print(f"Groessen: {sorted(check.info.get('sizes', []))}")