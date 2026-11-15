// What arrived over a span of time, counted the same way in the page and in measure-web.mjs: frames, the
// gaps in the sequence (frames lost on the way), bytes, and the latency — the producer's clock against
// this one, which means something only when both run on one host.
export class Tally {
  constructor() {
    this.frames = 0;
    this.lost = 0;
    this.bytes = 0;
    this.largest = 0;
    this.latencySum = 0;
    this.latencyMax = 0;
    this.began = performance.now();
  }

  add(frame, last, arrivedNs) {
    const lost = last !== null && frame.sequence > last + 1n ? Number(frame.sequence - last - 1n) : 0;
    const latency = Math.max(0, Number(arrivedNs - frame.sentAt)) / 1e6;
    this.frames += 1;
    this.lost += lost;
    this.bytes += frame.payload.byteLength;
    this.largest = Math.max(this.largest, frame.payload.byteLength);
    this.latencySum += latency;
    this.latencyMax = Math.max(this.latencyMax, latency);
  }

  summary() {
    const secs = (performance.now() - this.began) / 1000;
    const n = Math.max(this.frames, 1);
    return {
      seconds: secs,
      frames: this.frames,
      fps: this.frames / secs,
      mbps: this.bytes / secs / 1e6,
      averageKB: Math.round(this.bytes / n / 1024),
      largestKB: Math.round(this.largest / 1024),
      lost: this.lost,
      lostPercent: (100 * this.lost) / Math.max(this.frames + this.lost, 1),
      latencyMs: this.latencySum / n,
      latencyWorstMs: this.latencyMax,
    };
  }
}

export function line(s) {
  return `${s.fps.toFixed(1)} fps, ${s.mbps.toFixed(2)} MB/s, ${s.averageKB} KB average, ${s.largestKB} KB largest, ` +
    `${s.lost} lost (${s.lostPercent.toFixed(1)}%), latency ${s.latencyMs.toFixed(1)} ms average, ${s.latencyWorstMs.toFixed(1)} ms worst`;
}

// The wall clock in nanoseconds, as the producer stamps its frames.
export function nowNs() {
  return BigInt(Math.round((performance.timeOrigin + performance.now()) * 1e6));
}
