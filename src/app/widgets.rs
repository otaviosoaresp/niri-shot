use gtk4::gdk::RGBA;
use gtk4::prelude::*;
use gtk4::{
    ApplicationWindow, Box, Button, ColorButton, Label, Orientation, Scale, Separator, ToggleButton,
};

use crate::editor::EditorCanvas;

pub fn create_capture_bar() -> (Box, Label) {
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

    let status = Label::new(None);
    status.set_hexpand(true);
    status.set_halign(gtk4::Align::End);
    status.set_ellipsize(gtk4::pango::EllipsizeMode::End);

    bar.append(&btn_fullscreen);
    bar.append(&btn_region);
    bar.append(&btn_window);
    bar.append(&status);

    (bar, status)
}

pub fn show_status(label: &Label, message: &str, is_error: bool) {
    label.set_text(message);
    label.remove_css_class("status-error");
    label.remove_css_class("status-ok");
    label.add_css_class(if is_error {
        "status-error"
    } else {
        "status-ok"
    });
}

pub fn create_floating_toolbar() -> Box {
    let bar = Box::new(Orientation::Horizontal, 6);

    let btn_select = create_nerd_button("󰍽", "tool_select", "Select");
    btn_select.set_active(true);

    let btn_rect = create_nerd_button("□", "tool_rectangle", "Rectangle");
    btn_rect.set_group(Some(&btn_select));

    let btn_circle = create_nerd_button("○", "tool_circle", "Circle");
    btn_circle.set_group(Some(&btn_select));

    let btn_line = create_nerd_button("╱", "tool_line", "Line");
    btn_line.set_group(Some(&btn_select));

    let btn_arrow = create_nerd_button("󰁕", "tool_arrow", "Arrow");
    btn_arrow.set_group(Some(&btn_select));

    let btn_freehand = create_nerd_button("󰏬", "tool_freehand", "Freehand");
    btn_freehand.set_group(Some(&btn_select));

    let btn_text = create_nerd_button("󰊄", "tool_text", "Text");
    btn_text.set_group(Some(&btn_select));

    let btn_blur = create_nerd_button("󰂵", "tool_blur", "Blur");
    btn_blur.set_group(Some(&btn_select));

    let btn_highlight = create_nerd_button("󰸱", "tool_highlight", "Highlight");
    btn_highlight.set_group(Some(&btn_select));

    let color_btn = ColorButton::with_rgba(&RGBA::new(1.0, 0.0, 0.0, 1.0));
    color_btn.set_widget_name("color_picker");
    color_btn.set_tooltip_text(Some("Color"));

    let stroke_scale = Scale::with_range(Orientation::Horizontal, 1.0, 20.0, 1.0);
    stroke_scale.set_value(3.0);
    stroke_scale.set_width_request(80);
    stroke_scale.set_widget_name("stroke_width");
    stroke_scale.set_tooltip_text(Some("Stroke Width"));

    let btn_zoom_out = create_nerd_action_button("󰍴", "btn_zoom_out", "Zoom - (Ctrl+-)");
    let btn_zoom_in = create_nerd_action_button("󰍷", "btn_zoom_in", "Zoom + (Ctrl++)");

    let btn_undo = create_nerd_action_button("󰕌", "btn_undo", "Undo (Ctrl+Z)");
    let btn_redo = create_nerd_action_button("󰑎", "btn_redo", "Redo (Ctrl+Y)");

    let btn_save = create_nerd_action_button("󰆓", "btn_save", "Save (Ctrl+S)");
    btn_save.set_sensitive(false);

    let btn_copy = create_nerd_action_button("󰆏", "btn_copy", "Copy (Ctrl+C)");
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

pub fn resize_window_to_image(window: &ApplicationWindow, canvas: &EditorCanvas) {
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

pub fn enable_action_buttons(toolbar: &Box, enabled: bool) {
    for widget in get_children(toolbar) {
        if let Ok(btn) = widget.downcast::<Button>() {
            let name = btn.widget_name();
            if name == "btn_save" || name == "btn_copy" {
                btn.set_sensitive(enabled);
            }
        }
    }
}

pub fn get_children(container: &Box) -> Vec<gtk4::Widget> {
    let mut children = Vec::new();
    let mut child = container.first_child();

    while let Some(widget) = child {
        children.push(widget.clone());
        child = widget.next_sibling();
    }

    children
}
