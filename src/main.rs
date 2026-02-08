mod app;
mod capture;
mod config;
mod editor;

use anyhow::Result;
use capture::{CaptureBackend, CaptureMode};
use clap::Parser;
use std::io::Write;
use std::process::{Command, Stdio};

#[derive(Parser)]
#[command(name = "niri-shot")]
#[command(about = "Screenshot tool for Niri Wayland compositor")]
#[command(version)]
struct Args {
    #[arg(short, long, help = "Capture fullscreen")]
    fullscreen: bool,

    #[arg(short, long, help = "Capture region")]
    region: bool,

    #[arg(short, long, help = "Capture window")]
    window: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let initial_mode = if args.fullscreen {
        Some(CaptureMode::Fullscreen)
    } else if args.region {
        Some(CaptureMode::Region)
    } else if args.window {
        Some(CaptureMode::Window)
    } else {
        None
    };

    let initial_data = if let Some(mode) = initial_mode {
        match CaptureBackend::capture(mode) {
            Ok(data) => {
                copy_to_clipboard(&data);
                Some(data)
            }
            Err(e) => {
                eprintln!("Capture error: {}", e);
                return Ok(());
            }
        }
    } else {
        None
    };

    let app = app::NiriShotApp::new(initial_data);
    app.run();
    Ok(())
}

fn copy_to_clipboard(data: &[u8]) {
    let result = Command::new("wl-copy")
        .arg("--type")
        .arg("image/png")
        .stdin(Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(data)?;
            }
            child.wait()
        });

    if result.is_err() {
        eprintln!("Failed to copy to clipboard");
    }
}
