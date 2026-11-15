# com.yoke.video-simple

A Yoke Plugin written in Rust that publishes a camera's frames as a stream, and nothing else. What is done
with them is the application's: the `try/` application records them and shows them in a page, and
measures how they arrive on each of the ways a stream reaches a client.

| Path | What it holds |
| --- | --- |
| `yoke-plugin.yaml` | the file every step of `yoke-compose` reads: the identity, the build, the architectures |
| `plugin/` | the Plugin's source, and its Manifest, `com.yoke.video-simple.yaml`, kept beside it |
| `try/` | an application project that uses the Plugin: `record`, the page, and the measurement scripts |

## What it declares

| | |
| --- | --- |
| need | `device:camera`, a V4L2 capture device, handed as `--device ${bind.camera}` |
| stream | `video.frames`, tolerating loss: each payload is one image as the camera gave it — a JPEG for `--format mjpeg`, packed YUYV 4:2:2 for `--format yuyv` |
| question | `video.format`: `{"schema":1,"format":…,"width":…,"height":…,"fps":…}` |
| arguments | `--device`, `--format mjpeg\|yuyv`, `--size WxH`, `--fps N` |

The camera is open only while the stream flows.

## Measuring

From `try/`, on a host with a camera and Yoke installed:

```sh
./measure.sh local mjpeg 1280x720 30 20 /tmp/v.mjpeg   # record, on the viewer channel's local socket
./measure.sh web   yuyv  640x480  30 20                # as a page receives it, through the browser projection
./measure-all.sh 15 results.md                          # every setting on both paths, one table
./web/view.sh mjpeg 1280x720 30                         # the page, live, at http://127.0.0.1:8080/
```

Each run starts the application on a local Core, grants what the Plugin declares, measures, and stops
it. `results.md` is the table: what the Plugin emitted against what arrived, the frames lost, and the
latency — the producer's clock against the client's, which is meaningful because both run on one host.

A camera that lowers its rate in dim light (`exposure_dynamic_framerate`) measures the light rather than
Yoke; `measure-all.sh` turns it off for its runs and restores it.

`try/web/server.mjs` and `try/web/public/attachment.js` are copied unchanged from the reference web client
of `yoke-reference` (`clients/web`, at `e66774e`).

## Licence

Apache-2.0.
