# video-simple measurements

2026-11-15T13:54Z on sa-nb089, 6.18.57-1-lts, yoke 0.14.1, 15 s per run, /dev/video0.
`net.core.wmem_default` 212992, `wmem_max` 4194304.

| Path | Setting | Emitted | Arrived | Lost | Latency (avg / worst) |
| --- | --- | --- | --- | ---: | --- |
| local | mjpeg 640x480 @ 30 | 30.2 fps, 1.25 MB/s, 40 KB | 29.9 fps, 1.24 MB/s, 40 KB | 0 (0.0%) | 0.4 / 0.9 ms |
| local | mjpeg 1280x720 @ 30 | 30.2 fps, 4.06 MB/s, 131 KB | 29.9 fps, 4.02 MB/s, 131 KB | 0 (0.0%) | 0.5 / 1.5 ms |
| local | yuyv 320x240 @ 30 | 30.2 fps, 4.64 MB/s, 150 KB | 29.9 fps, 4.60 MB/s, 150 KB | 0 (0.0%) | 0.5 / 1.4 ms |
| local | yuyv 424x240 @ 30 | 30.4 fps, 6.19 MB/s, 198 KB | 29.9 fps, 6.09 MB/s, 198 KB | 0 (0.0%) | 0.6 / 2.0 ms |
| local | yuyv 640x360 @ 30 | 30.2 fps, 13.92 MB/s, 450 KB | 29.9 fps, 13.79 MB/s, 450 KB | 0 (0.0%) | 0.9 / 2.1 ms |
| local | yuyv 640x480 @ 30 | 30.4 fps, 18.68 MB/s, 600 KB | 29.9 fps, 18.39 MB/s, 600 KB | 0 (0.0%) | 1.9 / 3.4 ms |
| web | mjpeg 640x480 @ 30 | 30.2 fps, 1.25 MB/s, 40 KB | 29.9 fps, 1.24 MB/s, 41 KB | 0 (0.0%) | 0.6 / 1.7 ms |
| web | mjpeg 1280x720 @ 30 | 30.2 fps, 4.06 MB/s, 131 KB | 29.9 fps, 4.02 MB/s, 131 KB | 0 (0.0%) | 0.9 / 2.0 ms |
| web | yuyv 320x240 @ 30 | 30.2 fps, 4.64 MB/s, 150 KB | 29.9 fps, 4.60 MB/s, 150 KB | 0 (0.0%) | 0.9 / 2.1 ms |
| web | yuyv 424x240 @ 30 | 30.2 fps, 6.15 MB/s, 198 KB | 29.9 fps, 6.09 MB/s, 199 KB | 0 (0.0%) | 1.1 / 2.2 ms |
| web | yuyv 640x360 @ 30 | 30.2 fps, 13.92 MB/s, 450 KB | 29.9 fps, 13.79 MB/s, 450 KB | 0 (0.0%) | 1.6 / 4.4 ms |
| web | yuyv 640x480 @ 30 | 30.2 fps, 18.56 MB/s, 600 KB | 29.9 fps, 18.38 MB/s, 600 KB | 0 (0.0%) | 2.3 / 6.5 ms |
