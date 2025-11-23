ffmpeg -loop 1 -i 1.png \
  -vf "scale=1920:1080:force_original_aspect_ratio=decrease,pad=1920:1080:(ow-iw)/2:(oh-ih)/2:black,\
       fade=t=in:st=0:d=1,colorchannelmixer=aa=1:aa_expr='if(between(T,4,5),1-(T-4),1)',\
       format=yuv420p" \
  -c:v libx264 -t 5 -r 30 -pix_fmt yuv420p -movflags +faststart output_fade_perfect.mp4
