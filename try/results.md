# video-simple measurements

2026-11-16T14:36Z on sa-nb089, 6.18.57-1-lts, yoke 0.15.0, 15 s per run, /dev/video0.
`net.core.wmem_default` 212992, `wmem_max` 4194304.

| Path | Setting | Emitted | Arrived | Lost | Latency (avg / worst) | Core |
| --- | --- | --- | --- | ---: | --- | ---: |
| direct | mjpeg 640x480 @ 30 | 30.2 fps, 1.25 MB/s, 40 KB | 29.9 fps, 1.24 MB/s, 40 KB | 0 (0.0%) | 0.1 / 0.6 ms | 0.3% |
| direct | mjpeg 1280x720 @ 30 | 30.2 fps, 4.05 MB/s, 131 KB | 29.9 fps, 4.02 MB/s, 131 KB | 0 (0.0%) | 0.1 / 0.5 ms | 0.2% |
| direct | yuyv 320x240 @ 30 | 30.2 fps, 4.64 MB/s, 150 KB | 29.9 fps, 4.60 MB/s, 150 KB | 0 (0.0%) | 0.1 / 0.6 ms | 0.2% |
| direct | yuyv 424x240 @ 30 | 30.2 fps, 6.15 MB/s, 198 KB | 29.9 fps, 6.09 MB/s, 198 KB | 0 (0.0%) | 0.1 / 0.8 ms | 0.3% |
| direct | yuyv 640x360 @ 30 | 30.2 fps, 13.92 MB/s, 450 KB | 29.9 fps, 13.79 MB/s, 450 KB | 0 (0.0%) | 0.2 / 0.9 ms | 0.3% |
| direct | yuyv 640x480 @ 30 | 30.2 fps, 18.55 MB/s, 600 KB | 29.9 fps, 18.39 MB/s, 600 KB | 0 (0.0%) | 0.3 / 1.4 ms | 0.2% |
| direct | synthetic 1280x720 @ 30 | 30.0 fps, 55.29 MB/s, 1800 KB | 30.1 fps, 55.41 MB/s, 1800 KB | 0 (0.0%) | 0.6 / 2.8 ms | 0.3% |
| direct | synthetic 1920x1080 @ 30 | 30.0 fps, 124.41 MB/s, 4050 KB | 30.0 fps, 124.41 MB/s, 4050 KB | 0 (0.0%) | 1.3 / 6.3 ms | 0.3% |
| local | mjpeg 640x480 @ 30 | 30.2 fps, 1.25 MB/s, 40 KB | 29.7 fps, 1.23 MB/s, 40 KB | 0 (0.0%) | 0.4 / 1.1 ms | 1.7% |
| local | mjpeg 1280x720 @ 30 | 30.2 fps, 4.05 MB/s, 131 KB | 29.9 fps, 4.01 MB/s, 130 KB | 0 (0.0%) | 0.6 / 1.7 ms | 2.9% |
| local | yuyv 320x240 @ 30 | 30.2 fps, 4.64 MB/s, 150 KB | 29.9 fps, 4.60 MB/s, 150 KB | 0 (0.0%) | 0.6 / 1.3 ms | 3.1% |
| local | yuyv 424x240 @ 30 | 30.2 fps, 6.15 MB/s, 198 KB | 29.9 fps, 6.09 MB/s, 198 KB | 0 (0.0%) | 0.8 / 1.8 ms | 4.2% |
| local | yuyv 640x360 @ 30 | 30.2 fps, 13.92 MB/s, 450 KB | 29.9 fps, 13.79 MB/s, 450 KB | 0 (0.0%) | 1.3 / 2.3 ms | 6.9% |
| local | yuyv 640x480 @ 30 | 30.2 fps, 18.56 MB/s, 600 KB | 29.9 fps, 18.39 MB/s, 600 KB | 0 (0.0%) | 1.7 / 3.3 ms | 7.8% |
| local | synthetic 1280x720 @ 30 | 30.0 fps, 55.29 MB/s, 1800 KB | 30.0 fps, 55.29 MB/s, 1800 KB | 0 (0.0%) | 4.2 / 14.9 ms | 18.4% |
| local | synthetic 1920x1080 @ 30 | 30.0 fps, 124.41 MB/s, 4050 KB | 30.0 fps, 124.41 MB/s, 4050 KB | 0 (0.0%) | 5.9 / 19.1 ms | 21.0% |
| web | mjpeg 640x480 @ 30 | 30.2 fps, 1.25 MB/s, 40 KB | 29.7 fps, 1.23 MB/s, 40 KB | 0 (0.0%) | 0.7 / 1.4 ms | 1.5% |
| web | mjpeg 1280x720 @ 30 | 30.4 fps, 4.08 MB/s, 131 KB | 29.9 fps, 4.01 MB/s, 131 KB | 0 (0.0%) | 0.9 / 2.1 ms | 2.3% |
| web | yuyv 320x240 @ 30 | 30.4 fps, 4.67 MB/s, 150 KB | 29.9 fps, 4.59 MB/s, 150 KB | 0 (0.0%) | 1.0 / 2.0 ms | 2.5% |
| web | yuyv 424x240 @ 30 | 30.4 fps, 6.19 MB/s, 198 KB | 29.9 fps, 6.09 MB/s, 199 KB | 0 (0.0%) | 1.1 / 2.6 ms | 3.0% |
| web | yuyv 640x360 @ 30 | 30.4 fps, 14.01 MB/s, 450 KB | 29.9 fps, 13.79 MB/s, 450 KB | 0 (0.0%) | 1.6 / 4.3 ms | 5.0% |
| web | yuyv 640x480 @ 30 | 30.4 fps, 18.68 MB/s, 600 KB | 29.9 fps, 18.38 MB/s, 600 KB | 0 (0.0%) | 2.1 / 6.5 ms | 5.9% |
| web | synthetic 1280x720 @ 30 | 29.8 fps, 54.93 MB/s, 1800 KB | 30.0 fps, 55.27 MB/s, 1800 KB | 0 (0.0%) | 6.8 / 22.9 ms | 15.0% |
| web | synthetic 1920x1080 @ 30 | 30.0 fps, 124.41 MB/s, 4050 KB | 30.0 fps, 124.34 MB/s, 4050 KB | 0 (0.0%) | 30.4 / 114.1 ms | 18.5% |
