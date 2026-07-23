const sharp = require('sharp');
const path = require('path');

const sourcePath = path.join(__dirname, '..', 'src-tauri', 'icons', 'icon-source.png');

async function check() {
  const meta = await sharp(sourcePath).metadata();
  console.log('Size:', meta.width, 'x', meta.height);
  console.log('Format:', meta.format);
  console.log('Channels:', meta.channels);
  
  // Check if there are transparent pixels at edges
  const pixels = await sharp(sourcePath)
    .raw()
    .toBuffer();
  
  // Check top-left corner pixel
  console.log('Top-left pixel (RGBA):', pixels[0], pixels[1], pixels[2], pixels[3]);
  // Check center pixel
  const centerOffset = (Math.floor(meta.height/2) * meta.width + Math.floor(meta.width/2)) * 4;
  console.log('Center pixel (RGBA):', pixels[centerOffset], pixels[centerOffset+1], pixels[centerOffset+2], pixels[centerOffset+3]);
}

check().catch(err => {
  console.error('Error:', err);
  process.exit(1);
});