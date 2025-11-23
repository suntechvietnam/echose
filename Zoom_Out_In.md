### 720 Zoom Out
`ffmpeg -loop 1 -i 1.png -vf "scale=4000:-1,zoompan=z='if(eq(on,1),1.5,zoom-0.0033)':x='iw/2-(iw/zoom/2)':y='ih/2-(ih/zoom/2)':d=150:s=1280x720" -c:v libx264 -t 5 -r 30 -pix_fmt yuv420p 1.mp4`

### 1080 Zoom Out
`ffmpeg -loop 1 -i 1.png -vf "scale=4000:-1,zoompan=z='if(eq(on,1),1.5,zoom-0.0033)':x='iw/2-(iw/zoom/2)':y='ih/2-(ih/zoom/2)':d=150:s=1920x1080" -c:v libx264 -t 5 -r 30 -pix_fmt yuv420p 1.mp4`

### 2K Zoom Out
`ffmpeg -loop 1 -i 1.png -vf "scale=8000:-1,zoompan=z='if(eq(on,1),1.5,zoom-0.0033)':x='iw/2-(iw/zoom/2)':y='ih/2-(ih/zoom/2)':d=150:s=2560x1440" -c:v libx264 -t 5 -r 30 -pix_fmt yuv420p 1.mp4`

### 4K Zoom Out
`ffmpeg -loop 1 -i 1.png -vf "scale=8000:-1,zoompan=z='if(eq(on,1),1.5,zoom-0.0033)':x='iw/2-(iw/zoom/2)':y='ih/2-(ih/zoom/2)':d=150:s=3840x2160" -c:v libx264 -t 5 -r 30 -pix_fmt yuv420p 1.mp4`


### 720 Zoom In
`ffmpeg -loop 1 -i 1.png -vf "scale=4000:-1,zoompan=z='min(zoom+0.0033,1.5)':d=150:s=1280x720:x='iw/2-(iw/zoom/2)':y='ih/2-(ih/zoom/2)'" -c:v libx264 -t 5 -r 30 -pix_fmt yuv420p zoom_in_720.mp4`

### 1080 Zoom In
`ffmpeg -loop 1 -i 1.png -vf "scale=4000:-1,zoompan=z='min(zoom+0.0033,1.5)':d=150:s=1920x1080:x='iw/2-(iw/zoom/2)':y='ih/2-(ih/zoom/2)'" -c:v libx264 -t 5 -r 30 -pix_fmt yuv420p zoom_in_1080.mp4`

### 2K Zoom In
`ffmpeg -loop 1 -i 1.png -vf "scale=8000:-1,zoompan=z='min(zoom+0.0033,1.5)':d=150:s=2560x1440:x='iw/2-(iw/zoom/2)':y='ih/2-(ih/zoom/2)'" -c:v libx264 -t 5 -r 30 -pix_fmt yuv420p zoom_in_2k.mp4`

### 4K Zoom In
`ffmpeg -loop 1 -i 1.png -vf "scale=8000:-1,zoompan=z='min(zoom+0.0033,1.5)':d=150:s=3840x2160:x='iw/2-(iw/zoom/2)':y='ih/2-(ih/zoom/2)'" -c:v libx264 -t 5 -r 30 -pix_fmt yuv420p zoom_in_4k.mp4`
