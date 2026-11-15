# video-simple measurements

2026-11-15T10:32Z on sa-nb089, 6.18.57-1-lts, yoke 0.13.0, 15 s per run, /dev/video0.
`net.core.wmem_default` 212992, `wmem_max` 4194304.

| Path | Setting | Emitted | Arrived | Lost | Latency (avg / worst) |
| --- | --- | --- | --- | ---: | --- |
| local | mjpeg 640x480 @ 30 | 30.4 fps, 1.26 MB/s, 40 KB | 29.8 fps, 1.23 MB/s, 40 KB | 0 (0.0%) | 0.4 / 0.9 ms |
| local | mjpeg 1280x720 @ 30 | 30.2 fps, 4.06 MB/s, 131 KB | 29.8 fps, 4.00 MB/s, 131 KB | 0 (0.0%) | 0.5 / 1.3 ms |
| local | yuyv 320x240 @ 30 | 30.2 fps, 4.64 MB/s, 150 KB | 29.8 fps, 4.58 MB/s, 150 KB | 0 (0.0%) | 0.5 / 2.1 ms |
| local | yuyv 424x240 @ 30 | 30.4 fps, 6.19 MB/s, 198 KB | 29.8 fps, 6.06 MB/s, 198 KB | 0 (0.0%) | 0.6 / 1.6 ms |
| local | yuyv 640x360 @ 30 | 30.4 fps, 14.01 MB/s, 450 KB | **nothing** | — | — |
| local | yuyv 640x480 @ 30 | 30.2 fps, 18.56 MB/s, 600 KB | **nothing** | — | — |
| web | mjpeg 640x480 @ 30 | 30.2 fps, 1.25 MB/s, 40 KB | 29.8 fps, 1.23 MB/s, 40 KB | 0 (0.0%) | 0.8 / 1.6 ms |
| web | mjpeg 1280x720 @ 30 | 30.2 fps, 4.06 MB/s, 131 KB | 29.8 fps, 4.00 MB/s, 131 KB | 0 (0.0%) | 1.0 / 2.1 ms |
| web | yuyv 320x240 @ 30 | 30.2 fps, 4.64 MB/s, 150 KB | 29.8 fps, 4.57 MB/s, 150 KB | 0 (0.0%) | 1.0 / 2.0 ms |
| web | yuyv 424x240 @ 30 | 30.2 fps, 6.15 MB/s, 198 KB | 29.8 fps, 6.06 MB/s, 199 KB | 0 (0.0%) | 1.2 / 4.7 ms |
| web | yuyv 640x360 @ 30 | 30.2 fps, 13.92 MB/s, 450 KB | 29.8 fps, 13.72 MB/s, 450 KB | 0 (0.0%) | 1.7 / 4.6 ms |
| web | yuyv 640x480 @ 30 | 30.2 fps, 18.56 MB/s, 600 KB | 29.8 fps, 18.30 MB/s, 600 KB | 0 (0.0%) | 2.1 / 5.9 ms |
