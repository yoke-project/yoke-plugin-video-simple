//! video-simple: a Yoke Plugin that publishes a camera's frames as a stream, and nothing else. What is
//! done with them — shown, recorded, analysed — is the application's. Its Manifest is kept beside the
//! source, as com.yoke.video-simple.yaml, and is what `video-simple --manifest` prints; the continuous
//! integration of this project compares the two.
//!
//! - The camera is a `device` the application binds, handed as `--device ${bind.camera}`.
//! - `video.frames` tolerates loss, so the Core carries it on the shared ring: a frame late for its slot
//!   is a gap the consumer sees in the sequence, never a producer that waits.
//! - Each frame's payload is one image exactly as the camera gave it (a JPEG for `mjpeg`, packed YUYV
//!   4:2:2 for `yuyv`); the question `video.format` says which, and at what size and rate.
//! - The camera is open only while the stream flows: opened at its activation, closed when it stops.
//! - `--source synthetic` reads no camera: it emits packed YUYV frames of the size and rate asked for,
//!   drawn here, so that a frame larger than any the camera offers can be carried and measured.
//! - The stream declares its largest frame, 4 MiB: a raw 1920x1080 image is 4 147 200 bytes.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};

use tokio::signal::unix::{SignalKind, signal};
use tokio::sync::mpsc;
use yoke_sdk::plugin::{Capability, Declaration, Event, Object, Stream, start};

const FRAMES: &str = "video.frames";
const FORMAT: &str = "video.format";
/// The largest payload `video.frames` carries: room for a raw YUYV image of 1920x1080.
const MAX_FRAME: u64 = 4 * 1024 * 1024;

/// What this Plugin says about itself: a camera it needs, one stream, one question.
fn declaration() -> Declaration {
    Declaration {
        id: "com.yoke.video-simple".into(),
        needs: vec!["device:camera".into()],
        streams: vec![Stream { id: FRAMES.into(), tolerates_loss: true, tolerates_reorder: false, max_frame: MAX_FRAME }],
        queries: vec![FORMAT.into()],
        capabilities: vec![
            Capability { name: "stream.video.frames.publish".into(), governs: Object::Stream(FRAMES.into()) },
            Capability { name: "query.video.format.answer".into(), governs: Object::Query(FORMAT.into()) },
        ],
        ..Default::default()
    }
}

/// What the application asked for, from its arguments.
#[derive(Debug, Clone)]
struct Settings {
    device: String,
    /// `mjpeg` or `yuyv`.
    format: String,
    /// `camera`, or `synthetic`: frames drawn here, always `yuyv`.
    source: String,
    width: u32,
    height: u32,
    fps: u32,
}

impl Settings {
    fn from_args(args: impl Iterator<Item = String>) -> Result<Settings, String> {
        let mut s = Settings { device: "/dev/video0".into(), format: "mjpeg".into(), source: "camera".into(), width: 1280, height: 720, fps: 30 };
        let mut it = args;
        while let Some(flag) = it.next() {
            let mut value = || it.next().ok_or(format!("{flag} takes a value"));
            match flag.as_str() {
                "--device" => s.device = value()?,
                "--format" => s.format = value()?,
                "--source" => s.source = value()?,
                "--size" => {
                    let v = value()?;
                    let (w, h) = v.split_once('x').ok_or(format!("--size is WxH, not {v}"))?;
                    s.width = w.parse().map_err(|_| format!("--size: {w} is not a width"))?;
                    s.height = h.parse().map_err(|_| format!("--size: {h} is not a height"))?;
                }
                "--fps" => s.fps = value()?.parse().map_err(|_| "--fps takes a number".to_string())?,
                other => return Err(format!("unknown argument {other}")),
            }
        }
        if s.format != "mjpeg" && s.format != "yuyv" {
            return Err(format!("--format is mjpeg or yuyv, not {}", s.format));
        }
        if s.source != "camera" && s.source != "synthetic" {
            return Err(format!("--source is camera or synthetic, not {}", s.source));
        }
        if s.source == "synthetic" {
            s.format = "yuyv".into();
            if s.width == 0 || s.height == 0 || s.fps == 0 {
                return Err("--source synthetic takes a size and a rate above zero".into());
            }
            let size = 2 * s.width as u64 * s.height as u64;
            if size > MAX_FRAME {
                return Err(format!("a {}x{} frame is {size} bytes, above the {MAX_FRAME} the stream declares", s.width, s.height));
            }
        }
        Ok(s)
    }

    fn fourcc(&self) -> &'static [u8] {
        if self.format == "yuyv" { b"YUYV" } else { b"MJPG" }
    }

    /// The answer to `video.format`.
    fn json(&self) -> String {
        format!(
            "{{\"schema\":1,\"format\":\"{}\",\"width\":{},\"height\":{},\"fps\":{},\"source\":\"{}\"}}",
            self.format, self.width, self.height, self.fps, self.source
        )
    }
}

/// The capture thread's handle: dropped, it tells the thread to stop, and the thread closes the camera.
struct Capture {
    stop: Arc<AtomicBool>,
}

impl Drop for Capture {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

/// What the capture thread hands the Session's loop.
enum Captured {
    Frame(Vec<u8>),
    Failed(String),
}

/// Opens the camera and reads it on a thread of its own, handing each frame over. A frame the loop has
/// no room for is counted as dropped here rather than queued, since the stream tolerates loss.
fn capture(s: &Settings, tx: mpsc::Sender<Captured>, dropped: Arc<AtomicU64>) -> Capture {
    let stop = Arc::new(AtomicBool::new(false));
    let (s, halt) = (s.clone(), stop.clone());
    std::thread::spawn(move || {
        if s.source == "synthetic" {
            return synthesise(&s, &tx, &halt, &dropped);
        }
        let mut camera = match rscam::Camera::new(&s.device) {
            Ok(c) => c,
            Err(e) => {
                let _ = tx.blocking_send(Captured::Failed(format!("{} cannot be opened: {e}", s.device)));
                return;
            }
        };
        let config = rscam::Config {
            interval: (1, s.fps),
            resolution: (s.width, s.height),
            format: s.fourcc(),
            nbuffers: 4,
            ..Default::default()
        };
        if let Err(e) = camera.start(&config) {
            let _ = tx.blocking_send(Captured::Failed(format!(
                "{} refuses {} {}x{} at {} fps: {e}",
                s.device, s.format, s.width, s.height, s.fps
            )));
            return;
        }
        while !halt.load(Ordering::Relaxed) {
            match camera.capture() {
                Ok(frame) => {
                    if let Err(mpsc::error::TrySendError::Full(_)) = tx.try_send(Captured::Frame(frame.to_vec())) {
                        dropped.fetch_add(1, Ordering::Relaxed);
                    }
                }
                Err(e) => {
                    let _ = tx.blocking_send(Captured::Failed(format!("{} stopped giving frames: {e}", s.device)));
                    return;
                }
            }
        }
    });
    Capture { stop }
}

/// Draws packed YUYV frames at the rate asked for: a grey field with a bar that moves one column pair a
/// frame, so that two frames differ and a viewer sees it move.
fn synthesise(s: &Settings, tx: &mpsc::Sender<Captured>, halt: &AtomicBool, dropped: &AtomicU64) {
    let (width, height) = (s.width as usize, s.height as usize);
    let period = Duration::from_secs_f64(1.0 / s.fps as f64);
    let mut due = Instant::now();
    let mut n = 0usize;
    while !halt.load(Ordering::Relaxed) {
        let mut frame = vec![0x80u8; 2 * width * height];
        let bar = (2 * n) % width.max(1);
        for row in frame.chunks_exact_mut(2 * width) {
            for x in bar..(bar + 16).min(width) {
                row[2 * x] = 0xeb;
            }
        }
        if let Err(mpsc::error::TrySendError::Full(_)) = tx.try_send(Captured::Frame(frame)) {
            dropped.fetch_add(1, Ordering::Relaxed);
        }
        n += 1;
        due += period;
        match due.checked_duration_since(Instant::now()) {
            Some(wait) => std::thread::sleep(wait),
            None => due = Instant::now(),
        }
    }
}

/// What the unit has done since it last said so, for its log.
#[derive(Default)]
struct Window {
    frames: u64,
    bytes: u64,
    largest: usize,
    refused: u64,
}

#[tokio::main]
async fn main() {
    if std::env::args().nth(1).as_deref() == Some("--manifest") {
        print!("{}", declaration().manifest());
        return;
    }
    let settings = match Settings::from_args(std::env::args().skip(1)) {
        Ok(s) => s,
        Err(err) => {
            eprintln!("video-simple: {err}");
            std::process::exit(2);
        }
    };
    let unit = match start(&declaration()).await {
        Ok(unit) => unit,
        Err(err) => {
            eprintln!("video-simple: {err}");
            std::process::exit(1);
        }
    };
    let idle = format!(
        "ready: {} {}x{} at {} fps from {}",
        settings.format,
        settings.width,
        settings.height,
        settings.fps,
        if settings.source == "synthetic" { "a synthetic source" } else { &settings.device }
    );
    let _ = unit.health(0, &idle).await;
    let (tx, mut rx) = mpsc::channel::<Captured>(4);
    let dropped = Arc::new(AtomicU64::new(0));
    let mut flowing: Option<Capture> = None;
    let mut window = Window::default();
    let mut since = Instant::now();
    let mut tick = tokio::time::interval(Duration::from_secs(5));
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let mut term = signal(SignalKind::terminate()).expect("video-simple listens for SIGTERM");
    loop {
        tokio::select! {
            event = unit.next() => match event {
                None | Some(Event::Ended(_)) => return,
                Some(Event::Activated(a)) if a.stream == FRAMES => {
                    eprintln!("video-simple: {FRAMES} activated on {} at {}", a.transport, a.address);
                    flowing = Some(capture(&settings, tx.clone(), dropped.clone()));
                    (window, since) = (Window::default(), Instant::now());
                    let _ = unit.health(0, &format!("streaming {} {}x{}", settings.format, settings.width, settings.height)).await;
                }
                Some(Event::Stopped { stream }) if stream == FRAMES => {
                    flowing = None;
                    eprintln!("video-simple: {FRAMES} stopped");
                    let _ = unit.health(0, &idle).await;
                }
                Some(Event::Question(q)) if q.r#type == FORMAT => {
                    if let Err(err) = unit.answer(&q, settings.json().into_bytes()).await {
                        eprintln!("video-simple: {err}");
                    }
                }
                Some(_) => {}
            },
            Some(captured) = rx.recv() => match captured {
                // A frame that arrives after the stream stopped belongs to no activation, and goes.
                Captured::Frame(_) if flowing.is_none() => {}
                Captured::Frame(payload) => {
                    let size = payload.len();
                    match unit.emit(FRAMES, payload).await {
                        Ok(()) => {
                            window.frames += 1;
                            window.bytes += size as u64;
                            window.largest = window.largest.max(size);
                        }
                        Err(err) => {
                            window.refused += 1;
                            if window.refused == 1 {
                                eprintln!("video-simple: a frame of {size} bytes was refused: {err}");
                            }
                        }
                    }
                }
                Captured::Failed(line) => {
                    flowing = None;
                    eprintln!("video-simple: {line}");
                    let _ = unit.health(50, &line).await;
                }
            },
            _ = tick.tick() => {
                if flowing.is_some() {
                    let secs = since.elapsed().as_secs_f64();
                    eprintln!(
                        "video-simple: {:.1} fps, {:.2} MB/s, {} KB average, {} KB largest, {} refused, {} dropped at capture",
                        window.frames as f64 / secs,
                        window.bytes as f64 / secs / 1e6,
                        window.bytes / window.frames.max(1) / 1024,
                        window.largest / 1024,
                        window.refused,
                        dropped.swap(0, Ordering::Relaxed),
                    );
                    (window, since) = (Window::default(), Instant::now());
                }
            },
            _ = term.recv() => {
                drop(flowing.take());
                let _ = unit.close().await;
                while let Some(event) = unit.next().await {
                    if matches!(event, Event::Ended(_)) {
                        break;
                    }
                }
                return;
            }
        }
    }
}
