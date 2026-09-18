use gtk4::prelude::*;
use gtk4::{gio, glib};
use gtk4::{
    ApplicationWindow, Box, Button, ColorButton, EventControllerKey, Label, Scale, ToggleButton,
};
use std::time::Duration;

use super::widgets::{enable_action_buttons, get_children, resize_window_to_image, show_status};
use crate::capture::{self, CaptureError, CaptureMode};
use crate::editor::{Color, EditorCanvas, ToolType};
use crate::export;

enum Export {
    Save,
    Copy,
}

const SETTLE_DELAY: Duration = Duration::from_millis(300);

pub fn connect_capture_buttons(
    capture_bar: &Box,
    canvas: &EditorCanvas,
    window: &ApplicationWindow,
    toolbar: &Box,
    status: &Label,
) {
    for btn in capture_buttons(capture_bar) {
        let capture_bar = capture_bar.clone();
        let canvas = canvas.clone();
        let window = window.clone();
        let toolbar = toolbar.clone();
        let status = status.clone();

        btn.connect_clicked(move |button| {
            let mode = match button.widget_name().as_str() {
                "btn_fullscreen" => CaptureMode::Fullscreen,
                "btn_region" => CaptureMode::Region,
                "btn_window" => CaptureMode::Window,
                _ => return,
            };

            set_capture_buttons_sensitive(&capture_bar, false);
            window.set_visible(false);

            let capture_bar = capture_bar.clone();
            let canvas = canvas.clone();
            let window = window.clone();
            let toolbar = toolbar.clone();
            let status = status.clone();

            glib::spawn_future_local(async move {
                let result = gio::spawn_blocking(move || {
                    std::thread::sleep(SETTLE_DELAY);
                    capture::capture(mode)
                })
                .await
                .unwrap_or_else(|_| {
                    Err(CaptureError::Failed("capture thread panicked".to_string()))
                });

                window.set_visible(true);
                set_capture_buttons_sensitive(&capture_bar, true);

                match result {
                    Ok(data) => load_capture(&canvas, &window, &toolbar, &status, &data),
                    Err(CaptureError::Cancelled) => show_status(&status, "", false),
                    Err(e) => {
                        eprintln!("Capture error: {}", e);
                        show_status(&status, &format!("Capture failed: {}", e), true);
                    }
                }
            });
        });
    }
}

fn capture_buttons(capture_bar: &Box) -> Vec<Button> {
    get_children(capture_bar)
        .into_iter()
        .filter_map(|w| w.downcast::<Button>().ok())
        .collect()
}

fn set_capture_buttons_sensitive(capture_bar: &Box, sensitive: bool) {
    for btn in capture_buttons(capture_bar) {
        btn.set_sensitive(sensitive);
    }
}

pub fn load_capture(
    canvas: &EditorCanvas,
    window: &ApplicationWindow,
    toolbar: &Box,
    status: &Label,
    data: &[u8],
) {
    match canvas.set_image(data) {
        Ok(()) => {
            enable_action_buttons(toolbar, true);
            resize_window_to_image(window, canvas);
            show_status(status, "", false);
        }
        Err(e) => show_status(status, &format!("Could not load capture: {}", e), true),
    }
}

pub fn connect_tool_buttons(toolbar: &Box, canvas: &EditorCanvas) {
    let children = get_children(toolbar);

    for widget in children {
        if let Ok(toggle) = widget.clone().downcast::<ToggleButton>() {
            let canvas = canvas.clone();

            toggle.connect_toggled(move |button| {
                if !button.is_active() {
                    return;
                }

                let tool = match button.widget_name().as_str() {
                    "tool_select" => ToolType::Select,
                    "tool_rectangle" => ToolType::Rectangle,
                    "tool_circle" => ToolType::Circle,
                    "tool_line" => ToolType::Line,
                    "tool_arrow" => ToolType::Arrow,
                    "tool_freehand" => ToolType::FreeHand,
                    "tool_text" => ToolType::Text,
                    "tool_blur" => ToolType::Blur,
                    "tool_highlight" => ToolType::Highlight,
                    _ => return,
                };

                canvas.set_tool_type(tool);
            });
        }

        if let Ok(color_btn) = widget.clone().downcast::<ColorButton>() {
            let canvas = canvas.clone();

            color_btn.connect_rgba_notify(move |button| {
                let rgba = button.rgba();
                let color = Color::new(
                    rgba.red() as f64,
                    rgba.green() as f64,
                    rgba.blue() as f64,
                    rgba.alpha() as f64,
                );
                canvas.set_color(color);
            });
        }

        if let Ok(scale) = widget.clone().downcast::<Scale>() {
            if scale.widget_name() == "stroke_width" {
                let canvas = canvas.clone();

                scale.connect_value_changed(move |s| {
                    canvas.set_stroke_width(s.value());
                });
            }
        }

        if let Ok(btn) = widget.downcast::<Button>() {
            let canvas = canvas.clone();

            btn.connect_clicked(move |button| match button.widget_name().as_str() {
                "btn_undo" => canvas.undo(),
                "btn_redo" => canvas.redo(),
                "btn_zoom_in" => canvas.zoom_in(),
                "btn_zoom_out" => canvas.zoom_out(),
                _ => {}
            });
        }
    }
}

pub fn connect_action_buttons(toolbar: &Box, canvas: &EditorCanvas, status: &Label) {
    let buttons: Vec<_> = get_children(toolbar)
        .into_iter()
        .filter_map(|w| w.downcast::<Button>().ok())
        .collect();

    for btn in buttons {
        let canvas = canvas.clone();
        let status = status.clone();

        btn.connect_clicked(move |button| match button.widget_name().as_str() {
            "btn_save" => export_image(&canvas, &status, Export::Save),
            "btn_copy" => export_image(&canvas, &status, Export::Copy),
            _ => {}
        });
    }
}

fn export_image(canvas: &EditorCanvas, status: &Label, what: Export) {
    let Some(data) = canvas.get_image_data() else {
        show_status(status, "Nothing to export", true);
        return;
    };

    match what {
        Export::Save => report_save(status, export::save_png(&data)),
        Export::Copy => report_copy(status, export::copy_png(&data)),
    }
}

fn report_save(status: &Label, result: anyhow::Result<std::path::PathBuf>) {
    match result {
        Ok(path) => {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| path.display().to_string());
            show_status(status, &format!("Saved {}", name), false);
        }
        Err(e) => {
            eprintln!("Save error: {}", e);
            show_status(status, &format!("Save failed: {}", e), true);
        }
    }
}

fn report_copy(status: &Label, result: anyhow::Result<()>) {
    match result {
        Ok(()) => show_status(status, "Copied to clipboard", false),
        Err(e) => {
            eprintln!("Copy error: {}", e);
            show_status(status, &format!("Copy failed: {}", e), true);
        }
    }
}

pub fn setup_keyboard_shortcuts(window: &ApplicationWindow, canvas: &EditorCanvas, status: &Label) {
    let key_controller = EventControllerKey::new();

    let canvas = canvas.clone();
    let status = status.clone();

    key_controller.connect_key_pressed(move |_, key, _, modifier| {
        let ctrl = modifier.contains(gtk4::gdk::ModifierType::CONTROL_MASK);

        if ctrl {
            match key {
                gtk4::gdk::Key::z => {
                    canvas.undo();
                    return glib::Propagation::Stop;
                }
                gtk4::gdk::Key::y => {
                    canvas.redo();
                    return glib::Propagation::Stop;
                }
                gtk4::gdk::Key::s => {
                    export_image(&canvas, &status, Export::Save);
                    return glib::Propagation::Stop;
                }
                gtk4::gdk::Key::c => {
                    export_image(&canvas, &status, Export::Copy);
                    return glib::Propagation::Stop;
                }
                gtk4::gdk::Key::plus | gtk4::gdk::Key::equal => {
                    canvas.zoom_in();
                    return glib::Propagation::Stop;
                }
                gtk4::gdk::Key::minus => {
                    canvas.zoom_out();
                    return glib::Propagation::Stop;
                }
                gtk4::gdk::Key::_0 => {
                    canvas.zoom_reset();
                    return glib::Propagation::Stop;
                }
                _ => {}
            }
        }

        glib::Propagation::Proceed
    });

    window.add_controller(key_controller);
}
