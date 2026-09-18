mod actions;
mod widgets;

use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box, CssProvider, Orientation, Overlay, ScrolledWindow,
    Separator,
};
use std::cell::RefCell;
use std::rc::Rc;

use crate::editor::EditorCanvas;

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

            .status-error {
                color: #ff6b6b;
            }

            .status-ok {
                color: #8bc34a;
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

        let (capture_bar, status) = widgets::create_capture_bar();
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

        let floating_toolbar = widgets::create_floating_toolbar();
        floating_toolbar.add_css_class("floating-toolbar");
        floating_toolbar.set_halign(gtk4::Align::Center);
        floating_toolbar.set_valign(gtk4::Align::End);
        floating_toolbar.set_margin_bottom(20);

        overlay.add_overlay(&floating_toolbar);

        main_box.append(&capture_bar);
        main_box.append(&Separator::new(Orientation::Horizontal));
        main_box.append(&overlay);

        actions::connect_capture_buttons(
            &capture_bar,
            &canvas,
            &window,
            &floating_toolbar,
            &status,
        );
        actions::connect_tool_buttons(&floating_toolbar, &canvas);
        actions::connect_action_buttons(&floating_toolbar, &canvas, &status);
        actions::setup_keyboard_shortcuts(&window, &canvas, &status);

        window.set_child(Some(&main_box));

        let data = initial_data.borrow_mut().take();
        window.present();

        if let Some(image_data) = data {
            canvas.set_image(&image_data);
            widgets::enable_action_buttons(&floating_toolbar, true);
            widgets::resize_window_to_image(&window, &canvas);
        }
    }
}
