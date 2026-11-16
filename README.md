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
| stream | `video.frames`, tolerating loss, its largest frame (`max_frame`) 4 MiB: each payload is one image as the camera gave it — a JPEG for `--format mjpeg`, packed YUYV 4:2:2 for `--format yuyv` |
| question | `video.format`: `{"schema":1,"format":…,"width":…,"height":…,"fps":…,"source":…}` |
| arguments | `--device`, `--format mjpeg\|yuyv`, `--source camera\|synthetic`, `--size WxH`, `--fps N` |

The camera is open only while the stream flows. With `--source synthetic` no camera is read: the Plugin
draws packed YUYV frames of the size and rate asked for, up to 1920x1080, so that a frame larger than a
camera offers can be carried and measured. It needs Yoke 0.15.0 or later, which is where a stream
declares its largest frame.

## Measuring

From `try/`, on a host with a camera and Yoke installed:

```sh
./measure.sh direct synthetic 1920x1080 30 20          # record, reading the stream's ring itself
./measure.sh local  mjpeg 1280x720 30 20 /tmp/v.mjpeg  # record, on a socket the Core copies every frame to
./measure.sh web    yuyv  640x480  30 20               # as a page receives it, through the browser projection
./measure-all.sh 15 results.md                          # every setting on the three paths, one table
./web/view.sh mjpeg 1280x720 30                         # the page, live, at http://127.0.0.1:8080/
```

Each run starts the application on a local Core, grants what the Plugin declares, measures, and stops
it. `results.md` is the table: what the Plugin emitted against what arrived, the frames lost, and the
latency — the producer's clock against the client's, which is meaningful because both run on one host —
and the share of one CPU the Core spent while the client ran.

A camera that lowers its rate in dim light (`exposure_dynamic_framerate`) measures the light rather than
Yoke; `measure-all.sh` turns it off for its runs and restores it.

`try/web/server.mjs` and `try/web/public/attachment.js` are copied unchanged from the reference web client
of `yoke-reference` (`clients/web`, at `e66774e`).

## The watch

`ci/watch.sh` builds the Plugin against the published library, runs `try/` under the newest published
programs with the synthetic source, and records the stream on both local paths. It runs every day and
whenever a release starts it.

## Licence

Apache-2.0.
