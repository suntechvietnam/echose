import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

// ES module compatibility
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Script to verify effect images exist for Vite build
const srcDir = path.join(__dirname, 'src', 'assets', 'img', 'effects');

console.log('Verifying effect images for build...');
console.log('Source directory:', srcDir);

try {
    if (fs.existsSync(srcDir)) {
        const files = fs.readdirSync(srcDir);
        const effectFiles = files.filter(file => 
            !file.startsWith('.') && 
            (file.endsWith('.png') || file.endsWith('.gif') || file.endsWith('.jpg') || file.endsWith('.jpeg'))
        );
        
        console.log(`✓ Found ${effectFiles.length} effect images:`);
        effectFiles.forEach(file => {
            const filePath = path.join(srcDir, file);
            const stats = fs.statSync(filePath);
            console.log(`  - ${file} (${stats.size} bytes)`);
        });
        
        const expectedFiles = [
            'none.png', 'circleopen.gif', 'diagbr.gif', 'diagtl.gif', 'diagtr.gif',
            'dissolve.gif', 'hlslice.gif', 'hlwind.gif', 'hrslice.gif', 'hrwind.gif',
            'radial.gif', 'vdslice.gif', 'vdwind.gif', 'vuslice.gif', 'vuwind.gif'
        ];
        
        const missingFiles = expectedFiles.filter(file => !effectFiles.includes(file));
        if (missingFiles.length > 0) {
            console.log(`\n⚠ Missing effect files:`);
            missingFiles.forEach(file => console.log(`  - ${file}`));
        } else {
            console.log('\n✓ All expected effect files are present');
        }
        
        console.log('\n✓ Effect images verification completed');
        console.log('Note: Vite will automatically bundle these imported assets');
    } else {
        console.log('✗ Effect images directory not found:', srcDir);
        console.log('Please ensure the directory exists with effect images');
        process.exit(1);
    }
} catch (error) {
    console.error('Error verifying effect images:', error.message);
    process.exit(1);
}