# The watch

| | |
| --- | --- |
| **Feature** | this Plugin against what Yoke last published |
| **Planning item** | yoke-project/yoke-plugin-video-simple#1 |

## yoke-plugin-video-simple:the-watch.01 — built against the published library and run under the newest Core, the Plugin's frames above 1 MiB reach a client on both local paths

| Field | Value |
| --- | --- |
| **Cites** | specs/95.40 |
| **Level** | L3 |
| **Method** | check |
| **Not applicable in** | — |
| **Label** | blocking |
| **Precondition** | the newest published programs of Yoke, and no camera |
| **Action** | build the Plugin and the recorder, run `try/` with the synthetic source at 1280x720 and 30 frames a second, and record five seconds reading the ring, then five on a stream socket |
| **Expected** | the instance comes up; each recording holds at least 100 frames, one of 1.8 MB among them; the first arrives on the shared object and the second on a stream socket |
