// The page: attach, ask the format, subscribe to the frames and draw each one. A JPEG is decoded by the
// browser; a YUYV frame is converted here, which is slow at high sizes and is why mjpeg is the format for
// a page.
import { Attachment } from "./attachment.js";
import { Tally, line, nowNs } from "./measure.js";

const UNIT = "video-simple";
const STREAM = "video.frames";
const $ = (id) => document.getElementById(id);
const screen = $("screen");
const draw = screen.getContext("2d");

function say(text, bad = false) {
  $("state").textContent = text;
  $("state").classList.toggle("bad", bad);
}

function yuyvToImage(payload, width, height) {
  const image = draw.createImageData(width, height);
  const out = image.data;
  const clamp = (v) => (v < 0 ? 0 : v > 255 ? 255 : v);
  for (let i = 0, o = 0; i + 3 < payload.length; i += 4) {
    const y0 = payload[i], u = payload[i + 1] - 128, y1 = payload[i + 2], v = payload[i + 3] - 128;
    for (const y of [y0, y1]) {
      out[o++] = clamp(y + 1.402 * v);
      out[o++] = clamp(y - 0.344 * u - 0.714 * v);
      out[o++] = clamp(y + 1.772 * u);
      out[o++] = 255;
    }
  }
  return image;
}

async function main() {
  let a;
  try {
    a = await Attachment.open();
  } catch (error) {
    say(`not attached: ${error.message}`, true);
    return;
  }
  const format = JSON.parse(new TextDecoder().decode(await a.query(UNIT, "video.format", new Uint8Array())));
  $("format").textContent = `${format.format} ${format.width}×${format.height} at ${format.fps} fps`;
  screen.width = format.width;
  screen.height = format.height;

  let tally = new Tally();
  let total = new Tally();
  let last = null;
  let busy = false;
  const delivery = await a.streamSubscribe(UNIT, STREAM, async (frame) => {
    const arrived = nowNs();
    tally.add(frame, last, arrived);
    total.add(frame, last, arrived);
    last = frame.sequence;
    // A frame that arrives while the last is still being drawn is skipped here, not queued: the page
    // shows the newest, and the count above is of what arrived, not of what was drawn.
    if (busy) return;
    busy = true;
    try {
      if (format.format === "mjpeg") {
        const bitmap = await createImageBitmap(new Blob([frame.payload], { type: "image/jpeg" }));
        draw.drawImage(bitmap, 0, 0);
        bitmap.close();
      } else {
        draw.putImageData(yuyvToImage(frame.payload, format.width, format.height), 0, 0);
      }
    } catch {
      // A frame that does not decode is drawn as nothing; the count still has it.
    } finally {
      busy = false;
    }
  });
  if (!delivery.flowing) await a.streamStart(UNIT, STREAM);
  say("streaming");

  let flowing = true;
  $("toggle").disabled = false;
  $("toggle").onclick = async () => {
    flowing = !flowing;
    if (flowing) await a.streamStart(UNIT, STREAM);
    else await a.streamStop(UNIT, STREAM);
    $("toggle").textContent = flowing ? "Stop" : "Start";
    say(flowing ? "streaming" : "stopped");
  };
  setInterval(() => {
    $("stats").textContent = `last 2 s: ${line(tally.summary())} · since the start: ${total.frames} frames, ${total.lost} lost`;
    tally = new Tally();
  }, 2000);
  window.addEventListener("pagehide", () => delivery.release());
}

main();
