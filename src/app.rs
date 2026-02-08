use gtk4::gdk::RGBA;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box, Button, ColorButton, CssProvider, EventControllerKey,
    Orientation, Overlay, Scale, ScrolledWindow, Separator, ToggleButton,
};
use std::cell::RefCell;
use std::rc::Rc;

use crate::capture::{CaptureBackend, CaptureMode};
use crate::editor::{Color, EditorCanvas, ToolType};

const APP_ID: &str = "com.github.niri-shot";

pub struct NiriShotApp {
    app: Application,
}

impl NiriShotApp {
    pub fn new(initial_data: Option<Vec<u8>>) -> Self {
        let app = Application::builder()
            .application_id(APP_ID)
            .flags(gtk4::gio::ApplicationFlags::NON_UNIQUE)
            .build();

        let initial_data = Rc::new(RefCell::new(initial_data));

        app.connect_activate(move |app| {
            Self::build_ui(app, initial_data.clone());
        });

        Self { app }
    }

    pub fn run(&self) {
        self.app.run_with_args::<String>(&[]);
    }

    fn load_css() {
        let provider = CssProvider::new();
        provider.load_from_data(
            r#"
            .floating-toolbar {
                background: rgba(40, 40, 40, 0.95);
                border-radius: 12px;
                padding: 8px 16px;
            }

            .floating-toolbar button,
            .floating-toolbar togglebutton {
                min-width: 32px;
                min-height: 32px;
                padding: 4px 8px;
                border-radius: 6px;
            }

            .floating-toolbar togglebutton:checked {
                background: rgba(100, 150, 255, 0.4);
            }

            .nerd-icon {
                font-family: "JetBrainsMono Nerd Font", "Hack Nerd Font", monospace;
                font-size: 16px;
            }

            .capture-bar {
                background: rgba(50, 50, 50, 0.9);
                padding: 8px 16px;
            }

            .capture-bar button {
                padding: 6px 12px;
            }
            "#,
        );

        gtk4::style_context_add_provider_for_display(
            &gtk4::gdk::Display::default().expect("Could not get display"),
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    fn build_ui(app: &Application, initial_data: Rc<RefCell<Option<Vec<u8>>>>) {
        Self::load_css();

        let window = ApplicationWindow::builder()
            .application(app)
            .title("niri-shot")
            .default_width(1000)
            .default_height(700)
            .build();

        let main_box = Box::new(Orientation::Vertical, 0);

        let capture_bar = Self::create_capture_bar();
        capture_bar.add_css_class("capture-bar");

        let canvas = EditorCanvas::new();
        canvas.set_hexpand(true);
        canvas.set_vexpand(true);

        let scrolled = ScrolledWindow::new();
        scrolled.set_child(Some(&canvas));
        scrolled.set_vexpand(true);
        scrolled.set_hexpand(true);

        let overlay = Overlay::new();
        overlay.set_child(Some(&scrolled));

        let floating_toolbar = Self::create_floating_toolbar();
        floating_toolbar.add_css_class("floating-toolbar");
        floating_toolbar.set_halign(gtk4::Align::Center);
        floating_toolbar.set_valign(gtk4::Align::End);
        floating_toolbar.set_margin_bottom(20);

        overlay.add_overlay(&floating_toolbar);

        main_box.append(&capture_bar);
        main_box.append(&Separator::new(Orientation::Horizontal));
        main_box.append(&overlay);

        Self::connect_capture_buttons(&capture_bar, &canvas, &window, &floating_toolbar);
        Self::connect_tool_buttons(&floating_toolbar, &canvas);
        Self::connect_action_buttons(&floating_toolbar, &canvas);
        Self::setup_keyboard_shortcuts(&window, &canvas);

        window.set_child(Some(&main_box));

        let data = initial_data.borrow_mut().take();
        window.present();

        if let Some(image_data) = data {
            canvas.set_image(&image_data);
            Self::enable_action_buttons(&floating_toolbar, true);
            Self::resize_window_to_image(&window, &canvas);
        }
    }

    fn create_capture_bar() -> Box {
        let bar = Box::new(Orientation::Horizontal, 8);
        bar.set_margin_top(8);
        bar.set_margin_bottom(8);
        bar.set_margin_start(16);
        bar.set_margin_end(16);

        let btn_fullscreen = Button::from_icon_name("view-fullscreen-symbolic");
        btn_fullscreen.set_widget_name("btn_fullscreen");
        btn_fullscreen.set_tooltip_text(Some("Fullscreen"));

        let btn_region = Button::from_icon_name("edit-select-all-symbolic");
        btn_region.set_widget_name("btn_region");
        btn_region.set_tooltip_text(Some("Region"));

        let btn_window = Button::from_icon_name("window-symbolic");
        btn_window.set_widget_name("btn_window");
        btn_window.set_tooltip_text(Some("Window"));

        bar.append(&btn_fullscreen);
        bar.append(&btn_region);
        bar.append(&btn_window);

        bar
    }

    fn create_floating_toolbar() -> Box {
        let bar = Box::new(Orientation::Horizontal, 6);

        let btn_select = Self::create_nerd_button("󰍽", "tool_select", "Select");
        btn_select.set_active(true);

        let btn_rect = Self::create_nerd_button("□", "tool_rectangle", "Rectangle");
        btn_rect.set_group(Some(&btn_select));

        let btn_circle = Self::create_nerd_button("○", "tool_circle", "Circle");
        btn_circle.set_group(Some(&btn_select));

        let btn_line = Self::create_nerd_button("╱", "tool_line", "Line");
        btn_line.set_group(Some(&btn_select));

        let btn_arrow = Self::create_nerd_button("󰁕", "tool_arrow", "Arrow");
        btn_arrow.set_group(Some(&btn_select));

        let btn_freehand = Self::create_nerd_button("󰏬", "tool_freehand", "Freehand");
        btn_freehand.set_group(Some(&btn_select));

        let btn_text = Self::create_nerd_button("󰊄", "tool_text", "Text");
        btn_text.set_group(Some(&btn_select));

        let btn_blur = Self::create_nerd_button("󰂵", "tool_blur", "Blur");
        btn_blur.set_group(Some(&btn_select));

        let btn_highlight = Self::create_nerd_button("󰸱", "tool_highlight", "Highlight");
        btn_highlight.set_group(Some(&btn_select));

        let color_btn = ColorButton::with_rgba(&RGBA::new(1.0, 0.0, 0.0, 1.0));
        color_btn.set_widget_name("color_picker");
        color_btn.set_tooltip_text(Some("Color"));

        let stroke_scale = Scale::with_range(Orientation::Horizontal, 1.0, 20.0, 1.0);
        stroke_scale.set_value(3.0);
        stroke_scale.set_width_request(80);
        stroke_scale.set_widget_name("stroke_width");
        stroke_scale.set_tooltip_text(Some("Stroke Width"));

        let btn_zoom_out = Self::create_nerd_action_button("󰍴", "btn_zoom_out", "Zoom - (Ctrl+-)");
        let btn_zoom_in = Self::create_nerd_action_button("󰍷", "btn_zoom_in", "Zoom + (Ctrl++)");

        let btn_undo = Self::create_nerd_action_button("󰕌", "btn_undo", "Undo (Ctrl+Z)");
        let btn_redo = Self::create_nerd_action_button("󰑎", "btn_redo", "Redo (Ctrl+Y)");

        let btn_save = Self::create_nerd_action_button("󰆓", "btn_save", "Save (Ctrl+S)");
        btn_save.set_sensitive(false);

        let btn_copy = Self::create_nerd_action_button("󰆏", "btn_copy", "Copy (Ctrl+C)");
        btn_copy.set_sensitive(false);

        bar.append(&btn_select);
        bar.append(&btn_rect);
        bar.append(&btn_circle);
        bar.append(&btn_line);
        bar.append(&btn_arrow);
        bar.append(&btn_freehand);
        bar.append(&btn_text);
        bar.append(&btn_blur);
        bar.append(&btn_highlight);
        bar.append(&Separator::new(Orientation::Vertical));
        bar.append(&color_btn);
        bar.append(&stroke_scale);
        bar.append(&Separator::new(Orientation::Vertical));
        bar.append(&btn_zoom_out);
        bar.append(&btn_zoom_in);
        bar.append(&Separator::new(Orientation::Vertical));
        bar.append(&btn_undo);
        bar.append(&btn_redo);
        bar.append(&Separator::new(Orientation::Vertical));
        bar.append(&btn_save);
        bar.append(&btn_copy);

        bar
    }

    fn create_nerd_button(icon: &str, widget_name: &str, tooltip: &str) -> ToggleButton {
        let btn = ToggleButton::with_label(icon);
        btn.set_widget_name(widget_name);
        btn.set_tooltip_text(Some(tooltip));
        btn.add_css_class("nerd-icon");
        btn
    }

    fn create_nerd_action_button(icon: &str, widget_name: &str, tooltip: &str) -> Button {
        let btn = Button::with_label(icon);
        btn.set_widget_name(widget_name);
        btn.set_tooltip_text(Some(tooltip));
        btn.add_css_class("nerd-icon");
        btn
    }

    fn connect_capture_buttons(
        capture_bar: &Box,
        canvas: &EditorCanvas,
        window: &ApplicationWindow,
        toolbar: &Box,
    ) {
        let buttons: Vec<_> = Self::get_children(capture_bar)
            .into_iter()
            .filter_map(|w| w.downcast::<Button>().ok())
            .collect();

        for btn in buttons {
            let canvas = canvas.clone();
            let window = window.clone();
            let toolbar = toolbar.clone();

            btn.connect_clicked(move |button| {
                let mode = match button.widget_name().as_str() {
                    "btn_fullscreen" => CaptureMode::Fullscreen,
                    "btn_region" => CaptureMode::Region,
                    "btn_window" => CaptureMode::Window,
                    _ => return,
                };

                window.set_visible(false);

                while glib::MainContext::default().iteration(false) {}
                std::thread::sleep(std::time::Duration::from_millis(150));

                let result = CaptureBackend::capture(mode);

                window.set_visible(true);

                match result {
                    Ok(data) => {
                        canvas.set_image(&data);
                        Self::enable_action_buttons(&toolbar, true);
                        Self::resize_window_to_image(&window, &canvas);
                    }
                    Err(e) => eprintln!("Capture error: {}", e),
                }
            });
        }
    }

    fn resize_window_to_image(window: &ApplicationWindow, canvas: &EditorCanvas) {
        let img_width = canvas.content_width();
        let img_height = canvas.content_height();

        if img_width <= 0 || img_height <= 0 {
            return;
        }

        let toolbar_height = 100;
        let max_width = 1600;
        let max_height = 1000;

        let width = (img_width + 40).min(max_width);
        let height = (img_height + toolbar_height).min(max_height);

        window.set_default_size(width, height);
    }

    fn connect_tool_buttons(toolbar: &Box, canvas: &EditorCanvas) {
        let children = Self::get_children(toolbar);

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

    fn connect_action_buttons(toolbar: &Box, canvas: &EditorCanvas) {
        let buttons: Vec<_> = Self::get_children(toolbar)
            .into_iter()
            .filter_map(|w| w.downcast::<Button>().ok())
            .collect();

        for btn in buttons {
            let canvas = canvas.clone();

            btn.connect_clicked(move |button| {
                if let Some(data) = canvas.get_image_data() {
                    let result = match button.widget_name().as_str() {
                        "btn_save" => Self::save_screenshot(&data),
                        "btn_copy" => Self::copy_to_clipboard(&data),
                        _ => Ok(()),
                    };

                    if let Err(e) = result {
                        eprintln!("Error: {}", e);
                    }
                }
            });
        }
    }

    fn setup_keyboard_shortcuts(window: &ApplicationWindow, canvas: &EditorCanvas) {
        let key_controller = EventControllerKey::new();

        let canvas = canvas.clone();

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
                        if let Some(data) = canvas.get_image_data() {
                            if let Err(e) = Self::save_screenshot(&data) {
                                eprintln!("Save error: {}", e);
                            }
                        }
                        return glib::Propagation::Stop;
                    }
                    gtk4::gdk::Key::c => {
                        if let Some(data) = canvas.get_image_data() {
                            if let Err(e) = Self::copy_to_clipboard(&data) {
                                eprintln!("Copy error: {}", e);
                            }
                        }
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

    fn enable_action_buttons(toolbar: &Box, enabled: bool) {
        for widget in Self::get_children(toolbar) {
            if let Ok(btn) = widget.downcast::<Button>() {
                let name = btn.widget_name();
                if name == "btn_save" || name == "btn_copy" {
                    btn.set_sensitive(enabled);
                }
            }
        }
    }

    fn get_children(container: &Box) -> Vec<gtk4::Widget> {
        let mut children = Vec::new();
        let mut child = container.first_child();

        while let Some(widget) = child {
            children.push(widget.clone());
            child = widget.next_sibling();
        }

        children
    }

    fn save_screenshot(data: &[u8]) -> anyhow::Result<()> {
        use chrono::Local;
        use std::fs;
        use std::path::PathBuf;

        let pictures_dir = directories::UserDirs::new()
            .and_then(|dirs| dirs.picture_dir().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| {
                PathBuf::from(std::env::var("HOME").unwrap_or_default()).join("Pictures")
            });

        let screenshots_dir = pictures_dir.join("Screenshots");
        fs::create_dir_all(&screenshots_dir)?;

        let timestamp = Local::now().format("%Y-%m-%d-%H%M%S");
        let filename = format!("screenshot-{}.png", timestamp);
        let filepath = screenshots_dir.join(&filename);

        fs::write(&filepath, data)?;
        println!("Saved to: {}", filepath.display());

        Ok(())
    }

    fn copy_to_clipboard(data: &[u8]) -> anyhow::Result<()> {
        use std::io::Write;
        use std::process::{Command, Stdio};

        let mut child = Command::new("wl-copy")
            .arg("--type")
            .arg("image/png")
            .stdin(Stdio::piped())
            .spawn()?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(data)?;
        }

        child.wait()?;
        println!("Copied to clipboard");

        Ok(())
    }
}
