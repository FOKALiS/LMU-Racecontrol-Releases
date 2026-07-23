const sharp = require('sharp');
const path = require('path');
const fs = require('fs');

const backupPath = path.join(__dirname, '..', 'src-tauri', 'icons', 'icon-source-backup.png');
const outputPath = path.join(__dirname, '..', 'src-tauri', 'icons', 'icon-source.png');

async function cropIcon() {
  const metadata = await sharp(backupPath).metadata();
  console.log('Original size:', metadata.width, 'x', metadata.height);

  // Step 1: Trim transparent edges to find actual logo bounds
  const trimmedBuffer = await sharp(backupPath)
    .trim({ threshold: 10 })
    .toBuffer();

  const trimmedMeta = await sharp(trimmedBuffer).metadata();
  console.log('Trimmed logo size:', trimmedMeta.width, 'x', trimmedMeta.height);

  // Step 2: Add minimal padding (10px) around the trimmed logo
  const padding = 10;
  const canvasSize = Math.max(trimmedMeta.width, trimmedMeta.height) + padding * 2;

  // Step 3: Create a square canvas with padding, place logo centered
  const leftOffset = padding + Math.round((canvasSize - trimmedMeta.width) / 2);
  const topOffset = padding + Math.round((canvasSize - trimmedMeta.height) / 2);

  const paddedBuffer = await sharp({
    create: {
      width: canvasSize,
      height: canvasSize,
      channels: 4,
      background: { r: 0, g: 0, b: 0, alpha: 0 }
    }
  })
  .composite([{
    input: trimmedBuffer,
    top: topOffset,
    left: leftOffset,
  }])
  .png()
  .toBuffer();

  // Step 4: Resize to original dimensions (512x512)
  await sharp(paddedBuffer)
    .resize(metadata.width, metadata.height, {
      fit: 'fill',
      background: { r: 0, g: 0, b: 0, alpha: 0 }
    })
    .png()
    .toFile(outputPath);

  console.log('Saved cropped icon to', outputPath);
  
  const finalMeta = await sharp(outputPath).metadata();
  console.log('Final size:', finalMeta.width, 'x', finalMeta.height);
}

cropIcon().catch(err => {
  console.error('Error:', err);
  process.exit(1);
});