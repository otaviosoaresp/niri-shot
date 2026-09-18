use serde::Deserialize;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::{env, fs, process};

use super::{spawn_error, CaptureError};

const SUBSCRIBE_TIMEOUT: Duration = Duration::from_secs(2);
const CAPTURE_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Deserialize)]
struct PickedWindow {
    id: u64,
}

#[derive(Deserialize)]
enum NiriEvent {
    ScreenshotCaptured { path: Option<String> },
}

fn parse_picked_window(output: &str) -> Result<Option<u64>, CaptureError> {
    serde_json::from_str::<Option<PickedWindow>>(output.trim())
        .map(|picked| picked.map(|window| window.id))
        .map_err(|e| CaptureError::Failed(format!("unexpected pick-window output: {}", e)))
}

fn parse_captured_path(line: &str) -> Option<Option<String>> {
    match serde_json::from_str::<NiriEvent>(line) {
        Ok(NiriEvent::ScreenshotCaptured { path }) => Some(path),
        Err(_) => None,
    }
}

pub(super) fn capture_window() -> Result<Vec<u8>, CaptureError> {
    ensure_session()?;
    let output = run_niri(&["--json", "pick-window"])?;
    let id = parse_picked_window(&output)?.ok_or(CaptureError::Cancelled)?;
    shoot(&["action", "screenshot-window", "--id", &id.to_string()])
}

pub(super) fn capture_screen() -> Result<Vec<u8>, CaptureError> {
    ensure_session()?;
    shoot(&["action", "screenshot-screen", "--show-pointer", "false"])
}

fn ensure_session() -> Result<(), CaptureError> {
    if env::var_os("NIRI_SOCKET").is_none() {
        return Err(CaptureError::Unavailable(
            "niri-shot needs a running niri session (NIRI_SOCKET is not set)".to_string(),
        ));
    }
    Ok(())
}

fn run_niri(args: &[&str]) -> Result<String, CaptureError> {
    let output = Command::new("niri")
        .arg("msg")
        .args(args)
        .output()
        .map_err(|e| spawn_error("niri", e))?;

    if !output.status.success() {
        return Err(CaptureError::Failed(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn shoot(action: &[&str]) -> Result<Vec<u8>, CaptureError> {
    let path = temp_capture_path();
    let path_str = path
        .to_str()
        .ok_or_else(|| CaptureError::Failed("temporary path is not valid UTF-8".to_string()))?
        .to_string();

    let events = EventStream::subscribe()?;

    let mut args = action.to_vec();
    args.extend(["--path", path_str.as_str()]);

    let result = run_niri(&args)
        .and_then(|_| events.wait_for_capture(&path_str))
        .and_then(|()| {
            fs::read(&path).map_err(|e| {
                CaptureError::Failed(format!("could not read {}: {}", path.display(), e))
            })
        });

    drop(events);
    let _ = fs::remove_file(&path);
    result
}

fn temp_capture_path() -> PathBuf {
    let dir = env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(env::temp_dir);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_nanos())
        .unwrap_or_default();
    dir.join(format!("niri-shot-{}-{}.png", process::id(), nanos))
}

struct EventStream {
    child: Child,
    lines: mpsc::Receiver<String>,
}

impl EventStream {
    fn subscribe() -> Result<Self, CaptureError> {
        let mut child = Command::new("niri")
            .args(["msg", "--json", "event-stream"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| spawn_error("niri", e))?;

        let stdout = child.stdout.take().expect("stdout is piped");
        let (sender, lines) = mpsc::channel();
        thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                if sender.send(line).is_err() {
                    break;
                }
            }
        });

        let stream = Self { child, lines };
        stream
            .lines
            .recv_timeout(SUBSCRIBE_TIMEOUT)
            .map_err(|_| CaptureError::Failed("could not subscribe to niri events".to_string()))?;
        Ok(stream)
    }

    fn wait_for_capture(&self, path: &str) -> Result<(), CaptureError> {
        let deadline = Instant::now() + CAPTURE_TIMEOUT;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            let line = self.lines.recv_timeout(remaining).map_err(|e| match e {
                RecvTimeoutError::Timeout => CaptureError::Failed(
                    "timed out waiting for niri to save the screenshot".to_string(),
                ),
                RecvTimeoutError::Disconnected => {
                    CaptureError::Failed("the niri event stream closed".to_string())
                }
            })?;

            if parse_captured_path(&line).flatten().as_deref() == Some(path) {
                return Ok(());
            }
        }
    }
}

impl Drop for EventStream {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_cancelled_pick_is_none() {
        assert_eq!(parse_picked_window("null\n").unwrap(), None);
    }

    #[test]
    fn a_picked_window_yields_its_id() {
        let output = r#"{"id":42,"title":"foot","app_id":"foot","pid":7,"is_focused":false}"#;
        assert_eq!(parse_picked_window(output).unwrap(), Some(42));
    }

    #[test]
    fn unexpected_pick_output_is_a_failure() {
        assert!(matches!(
            parse_picked_window("{}"),
            Err(CaptureError::Failed(_))
        ));
        assert!(matches!(
            parse_picked_window("No window selected."),
            Err(CaptureError::Failed(_))
        ));
    }

    #[test]
    fn a_captured_event_yields_its_path() {
        let line = r#"{"ScreenshotCaptured":{"path":"/run/user/1000/niri-shot-1-2.png"}}"#;
        assert_eq!(
            parse_captured_path(line),
            Some(Some("/run/user/1000/niri-shot-1-2.png".to_string()))
        );
    }

    #[test]
    fn a_captured_event_without_a_file_yields_no_path() {
        assert_eq!(
            parse_captured_path(r#"{"ScreenshotCaptured":{"path":null}}"#),
            Some(None)
        );
    }

    #[test]
    fn other_events_and_garbage_are_ignored() {
        assert_eq!(
            parse_captured_path(r#"{"WorkspacesChanged":{"workspaces":[]}}"#),
            None
        );
        assert_eq!(parse_captured_path(r#"{"SomeFutureEvent":{}}"#), None);
        assert_eq!(parse_captured_path("not json"), None);
    }

    #[test]
    fn capture_paths_are_absolute_pngs() {
        let path = temp_capture_path();
        assert!(path.is_absolute());
        assert_eq!(path.extension().and_then(|e| e.to_str()), Some("png"));
    }
}
