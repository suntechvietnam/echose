### 720 Pan
ffmpeg -loop 1 -i 1.png -vf "zoompan=z=1.2:x='on/150*(iw-iw/zoom)':y='ih/2-(ih/zoom/2)':d=150:s=1280x720" -c:v libx264 -t 5 -r 30 -pix_fmt yuv420p output_pan.mp4

### 1080 Pan
ffmpeg -loop 1 -i 2.png -vf "zoompan=z=1.2:x='on/150*(iw-iw/zoom)':y='ih/2-(ih/zoom/2)':d=150:s=1920x1080" -c:v libx264 -t 5 -r 30 -pix_fmt yuv420p output_pan.mp4

### 2K
ffmpeg -loop 1 -i 3.png -vf "zoompan=z=1.2:x='on/150*(iw-iw/zoom)':y='ih/2-(ih/zoom/2)':d=150:s=2560x1440" -c:v libx264 -t 5 -r 30 -pix_fmt yuv420p output_pan.mp4

### 4K
ffmpeg -loop 1 -i 4.png -vf "zoompan=z=1.2:x='on/150*(iw-iw/zoom)':y='ih/2-(ih/zoom/2)':d=150:s=3840x2160" -c:v libx264 -t 5 -r 30 -pix_fmt yuv420p output_pan.mp4

