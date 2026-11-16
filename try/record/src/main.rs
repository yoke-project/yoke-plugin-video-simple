//! record: takes video-simple's frames through a channel and writes them to a file, saying what arrived.
//!
//!   record --instance I --channel C [--unit U] [--out FILE] [--seconds N] [--reads ring|socket]
//!   record [--unit U] [--out FILE] [--seconds N] [--reads ring|socket]   as a managed interface (YOKE_SOCKET)
//!
//! `--reads ring` (the default) offers the Core the stream's shared ring, read here directly, and the
//! stream socket where the stream is on no ring; `--reads socket` offers the stream socket alone, so the
//! Core copies every frame to it. It says which of the two the Core answered with.
//!
//! It asks the unit's `video.format`, subscribes to `video.frames`, asks for the stream to flow, and
//! writes every payload as it came: a `.mjpeg` plays with `ffplay -f mjpeg FILE`, a `.yuyv` with
//! `ffplay -f rawvideo -pixel_format yuyv422 -video_size WxH FILE`. Every five seconds, and at the end,
//! it says the rate, the gaps in the sequence (frames lost on the way) and the latency, the producer's
//! clock against this host's — one host, one clock.

use std::io::Write;
use std::path::Path;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use yoke_sdk::interface::{self, Attachment, Options, Reads};

const STREAM: &str = "video.frames";

struct Args {
    address: String,
    unit: String,
    out: Option<String>,
    seconds: u64,
    reads: Vec<Reads>,
}

fn args() -> Result<Args, String> {
    let (mut instance, mut channel, mut unit, mut out, mut seconds) = (None, None, "video-simple".to_string(), None, 20);
    let mut reads = vec![Reads::StreamSocket, Reads::SharedObject];
    let mut it = std::env::args().skip(1);
    while let Some(flag) = it.next() {
        let mut value = || it.next().ok_or(format!("{flag} takes a value"));
        match flag.as_str() {
            "--instance" => instance = Some(value()?),
            "--channel" => channel = Some(value()?),
            "--unit" => unit = value()?,
            "--out" => out = Some(value()?),
            "--seconds" => seconds = value()?.parse().map_err(|_| "--seconds takes a number".to_string())?,
            "--reads" => {
                reads = match value()?.as_str() {
                    "ring" => vec![Reads::StreamSocket, Reads::SharedObject],
                    "socket" => vec![Reads::StreamSocket],
                    other => return Err(format!("--reads is ring or socket, not {other}")),
                }
            }
            other => return Err(format!("unknown argument {other}")),
        }
    }
    let getenv = |k: &str| std::env::var(k).ok();
    let path = match (instance, channel) {
        (Some(i), Some(c)) => interface::application(&i, &c, getenv).map_err(|e| e.to_string())?,
        (None, None) => interface::managed(getenv).map_err(|e| e.to_string())?,
        _ => return Err("--instance and --channel go together".into()),
    };
    Ok(Args { address: path.to_string_lossy().into_owned(), unit, out, seconds, reads })
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(0)
}

/// What arrived over a span of time.
#[derive(Default, Clone)]
struct Tally {
    frames: u64,
    lost: u64,
    bytes: u64,
    largest: usize,
    latency_sum: u64,
    latency_max: u64,
}

impl Tally {
    fn line(&self, secs: f64) -> String {
        let n = self.frames.max(1);
        format!(
            "{:.1} fps, {:.2} MB/s, {} KB average, {} KB largest, {} lost ({:.1}%), latency {:.1} ms average, {:.1} ms worst",
            self.frames as f64 / secs,
            self.bytes as f64 / secs / 1e6,
            self.bytes / n / 1024,
            self.largest / 1024,
            self.lost,
            100.0 * self.lost as f64 / (self.frames + self.lost).max(1) as f64,
            self.latency_sum as f64 / n as f64 / 1e6,
            self.latency_max as f64 / 1e6,
        )
    }
}

#[tokio::main]
async fn main() {
    let args = match args() {
        Ok(a) => a,
        Err(err) => {
            eprintln!("record: {err}");
            std::process::exit(2);
        }
    };
    if let Err(err) = run(&args).await {
        eprintln!("record: {err}");
        std::process::exit(1);
    }
}

async fn run(args: &Args) -> Result<(), String> {
    let attachment = Attachment::attach(&args.address, Options::new()).await.map_err(|e| e.to_string())?;
    let format = attachment.ask(&args.unit, "video.format", b"").await.map_err(|e| format!("video.format: {e}"))?;
    let format = String::from_utf8_lossy(&format).into_owned();
    eprintln!("record: {} says {format}", args.unit);
    let mut file = match &args.out {
        Some(out) => {
            std::fs::write(Path::new(out).with_extension("json"), &format).map_err(|e| format!("{out}: {e}"))?;
            Some(std::io::BufWriter::new(std::fs::File::create(out).map_err(|e| format!("{out}: {e}"))?))
        }
        None => None,
    };
    // Subscribed first, so the first frame the stream carries is the first this copy receives.
    let mut delivery = attachment
        .subscribe_stream_reading(&args.unit, STREAM, &args.reads)
        .await
        .map_err(|e| format!("subscribe: {e}"))?;
    if !delivery.flowing {
        attachment.start_stream(&args.unit, STREAM).await.map_err(|e| format!("start: {e}"))?;
    }
    eprintln!(
        "record: subscribed to {}/{STREAM}, arriving on {}, recording for {} s",
        args.unit, delivery.arrives, args.seconds
    );

    let began = Instant::now();
    let deadline = tokio::time::Instant::now() + Duration::from_secs(args.seconds);
    let (mut total, mut window) = (Tally::default(), Tally::default());
    let mut window_began = Instant::now();
    let mut last: Option<u64> = None;
    let mut first_sequence = None;
    let mut ended = None;
    let mut report = tokio::time::interval(Duration::from_secs(5));
    report.tick().await;
    loop {
        tokio::select! {
            _ = tokio::time::sleep_until(deadline) => break,
            _ = report.tick() => {
                eprintln!("record: {}", window.line(window_began.elapsed().as_secs_f64()));
                (window, window_began) = (Tally::default(), Instant::now());
            }
            frame = delivery.next() => match frame {
                Ok(Some(f)) => {
                    let arrived = now();
                    first_sequence.get_or_insert(f.sequence);
                    let lost = match last { Some(l) if f.sequence > l + 1 => f.sequence - l - 1, _ => 0 };
                    last = Some(f.sequence);
                    let latency = arrived.saturating_sub(f.sent_at);
                    for t in [&mut total, &mut window] {
                        t.frames += 1;
                        t.lost += lost;
                        t.bytes += f.payload.len() as u64;
                        t.largest = t.largest.max(f.payload.len());
                        t.latency_sum += latency;
                        t.latency_max = t.latency_max.max(latency);
                    }
                    if let Some(file) = file.as_mut() {
                        file.write_all(&f.payload).map_err(|e| e.to_string())?;
                    }
                }
                Ok(None) => { ended = Some("the Core ended the delivery".to_string()); break; }
                Err(e) => { ended = Some(e.to_string()); break; }
            }
        }
    }
    let secs = began.elapsed().as_secs_f64();
    if let Some(file) = file.as_mut() {
        file.flush().map_err(|e| e.to_string())?;
    }
    if let Some(why) = &ended {
        eprintln!("record: stopped early after {secs:.1} s: {why}");
    }
    println!(
        "record: {:.1} s, {} frames from sequence {}: {}",
        secs,
        total.frames,
        first_sequence.unwrap_or(0),
        total.line(secs)
    );
    if ended.is_none() {
        let _ = delivery.release().await;
        let _ = attachment.stop_stream(&args.unit, STREAM).await;
    }
    Ok(())
}
