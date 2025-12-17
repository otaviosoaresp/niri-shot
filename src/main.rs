mod app;
mod capture;
mod config;
mod editor;

use anyhow::Result;

fn main() -> Result<()> {
    let app = app::NiriShotApp::new();
    app.run();
    Ok(())
}
