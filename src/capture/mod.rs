mod backend;
mod region;

use anyhow::Result;

#[derive(Debug, Clone, Copy)]
pub enum CaptureMode {
    Fullscreen,
    Region,
    Window,
}

pub fn capture(mode: CaptureMode) -> Result<Vec<u8>> {
    match mode {
        CaptureMode::Fullscreen => backend::capture_fullscreen(),
        CaptureMode::Region => region::capture(),
        CaptureMode::Window => backend::capture_window(),
    }
}
