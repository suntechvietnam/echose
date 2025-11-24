import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

// ES module compatibility
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Script to copy ffmpeg files with .exe extension for Windows
const resourcesDir = path.join(__dirname, 'src-tauri', 'resources');

console.log('Creating Windows-compatible ffmpeg files...');
console.log('Resources directory:', resourcesDir);

try {
    // Copy ffmpeg to ffmpeg.exe
    const ffmpegPath = path.join(resourcesDir, 'ffmpeg');
    const ffmpegExePath = path.join(resourcesDir, 'ffmpeg.exe');
    
    if (fs.existsSync(ffmpegPath)) {
        fs.copyFileSync(ffmpegPath, ffmpegExePath);
        console.log('✓ Created ffmpeg.exe');
    } else {
        console.log('✗ ffmpeg not found');
    }
    
    // Copy ffprobe to ffprobe.exe
    const ffprobePath = path.join(resourcesDir, 'ffprobe');
    const ffprobeExePath = path.join(resourcesDir, 'ffprobe.exe');
    
    if (fs.existsSync(ffprobePath)) {
        fs.copyFileSync(ffprobePath, ffprobeExePath);
        console.log('✓ Created ffprobe.exe');
    } else {
        console.log('✗ ffprobe not found');
    }
    
    // List all files
    console.log('\nFiles in resources directory:');
    const files = fs.readdirSync(resourcesDir);
    files.forEach(file => {
        const filePath = path.join(resourcesDir, file);
        const stats = fs.statSync(filePath);
        console.log(`  ${file} (${stats.size} bytes)`);
    });
    
    console.log('\n✓ FFmpeg preparation completed');
} catch (error) {
    console.error('Error:', error.message);
    process.exit(1);
}