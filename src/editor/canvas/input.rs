use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::{glib, EventControllerMotion, GestureClick, GestureDrag};
use std::f64::consts::PI;

use super::{EditorCanvas, HandleType};
use crate::editor::history::UndoEntry;
use crate::editor::shapes::{Shape, ShapeType};
use crate::editor::tools::ToolType;

impl EditorCanvas {
    pub(super) fn setup_events(&self) {
        let click = GestureClick::new();
        click.set_button(1);

        let canvas = self.clone();
        click.connect_pressed(move |_, _, x, y| {
            canvas.grab_focus();
            canvas.on_press(x, y);
        });

        let canvas = self.clone();
        click.connect_released(move |_, _, x, y| {
            canvas.on_release(x, y);
        });

        self.add_controller(click);

        let drag = GestureDrag::new();

        let canvas = self.clone();
        drag.connect_drag_begin(move |_, x, y| {
            canvas.on_drag_begin(x, y);
        });

        let canvas = self.clone();
        drag.connect_drag_update(move |_, offset_x, offset_y| {
            canvas.on_drag_update(offset_x, offset_y);
        });

        let canvas = self.clone();
        drag.connect_drag_end(move |_, offset_x, offset_y| {
            canvas.on_drag_end(offset_x, offset_y);
        });

        self.add_controller(drag);

        let motion = EventControllerMotion::new();

        let canvas = self.clone();
        motion.connect_motion(move |_, x, y| {
            canvas.on_motion(x, y);
        });

        self.add_controller(motion);

        let key = gtk4::EventControllerKey::new();
        let canvas = self.clone();
        key.connect_key_pressed(move |_, keyval, _, state| canvas.on_key_pressed(keyval, state));

        self.add_controller(key);

        let scroll = gtk4::EventControllerScroll::new(gtk4::EventControllerScrollFlags::VERTICAL);
        let canvas = self.clone();
        scroll.connect_scroll(move |controller, _dx, dy| {
            let state = controller.current_event_state();
            if state.contains(gtk4::gdk::ModifierType::CONTROL_MASK) {
                if dy < 0.0 {
                    canvas.zoom_in();
                } else if dy > 0.0 {
                    canvas.zoom_out();
                }
                return glib::Propagation::Stop;
            }
            glib::Propagation::Proceed
        });

        self.add_controller(scroll);

        let middle_click = GestureClick::new();
        middle_click.set_button(2);

        let canvas = self.clone();
        middle_click.connect_pressed(move |_, _, x, y| {
            canvas.imp().panning.set(true);
            canvas.imp().pan_start_x.set(x);
            canvas.imp().pan_start_y.set(y);
        });

        let canvas = self.clone();
        middle_click.connect_released(move |_, _, _, _| {
            canvas.imp().panning.set(false);
        });

        self.add_controller(middle_click);

        let pan_drag = GestureDrag::new();
        pan_drag.set_button(2);

        let canvas = self.clone();
        pan_drag.connect_drag_update(move |_, offset_x, offset_y| {
            if canvas.imp().panning.get() {
                canvas.pan(-offset_x, -offset_y);
            }
        });

        self.add_controller(pan_drag);

        let right_click = GestureClick::new();
        right_click.set_button(3);

        let canvas = self.clone();
        right_click.connect_pressed(move |gesture, _, x, y| {
            let state = gesture.current_event_state();
            if state.contains(gtk4::gdk::ModifierType::SHIFT_MASK) {
                canvas.imp().panning.set(true);
                canvas.imp().pan_start_x.set(x);
                canvas.imp().pan_start_y.set(y);
            }
        });

        let canvas = self.clone();
        right_click.connect_released(move |_, _, _, _| {
            canvas.imp().panning.set(false);
        });

        self.add_controller(right_click);

        let right_pan_drag = GestureDrag::new();
        right_pan_drag.set_button(3);

        let canvas = self.clone();
        right_pan_drag.connect_drag_update(move |gesture, offset_x, offset_y| {
            let state = gesture.current_event_state();
            if state.contains(gtk4::gdk::ModifierType::SHIFT_MASK) && canvas.imp().panning.get() {
                canvas.pan(-offset_x, -offset_y);
            }
        });

        self.add_controller(right_pan_drag);
    }

    fn on_key_pressed(
        &self,
        keyval: gtk4::gdk::Key,
        state: gtk4::gdk::ModifierType,
    ) -> glib::Propagation {
        let imp = self.imp();

        if !imp.text_input_active.get() {
            if matches!(keyval, gtk4::gdk::Key::Delete | gtk4::gdk::Key::BackSpace)
                && imp.selected_index.get().is_some()
            {
                self.delete_selected();
                return glib::Propagation::Stop;
            }
            return glib::Propagation::Proceed;
        }

        // Shortcuts stay with the window even mid-annotation, otherwise Ctrl+S
        // would type an "s" into the text instead of saving.
        if state
            .intersects(gtk4::gdk::ModifierType::CONTROL_MASK | gtk4::gdk::ModifierType::ALT_MASK)
        {
            return glib::Propagation::Proceed;
        }

        match keyval {
            gtk4::gdk::Key::Return | gtk4::gdk::Key::KP_Enter => {
                self.finish_text_input();
                glib::Propagation::Stop
            }
            gtk4::gdk::Key::Escape => {
                self.cancel_text_input();
                glib::Propagation::Stop
            }
            gtk4::gdk::Key::BackSpace => {
                let mut buffer = imp.text_input_buffer.borrow_mut();
                buffer.pop();
                self.queue_draw();
                glib::Propagation::Stop
            }
            _ => {
                if let Some(c) = keyval.to_unicode() {
                    if !c.is_control() {
                        imp.text_input_buffer.borrow_mut().push(c);
                        self.queue_draw();
                    }
                }
                glib::Propagation::Stop
            }
        }
    }

    fn on_press(&self, screen_x: f64, screen_y: f64) {
        let (x, y) = self.screen_to_canvas(screen_x, screen_y);
        let imp = self.imp();
        let tool_type = imp.tool.borrow().tool_type;

        if imp.text_input_active.get() {
            self.finish_text_input();
            return;
        }

        match tool_type {
            ToolType::Select => {
                let handle = self.hit_test_handle(x, y);
                if handle != HandleType::None {
                    imp.active_handle.set(handle);
                    imp.dragging.set(true);
                    imp.drag_start_x.set(x);
                    imp.drag_start_y.set(y);

                    if handle == HandleType::Rotation {
                        if let Some(idx) = imp.selected_index.get() {
                            let shapes = imp.shapes.borrow();
                            if let Some(shape) = shapes.get(idx) {
                                imp.initial_rotation.set(shape.rotation);
                            }
                        }
                    }
                } else if let Some(idx) = self.hit_test(x, y) {
                    imp.selected_index.set(Some(idx));
                    imp.active_handle.set(HandleType::None);
                    imp.dragging.set(true);
                    imp.drag_start_x.set(x);
                    imp.drag_start_y.set(y);
                } else {
                    imp.selected_index.set(None);
                    imp.active_handle.set(HandleType::None);
                    *imp.pending_modify.borrow_mut() = None;
                }
                self.queue_draw();
            }
            ToolType::Text => {
                imp.text_input_active.set(true);
                *imp.text_input_pos.borrow_mut() = Some((x, y));
                *imp.text_input_buffer.borrow_mut() = String::new();
                self.queue_draw();
            }
            ToolType::FreeHand => {
                imp.drawing.set(true);
                let tool = imp.tool.borrow();
                let shape = Shape {
                    shape_type: ShapeType::FreeHand,
                    start_x: x,
                    start_y: y,
                    end_x: x,
                    end_y: y,
                    color: tool.color,
                    stroke_width: tool.stroke_width,
                    filled: false,
                    points: vec![(x, y)],
                    ..Default::default()
                };
                *imp.current_shape.borrow_mut() = Some(shape);
            }
            _ => {
                imp.drawing.set(true);
                let tool = imp.tool.borrow();
                if let Some(shape) = tool.create_shape(x, y, x, y) {
                    *imp.current_shape.borrow_mut() = Some(shape);
                }
            }
        }
    }

    fn on_motion(&self, screen_x: f64, screen_y: f64) {
        let (x, y) = self.screen_to_canvas(screen_x, screen_y);
        let imp = self.imp();

        if !imp.drawing.get() {
            return;
        }

        let tool_type = imp.tool.borrow().tool_type;

        if let Some(ref mut shape) = *imp.current_shape.borrow_mut() {
            if tool_type == ToolType::FreeHand {
                shape.add_point(x, y);
            } else {
                shape.end_x = x;
                shape.end_y = y;
            }
        }

        self.queue_draw();
    }

    fn on_drag_begin(&self, screen_x: f64, screen_y: f64) {
        let (x, y) = self.screen_to_canvas(screen_x, screen_y);
        let imp = self.imp();
        let tool_type = imp.tool.borrow().tool_type;

        if tool_type == ToolType::Select {
            if let Some(idx) = imp.selected_index.get() {
                if idx < imp.shapes.borrow().len() {
                    imp.dragging.set(true);
                    imp.drag_start_x.set(x);
                    imp.drag_start_y.set(y);
                    imp.drag_offset_x.set(0.0);
                    imp.drag_offset_y.set(0.0);
                }
            }
        }
    }

    fn on_drag_update(&self, screen_offset_x: f64, screen_offset_y: f64) {
        let imp = self.imp();
        let zoom = imp.zoom.get();
        let offset_x = screen_offset_x / zoom;
        let offset_y = screen_offset_y / zoom;

        if !imp.dragging.get() {
            return;
        }

        let Some(idx) = imp.selected_index.get() else {
            return;
        };

        if imp.pending_modify.borrow().is_none() {
            if let Some(shape) = imp.shapes.borrow().get(idx) {
                *imp.pending_modify.borrow_mut() = Some((idx, shape.clone()));
            }
        }

        let active_handle = imp.active_handle.get();
        let start_x = imp.drag_start_x.get();
        let start_y = imp.drag_start_y.get();
        let current_x = start_x + offset_x;
        let current_y = start_y + offset_y;

        match active_handle {
            HandleType::None => {
                imp.drag_offset_x.set(offset_x);
                imp.drag_offset_y.set(offset_y);
            }
            HandleType::Rotation => {
                let mut shapes = imp.shapes.borrow_mut();
                if let Some(shape) = shapes.get_mut(idx) {
                    let (cx, cy) = shape.center();
                    let angle = (current_y - cy).atan2(current_x - cx) + PI / 2.0;
                    shape.set_rotation(angle);
                }
            }
            _ => {
                let corner = match active_handle {
                    HandleType::TopLeft => 0,
                    HandleType::TopRight => 1,
                    HandleType::BottomLeft => 2,
                    HandleType::BottomRight => 3,
                    _ => return,
                };

                let mut shapes = imp.shapes.borrow_mut();
                if let Some(shape) = shapes.get_mut(idx) {
                    shape.resize_corner(corner, current_x, current_y);
                }
            }
        }

        self.queue_draw();
    }

    fn on_drag_end(&self, screen_offset_x: f64, screen_offset_y: f64) {
        let imp = self.imp();
        let zoom = imp.zoom.get();
        let offset_x = screen_offset_x / zoom;
        let offset_y = screen_offset_y / zoom;

        if imp.dragging.get() {
            let active_handle = imp.active_handle.get();

            if active_handle == HandleType::None {
                if let Some(idx) = imp.selected_index.get() {
                    let mut shapes = imp.shapes.borrow_mut();
                    if idx < shapes.len() {
                        let shape = &mut shapes[idx];
                        shape.translate(offset_x, offset_y);
                    }
                }
            }

            let snapshot = imp.pending_modify.borrow_mut().take();
            if let Some((idx, before)) = snapshot {
                let changed = imp.shapes.borrow().get(idx) != Some(&before);
                if changed {
                    self.push_undo(UndoEntry::Modify { idx, shape: before });
                }
            }

            imp.dragging.set(false);
            imp.active_handle.set(HandleType::None);
            imp.drag_offset_x.set(0.0);
            imp.drag_offset_y.set(0.0);
            self.queue_draw();
        }
    }

    fn on_release(&self, screen_x: f64, screen_y: f64) {
        let (x, y) = self.screen_to_canvas(screen_x, screen_y);
        let imp = self.imp();

        if !imp.drawing.get() {
            return;
        }

        imp.drawing.set(false);

        if let Some(mut shape) = imp.current_shape.borrow_mut().take() {
            shape.end_x = x;
            shape.end_y = y;
            self.commit_shape(shape);
        }

        self.queue_draw();
    }

    fn hit_test(&self, x: f64, y: f64) -> Option<usize> {
        let shapes = self.imp().shapes.borrow();

        for (idx, shape) in shapes.iter().enumerate().rev() {
            if shape.hit(x, y) {
                return Some(idx);
            }
        }

        None
    }

    fn hit_test_handle(&self, x: f64, y: f64) -> HandleType {
        let imp = self.imp();
        let selected_idx = match imp.selected_index.get() {
            Some(idx) => idx,
            None => return HandleType::None,
        };

        let shapes = imp.shapes.borrow();
        let shape = match shapes.get(selected_idx) {
            Some(s) => s,
            None => return HandleType::None,
        };

        let (min_x, min_y, max_x, max_y) = shape.bounds();
        let (test_x, test_y) = shape.to_local(x, y);

        let handle_size = 8.0;

        let center_x = (min_x + max_x) / 2.0;
        let rotation_y = min_y - 25.0;
        if (test_x - center_x).abs() <= handle_size && (test_y - rotation_y).abs() <= handle_size {
            return HandleType::Rotation;
        }

        let corners = [
            (min_x, min_y, HandleType::TopLeft),
            (max_x, min_y, HandleType::TopRight),
            (min_x, max_y, HandleType::BottomLeft),
            (max_x, max_y, HandleType::BottomRight),
        ];

        for (corner_x, corner_y, handle) in corners {
            if (test_x - corner_x).abs() <= handle_size && (test_y - corner_y).abs() <= handle_size
            {
                return handle;
            }
        }

        HandleType::None
    }

    pub(super) fn finish_text_input(&self) {
        let imp = self.imp();

        if let Some((x, y)) = imp.text_input_pos.borrow_mut().take() {
            let text = imp.text_input_buffer.borrow().clone();
            if !text.is_empty() {
                let shape = imp.tool.borrow().create_text_shape(x, y, text);
                self.commit_shape(shape);
            }
        }

        imp.text_input_active.set(false);
        *imp.text_input_buffer.borrow_mut() = String::new();
        self.queue_draw();
    }

    fn cancel_text_input(&self) {
        let imp = self.imp();
        imp.text_input_active.set(false);
        *imp.text_input_pos.borrow_mut() = None;
        *imp.text_input_buffer.borrow_mut() = String::new();
        self.queue_draw();
    }
}
