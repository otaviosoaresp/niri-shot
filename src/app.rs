use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box, Button, Orientation, ScrolledWindow,
    ToggleButton, Separator, ColorButton, Scale, Label, EventControllerKey,
};
use gtk4::gdk::RGBA;
use gtk4::glib;

use crate::capture::{CaptureBackend, CaptureMode};
use crate::editor::{EditorCanvas, ToolType, Color};

const APP_ID: &str = "com.github.niri-shot";

pub struct NiriShotApp {
    app: Application,
}

impl NiriShotApp {
    pub fn new() -> Self {
        let app = Application::builder()
            .application_id(APP_ID)
            .build();

        app.connect_activate(Self::build_ui);

        Self { app }
    }

    pub fn run(&self) {
        self.app.run();
    }

    fn build_ui(app: &Application) {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("niri-shot")
            .default_width(1000)
            .default_height(700)
            .build();

        let main_box = Box::new(Orientation::Vertical, 0);

        let capture_bar = Self::create_capture_bar();
        let tool_bar = Self::create_tool_bar();
        let action_bar = Self::create_action_bar();

        let canvas = EditorCanvas::new();
        canvas.set_hexpand(true);
        canvas.set_vexpand(true);

        let scrolled = ScrolledWindow::new();
        scrolled.set_child(Some(&canvas));
        scrolled.set_vexpand(true);
        scrolled.set_hexpand(true);

        main_box.append(&capture_bar);
        main_box.append(&Separator::new(Orientation::Horizontal));
        main_box.append(&tool_bar);
        main_box.append(&Separator::new(Orientation::Horizontal));
        main_box.append(&scrolled);
        main_box.append(&Separator::new(Orientation::Horizontal));
        main_box.append(&action_bar);

        Self::connect_capture_buttons(&capture_bar, &canvas, &window, &action_bar);
        Self::connect_tool_buttons(&tool_bar, &canvas, &window);
        Self::connect_action_buttons(&action_bar, &canvas);
        Self::setup_keyboard_shortcuts(&window, &canvas);

        window.set_child(Some(&main_box));
        window.present();
    }

    fn create_capture_bar() -> Box {
        let bar = Box::new(Orientation::Horizontal, 5);
        bar.set_margin_top(5);
        bar.set_margin_bottom(5);
        bar.set_margin_start(10);
        bar.set_margin_end(10);

        let btn_fullscreen = Button::with_label("Tela Inteira");
        btn_fullscreen.set_widget_name("btn_fullscreen");

        let btn_region = Button::with_label("Regiao");
        btn_region.set_widget_name("btn_region");

        let btn_window = Button::with_label("Janela");
        btn_window.set_widget_name("btn_window");

        bar.append(&btn_fullscreen);
        bar.append(&btn_region);
        bar.append(&btn_window);

        bar
    }

    fn create_tool_bar() -> Box {
        let bar = Box::new(Orientation::Horizontal, 5);
        bar.set_margin_top(5);
        bar.set_margin_bottom(5);
        bar.set_margin_start(10);
        bar.set_margin_end(10);

        let btn_select = ToggleButton::with_label("Selecao");
        btn_select.set_widget_name("tool_select");
        btn_select.set_active(true);

        let btn_rect = ToggleButton::with_label("Retangulo");
        btn_rect.set_widget_name("tool_rectangle");
        btn_rect.set_group(Some(&btn_select));

        let btn_circle = ToggleButton::with_label("Circulo");
        btn_circle.set_widget_name("tool_circle");
        btn_circle.set_group(Some(&btn_select));

        let btn_line = ToggleButton::with_label("Linha");
        btn_line.set_widget_name("tool_line");
        btn_line.set_group(Some(&btn_select));

        let btn_arrow = ToggleButton::with_label("Seta");
        btn_arrow.set_widget_name("tool_arrow");
        btn_arrow.set_group(Some(&btn_select));

        let btn_freehand = ToggleButton::with_label("Desenho");
        btn_freehand.set_widget_name("tool_freehand");
        btn_freehand.set_group(Some(&btn_select));

        let btn_text = ToggleButton::with_label("Texto");
        btn_text.set_widget_name("tool_text");
        btn_text.set_group(Some(&btn_select));

        let btn_blur = ToggleButton::with_label("Blur");
        btn_blur.set_widget_name("tool_blur");
        btn_blur.set_group(Some(&btn_select));

        let btn_highlight = ToggleButton::with_label("Destaque");
        btn_highlight.set_widget_name("tool_highlight");
        btn_highlight.set_group(Some(&btn_select));

        let color_btn = ColorButton::with_rgba(&RGBA::new(1.0, 0.0, 0.0, 1.0));
        color_btn.set_widget_name("color_picker");
        color_btn.set_title("Cor");

        let stroke_label = Label::new(Some("Espessura:"));
        let stroke_scale = Scale::with_range(Orientation::Horizontal, 1.0, 20.0, 1.0);
        stroke_scale.set_value(3.0);
        stroke_scale.set_width_request(100);
        stroke_scale.set_widget_name("stroke_width");

        let btn_undo = Button::with_label("Desfazer");
        btn_undo.set_widget_name("btn_undo");

        let btn_redo = Button::with_label("Refazer");
        btn_redo.set_widget_name("btn_redo");

        let btn_clear = Button::with_label("Limpar");
        btn_clear.set_widget_name("btn_clear");

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
        bar.append(&Separator::new(Orientation::Vertical));
        bar.append(&stroke_label);
        bar.append(&stroke_scale);
        bar.append(&Separator::new(Orientation::Vertical));
        bar.append(&btn_undo);
        bar.append(&btn_redo);
        bar.append(&btn_clear);

        bar
    }

    fn create_action_bar() -> Box {
        let bar = Box::new(Orientation::Horizontal, 5);
        bar.set_margin_top(5);
        bar.set_margin_bottom(5);
        bar.set_margin_start(10);
        bar.set_margin_end(10);

        let btn_save = Button::with_label("Salvar (Ctrl+S)");
        btn_save.set_widget_name("btn_save");
        btn_save.set_sensitive(false);

        let btn_copy = Button::with_label("Copiar (Ctrl+C)");
        btn_copy.set_widget_name("btn_copy");
        btn_copy.set_sensitive(false);

        bar.append(&btn_save);
        bar.append(&btn_copy);

        bar
    }

    fn connect_capture_buttons(
        capture_bar: &Box,
        canvas: &EditorCanvas,
        window: &ApplicationWindow,
        action_bar: &Box,
    ) {
        let buttons: Vec<_> = Self::get_children(capture_bar)
            .into_iter()
            .filter_map(|w| w.downcast::<Button>().ok())
            .collect();

        for btn in buttons {
            let canvas = canvas.clone();
            let window = window.clone();
            let action_bar = action_bar.clone();

            btn.connect_clicked(move |button| {
                let mode = match button.widget_name().as_str() {
                    "btn_fullscreen" => CaptureMode::Fullscreen,
                    "btn_region" => CaptureMode::Region,
                    "btn_window" => CaptureMode::Window,
                    _ => return,
                };

                window.set_visible(false);

                let result = CaptureBackend::capture(mode);

                window.set_visible(true);

                match result {
                    Ok(data) => {
                        canvas.set_image(&data);
                        Self::enable_action_buttons(&action_bar, true);
                    }
                    Err(e) => eprintln!("Erro na captura: {}", e),
                }
            });
        }
    }

    fn connect_tool_buttons(tool_bar: &Box, canvas: &EditorCanvas, _window: &ApplicationWindow) {
        let children = Self::get_children(tool_bar);

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

                btn.connect_clicked(move |button| {
                    match button.widget_name().as_str() {
                        "btn_undo" => canvas.undo(),
                        "btn_redo" => canvas.redo(),
                        "btn_clear" => canvas.clear_shapes(),
                        _ => {}
                    }
                });
            }
        }

    }

    fn connect_action_buttons(action_bar: &Box, canvas: &EditorCanvas) {
        let buttons: Vec<_> = Self::get_children(action_bar)
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
                        eprintln!("Erro: {}", e);
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
                                eprintln!("Erro ao salvar: {}", e);
                            }
                        }
                        return glib::Propagation::Stop;
                    }
                    gtk4::gdk::Key::c => {
                        if let Some(data) = canvas.get_image_data() {
                            if let Err(e) = Self::copy_to_clipboard(&data) {
                                eprintln!("Erro ao copiar: {}", e);
                            }
                        }
                        return glib::Propagation::Stop;
                    }
                    _ => {}
                }
            }

            glib::Propagation::Proceed
        });

        window.add_controller(key_controller);
    }

    fn enable_action_buttons(action_bar: &Box, enabled: bool) {
        for widget in Self::get_children(action_bar) {
            if let Ok(btn) = widget.downcast::<Button>() {
                btn.set_sensitive(enabled);
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
        println!("Salvo em: {}", filepath.display());

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
        println!("Copiado para clipboard");

        Ok(())
    }
}
