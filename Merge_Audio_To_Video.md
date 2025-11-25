### Thời gian theo audio

ffmpeg -stream_loop -1 -i video.mp4 -i audio1.mp3 \
  -filter_complex "[0:v]setpts=N/FRAME_RATE/TB[v]" \
  -map "[v]" -map 1:a \
  -shortest -c:v libx264 -crf 23 -preset veryfast \
  -c:a aac -b:a 256k -movflags +faststart output.mp4


Cách 1: Loop video theo độ dài audio → không dư hình, không mất tiếng (KHÔNG re-encode)

Đây là cách tối ưu nhất.

1. Lấy thời lượng audio
ffprobe -v error -show_entries format=duration -of csv=p=0 audio1.mp3


Ví dụ audio dài 120 giây.

2. Loop video vừa đủ theo thời lượng audio

ffmpeg -stream_loop 100 -i video.mp4 -i audio1.mp3 \
  -t 120 \
  -c:v copy -c:a aac output.mp4
  

Kết luận cách này

ffmpeg -i audio1.mp3 -stream_loop 100 -i video.mp4 \
  -t $(ffprobe -v error -show_entries format=duration -of csv=p=0 audio1.mp3) \
  -c:v copy -c:a aac output.mp4


✔ Performance cực nhanh
✔ Không giảm chất lượng
✔ CPU load thấp nhất
✔ Cách tốt nhất để 'loop video → khớp audio' mà không dùng filter_complex
✔ Không gây lỗi mất tiếng cuối video

=> Đây là cách GHÉP NHẠC + LOOP VIDEO tốt nhất nếu bạn muốn performance tối đa.