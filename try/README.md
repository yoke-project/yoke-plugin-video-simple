# Trying video-simple

This directory is an application project that uses the Plugin in `..`, and the instruments that measure
how its stream arrives. Nothing here is the Plugin, and taking the Plugin into another application never
takes this directory.

| Path | What it is |
| --- | --- |
| `yoke.yaml` | the application: the unit `video-simple` with the camera bound, the channel `viewer` (a local socket) and the channel `web` (`http+ws` on loopback, port 47911) |
| `record/` | an interface client: it subscribes on `viewer`, writes the frames to a file and counts what arrived |
| `web/` | a page drawing the frames, its front (`server.mjs`), and `measure-web.mjs`, which counts what a page receives |
| `lib.sh`, `measure.sh`, `measure-all.sh`, `web/view.sh` | run the application on a local Core and measure, or show the page |
| `results.md` | the last table `measure-all.sh` wrote |

The scripts build the Plugin and carry it into `components/` themselves. To take it by hand instead,
with a composer of 0.14.1 or later:

    yoke-compose add plugin .. --bind camera=/dev/video0
