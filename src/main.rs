mod app;
mod capture;
mod editor;
mod export;

use app::{InitialState, NiriShotApp};
use capture::{CaptureError, CaptureMode};
use clap::Parser;

#[derive(Parser)]
#[command(name = "niri-shot")]
#[command(about = "Screenshot tool for the niri Wayland compositor")]
#[command(version)]
struct Args {
    #[arg(short, long, help = "Capture the focused monitor")]
    fullscreen: bool,

    #[arg(short, long, help = "Capture a region (interactive selection)")]
    region: bool,

    #[arg(short, long, help = "Pick a window to capture")]
    window: bool,
}

fn main() {
    let args = Args::parse();

    let mode = if args.fullscreen {
        Some(CaptureMode::Fullscreen)
    } else if args.region {
        Some(CaptureMode::Region)
    } else if args.window {
        Some(CaptureMode::Window)
    } else {
        None
    };

    let initial = match mode {
        None => InitialState::default(),
        Some(mode) => match capture::capture(mode) {
            Ok(image) => {
                let error = match mode {
                    CaptureMode::Region => export::copy_png(&image)
                        .err()
                        .map(|e| format!("Copy failed: {}", e)),
                    CaptureMode::Fullscreen | CaptureMode::Window => None,
                };
                InitialState {
                    image: Some(image),
                    error,
                }
            }
            Err(CaptureError::Cancelled) => return,
            Err(e) => {
                eprintln!("Capture error: {}", e);
                InitialState {
                    image: None,
                    error: Some(format!("Capture failed: {}", e)),
                }
            }
        },
    };

    NiriShotApp::new(initial).run();
}
