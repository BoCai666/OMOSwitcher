import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import toIco from 'to-ico';
import sharp from 'sharp';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// 路径
const iconsDir = path.join(__dirname, '..', 'src-tauri', 'icons');
const sourceImage = path.join(iconsDir, 'icon.png');

// ICO 需要的尺寸
const ICO_SIZES = [16, 32, 48, 64, 128, 256];

async function generateIcons() {
    console.log('Reading source image:', sourceImage);
    
    const buffers = [];
    
    for (const size of ICO_SIZES) {
        const buffer = await sharp(sourceImage)
            .resize(size, size, { kernel: 'lanczos3' })
            .png()
            .toBuffer();
        buffers.push(buffer);
        console.log(`  Generated ${size}x${size}`);
    }
    
    // 合并为 ICO
    const ico = await toIco(buffers);
    
    const outputPath = path.join(iconsDir, 'icon.ico');
    fs.writeFileSync(outputPath, ico);
    
    console.log(`\nSaved: ${outputPath}`);
    console.log(`Size: ${ico.length} bytes`);
}

generateIcons().catch(console.error);
