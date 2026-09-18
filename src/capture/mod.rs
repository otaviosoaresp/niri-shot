mod niri;
mod region;

use std::fmt;
use std::io;

#[derive(Debug, Clone, Copy)]
pub enum CaptureMode {
    Fullscreen,
    Region,
    Window,
}

#[derive(Debug)]
pub enum CaptureError {
    Cancelled,
    Unavailable(String),
    Failed(String),
}

impl fmt::Display for CaptureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cancelled => write!(f, "capture cancelled"),
            Self::Unavailable(message) | Self::Failed(message) => write!(f, "{}", message),
        }
    }
}

pub fn capture(mode: CaptureMode) -> Result<Vec<u8>, CaptureError> {
    match mode {
        CaptureMode::Fullscreen => niri::capture_screen(),
        CaptureMode::Region => region::capture(),
        CaptureMode::Window => niri::capture_window(),
    }
}

fn spawn_error(program: &str, err: io::Error) -> CaptureError {
    if err.kind() == io::ErrorKind::NotFound {
        CaptureError::Unavailable(format!("{} is not installed or not on PATH", program))
    } else {
        CaptureError::Failed(format!("could not run {}: {}", program, err))
    }
}
