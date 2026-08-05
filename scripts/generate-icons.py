"""
Generiert alle benoetigten Icons und Logos aus den Quell-Dateien.
Loescht vorher alle alten Dateien im Zielordner.

Quellen:
  - Logo: LMU RC Logo - hell.* (breites Banner, 1024x486)
  - Icon: LMU RC - Icon transparent.* (quadratisch, 1024x1024)
"""
import os
import shutil
from PIL import Image

SRC_DIR = r"C:\Users\Administrator\Documents\AI\Software Entwicklung\LMU Racecontrol\Logo"
ICONS_DIR = r"C:\Users\Administrator\Documents\AI\Software Entwicklung\LMU Racecontrol\Tool\lmu-race-control\src-tauri\icons"
PUBLIC_DIR = r"C:\Users\Administrator\Documents\AI\Software Entwicklung\LMU Racecontrol\Tool\lmu-race-control\frontend\public"

SRC_LOGO_PNG = os.path.join(SRC_DIR, "LMU RC Logo - hell.png")
SRC_LOGO_SVG = os.path.join(SRC_DIR, "LMU RC Logo - hell.svg")
SRC_ICON_PNG = os.path.join(SRC_DIR, "LMU RC - Icon transparent.png")

# Alle benoetigten PNG-Groessen fuer ICONS (quadratisch)
ICON_SIZES = [
    (32, 32, "32x32.png"),
    (64, 64, "64x64.png"),
    (128, 128, "128x128.png"),
    (256, 256, "128x128@2x.png"),
    (30, 30, "Square30x30Logo.png"),
    (44, 44, "Square44x44Logo.png"),
    (71, 71, "Square71x71Logo.png"),
    (89, 89, "Square89x89Logo.png"),
    (107, 107, "Square107x107Logo.png"),
    (142, 142, "Square142x142Logo.png"),
    (150, 150, "Square150x150Logo.png"),
    (284, 284, "Square284x284Logo.png"),
    (310, 310, "Square310x310Logo.png"),
    (512, 512, "icon.png"),
    (1024, 1024, "icon-source.png"),
]

def clean_icons_dir():
    """Loescht alle Dateien im icons-Ordner, behaelt aber android/ und ios/."""
    for entry in os.listdir(ICONS_DIR):
        path = os.path.join(ICONS_DIR, entry)
        if entry in ("android", "ios"):
            continue
        if os.path.isfile(path):
            os.remove(path)
            print(f"  Geloescht: {entry}")
        elif os.path.isdir(path):
            shutil.rmtree(path)
            print(f"  Geloescht (Ordner): {entry}")

def generate_icon_pngs(img):
    """Generiert alle PNG-Icons in den benoetigten Groessen."""
    for w, h, filename in ICON_SIZES:
        resized = img.resize((w, h), Image.LANCZOS)
        path = os.path.join(ICONS_DIR, filename)
        resized.save(path, "PNG")
        print(f"  Icon erstellt: {filename} ({w}x{h})")

def generate_ico(img):
    """Generiert icon.ico mit mehreren Groessen fuer Windows (16-256px)."""
    import struct
    import io

    ico_sizes = [16, 24, 32, 48, 64, 128, 256]

    # PNG-Daten fuer jede Groesse generieren
    png_data_list = []
    for size in ico_sizes:
        resized = img.resize((size, size), Image.LANCZOS)
        png_bytes = io.BytesIO()
        resized.save(png_bytes, format="PNG")
        png_data_list.append(png_bytes.getvalue())

    # ICO-Datei schreiben (Header + Directory + PNG-Daten)
    path = os.path.join(ICONS_DIR, "icon.ico")
    with open(path, "wb") as f:
        # ICO Header: reserved=0, type=1 (ICO), count
        f.write(struct.pack("<HHH", 0, 1, len(ico_sizes)))

        # Directory entries
        offset = 6 + 16 * len(ico_sizes)
        for i, size in enumerate(ico_sizes):
            png_data = png_data_list[i]
            # width, height (0 = 256), colors, reserved, planes, bpp, size, offset
            w = 0 if size == 256 else size
            h = 0 if size == 256 else size
            f.write(struct.pack("<BBBBHHII", w, h, 0, 0, 1, 32, len(png_data), offset))
            offset += len(png_data)

        # Alle PNG-Daten schreiben
        for png_data in png_data_list:
            f.write(png_data)

    print(f"  Icon erstellt: icon.ico (Groessen: {ico_sizes}, {os.path.getsize(path)} Bytes)")

def generate_icns(img):
    """Generiert icon.icns fuer macOS (1024x1024)."""
    path = os.path.join(ICONS_DIR, "icon.icns")
    img.save(path, "PNG")
    print(f"  Icon erstellt: icon.icns (als PNG-Fallback, 1024x1024)")

def copy_logo():
    """Kopiert das Logo (hell/Vollversion) nach frontend/public/."""
    # Logo als PNG (1024x486 skaliert auf 512x243 oder passend)
    img = Image.open(SRC_LOGO_PNG)
    print(f"  Logo-Quelle: {img.size}, Mode: {img.mode}")

    # Auf 512x243 fuer die App skalieren (oder Original behalten)
    logo_png = os.path.join(PUBLIC_DIR, "logo.png")
    # Originalgroesse beibehalten fuer maximale Qualitaet
    img.save(logo_png, "PNG")
    print(f"  Logo kopiert: logo.png ({img.size[0]}x{img.size[1]}) nach frontend/public/")

    # SVG-Logo kopieren
    if os.path.exists(SRC_LOGO_SVG):
        logo_svg = os.path.join(PUBLIC_DIR, "logo.svg")
        shutil.copy2(SRC_LOGO_SVG, logo_svg)
        print(f"  Logo kopiert: logo.svg nach frontend/public/")
    else:
        print(f"  Warnung: {SRC_LOGO_SVG} nicht gefunden!")

def main():
    print("=" * 60)
    print("Icon- und Logo-Generator fuer LMU Racecontrol")
    print("=" * 60)

    # --- LOGO kopieren ---
    print(f"\n--- LOGO (hell, 1024x486) ---")
    print(f"Quelle: {SRC_LOGO_PNG}")
    copy_logo()

    # --- ICONS generieren (quadratisch, 1024x1024) ---
    print(f"\n--- ICONS (transparent, 1024x1024) ---")
    print(f"Quelle: {SRC_ICON_PNG}")

    img = Image.open(SRC_ICON_PNG)
    print(f"  Icon-Quelle: {img.size}, Mode: {img.mode}")

    print(f"\nBereinige {ICONS_DIR} ...")
    clean_icons_dir()

    print(f"\nGeneriere PNG-Icons ...")
    generate_icon_pngs(img)

    print(f"\nGeneriere icon.ico ...")
    generate_ico(img)

    print(f"\nGeneriere icon.icns ...")
    generate_icns(img)

    print(f"\nFertig! Alle Icons und Logos wurden generiert.")
    print(f"  Icons: {ICONS_DIR}")
    print(f"  Logo:  {PUBLIC_DIR}/logo.png, logo.svg")

if __name__ == "__main__":
    main()