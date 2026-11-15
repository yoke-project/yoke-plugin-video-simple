// The page's measurement without a page: it attaches through the front exactly as the browser does, on
// the browser projection, and counts what arrives on the stream's binary WebSocket for a while.
//
//   node measure-web.mjs [--base http://127.0.0.1:8080] [--seconds 20] [--json FILE]
import { writeFileSync } from "node:fs";
import http from "node:http";
import { randomBytes } from "node:crypto";
import { Attachment } from "./public/attachment.js";

// What a browser does and Node's WebSocket does not: keep the cookie the upgrade's answer sets, and send
// it with every request. The channel ties a control request to its attachment by that cookie.
const jar = new Map();
function keep(headers) {
  for (const set of [headers["set-cookie"] ?? []].flat()) {
    const [pair] = set.split(";");
    const at = pair.indexOf("=");
    jar.set(pair.slice(0, at).trim(), pair.slice(at + 1).trim());
  }
}
const cookie = () => [...jar].map(([k, v]) => `${k}=${v}`).join("; ");
const fetchWithCookie = (url, init = {}) =>
  fetch(url, { ...init, headers: { ...(init.headers ?? {}), ...(jar.size ? { cookie: cookie() } : {}) } });

// A WebSocket client, enough for the browser projection: it opens with a cookie, keeps the one it is
// given, reads text and binary messages, and sends masked text.
class CookieWebSocket {
  constructor(url) {
    this.binaryType = "blob";
    this.onmessage = this.onclose = this.onerror = this.onopen = null;
    const key = randomBytes(16).toString("base64");
    const request = http.request(url.replace(/^ws/, "http"), {
      headers: { connection: "Upgrade", upgrade: "websocket", "sec-websocket-version": "13", "sec-websocket-key": key, ...(jar.size ? { cookie: cookie() } : {}) },
    });
    request.on("upgrade", (response, socket, head) => {
      keep(response.headers);
      this.socket = socket;
      this.buffer = head;
      this.parts = [];
      socket.on("data", (chunk) => this.read(chunk));
      socket.on("close", () => this.closed());
      this.onopen?.();
    });
    request.on("response", () => this.closed(1006));
    request.on("error", () => this.closed(1006));
    request.end();
  }

  read(chunk) {
    this.buffer = this.buffer.length ? Buffer.concat([this.buffer, chunk]) : chunk;
    for (;;) {
      const b = this.buffer;
      if (b.length < 2) return;
      let length = b[1] & 0x7f, at = 2;
      if (length === 126) { if (b.length < 4) return; length = b.readUInt16BE(2); at = 4; }
      else if (length === 127) { if (b.length < 10) return; length = Number(b.readBigUInt64BE(2)); at = 10; }
      if (b.length < at + length) return;
      const fin = b[0] & 0x80, op = b[0] & 0x0f, data = b.subarray(at, at + length);
      this.buffer = b.subarray(at + length);
      if (op === 8) { this.closed(data.length >= 2 ? data.readUInt16BE(0) : 1005); return; }
      if (op === 9) { this.frame(10, data); continue; }
      if (op === 1 || op === 2) this.kind = op;
      if (op === 0 || op === 1 || op === 2) {
        this.parts.push(data);
        if (fin) {
          const whole = Buffer.concat(this.parts);
          this.parts = [];
          const message = this.kind === 1 ? whole.toString("utf8") : whole.buffer.slice(whole.byteOffset, whole.byteOffset + whole.byteLength);
          this.onmessage?.({ data: message });
        }
      }
    }
  }

  frame(op, payload) {
    const mask = randomBytes(4);
    const n = payload.length;
    const head = n < 126 ? Buffer.from([0x80 | op, 0x80 | n]) : n < 65536 ? Buffer.from([0x80 | op, 0x80 | 126, n >> 8, n & 0xff]) : Buffer.concat([Buffer.from([0x80 | op, 0x80 | 127]), (() => { const l = Buffer.alloc(8); l.writeBigUInt64BE(BigInt(n)); return l; })()]);
    const body = Buffer.from(payload);
    for (let i = 0; i < body.length; i++) body[i] ^= mask[i % 4];
    this.socket?.write(Buffer.concat([head, mask, body]));
  }

  send(text) { this.frame(1, Buffer.from(String(text))); }

  close(code = 1000) {
    const c = Buffer.alloc(2);
    c.writeUInt16BE(code);
    this.frame(8, c);
    this.socket?.end();
  }

  closed(code = 1006) {
    if (this.done) return;
    this.done = true;
    this.socket?.destroy();
    this.onclose?.({ code, reason: "" });
  }
}
import { Tally, line, nowNs } from "./public/measure.js";

const UNIT = "video-simple";
const STREAM = "video.frames";

function argument(name, fallback) {
  const at = process.argv.indexOf(`--${name}`);
  return at > 0 ? process.argv[at + 1] : fallback;
}

const base = argument("base", "http://127.0.0.1:8080");
const seconds = Number(argument("seconds", "20"));
const json = argument("json", null);

const a = await Attachment.open({ base, fetch: fetchWithCookie, WebSocket: CookieWebSocket });
const format = new TextDecoder().decode(await a.query(UNIT, "video.format", new Uint8Array()));
console.error(`measure-web: ${UNIT} says ${format}`);
let window = new Tally();
const total = new Tally();
let last = null;
let first = null;
const delivery = await a.streamSubscribe(UNIT, STREAM, (frame) => {
  const arrived = nowNs();
  first ??= frame.sequence;
  window.add(frame, last, arrived);
  total.add(frame, last, arrived);
  last = frame.sequence;
});
if (!delivery.flowing) await a.streamStart(UNIT, STREAM);
console.error(`measure-web: subscribed to ${UNIT}/${STREAM} through ${base}, measuring for ${seconds} s`);
const report = setInterval(() => {
  console.error(`measure-web: ${line(window.summary())}`);
  window = new Tally();
}, 5000);
await new Promise((resolve) => setTimeout(resolve, seconds * 1000));
clearInterval(report);
const summary = { path: "browser projection", format: JSON.parse(format), firstSequence: Number(first ?? 0), ...total.summary() };
console.log(`measure-web: ${summary.seconds.toFixed(1)} s, ${summary.frames} frames from sequence ${summary.firstSequence}: ${line(summary)}`);
if (json) writeFileSync(json, JSON.stringify(summary, null, 2) + "\n");
await delivery.release().catch(() => {});
await a.streamStop(UNIT, STREAM).catch(() => {});
process.exit(0);
