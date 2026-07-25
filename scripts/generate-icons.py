from PIL import Image, ImageDraw
import os

path = r'c:\Users\Administrator\Documents\AI\Software Entwicklung\LMU Racecontrol\Tool\lmu-race-control\src-tauri\icons'
src = os.path.join(path, 'LMU RC - Icon.png')
img = Image.open(src).convert("RGBA")

# 10% Padding hinzufügen, damit Windows das Icon nicht abschneidet
padding = int(img.width * 0.10)  # 10%
new_size = img.width + 2 * padding

# Neue Leinwand mit Padding
canvas = Image.new("RGBA", (new_size, new_size), (0, 0, 0, 0))
canvas.paste(img, (padding, padding))

# PNGs generieren
sizes = {
    '32x32.png': 32,
    '64x64.png': 64,
    '128x128.png': 128,
    '128x128@2x.png': 256,
    'icon.png': 1024,
}
for name, size in sizes.items():
    resized = canvas.resize((size, size), Image.LANCZOS)
    resized.save(os.path.join(path, name), 'PNG')
    print(f'{name} -> {size}x{size} OK')

# ICO für Windows (mit mehreren Größen)
ico_sizes = [(16,16), (32,32), (48,48), (64,64), (128,128), (256,256)]
ico_imgs = [canvas.resize(s, Image.LANCZOS) for s in ico_sizes]
ico_imgs[0].save(
    os.path.join(path, 'icon.ico'),
    format='ICO',
    sizes=ico_sizes,
    append_images=ico_imgs[1:]
)
print('icon.ico -> 16x16..256x256 OK')

# icon-source.png aktualisieren
canvas.save(os.path.join(path, 'icon-source.png'), 'PNG')
print('icon-source.png aktualisiert')

print('\nALLE ICONS GENERIERT!')