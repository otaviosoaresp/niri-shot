use gtk4::gdk_pixbuf::Pixbuf;
use gtk4::gio::{Cancellable, MemoryInputStream};
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::{glib, DrawingArea, EventControllerMotion, GestureClick, GestureDrag};
use std::cell::{Cell, RefCell};
use std::f64::consts::PI;

use super::shapes::{Shape, ShapeType};
use super::tools::{Tool, ToolType};

#[derive(Clone, Copy, PartialEq, Default)]
pub enum HandleType {
    #[default]
    None,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Rotation,
}

mod imp {
    use super::*;

    pub struct EditorCanvas {
        pub image: RefCell<Option<Pixbuf>>,
        pub shapes: RefCell<Vec<Shape>>,
        pub redo_stack: RefCell<Vec<Shape>>,
        pub current_shape: RefCell<Option<Shape>>,
        pub tool: RefCell<Tool>,
        pub drawing: Cell<bool>,
        pub selected_index: Cell<Option<usize>>,
        pub dragging: Cell<bool>,
        pub drag_start_x: Cell<f64>,
        pub drag_start_y: Cell<f64>,
        pub drag_offset_x: Cell<f64>,
        pub drag_offset_y: Cell<f64>,
        pub text_input_active: Cell<bool>,
        pub text_input_pos: RefCell<Option<(f64, f64)>>,
        pub text_input_buffer: RefCell<String>,
        pub active_handle: Cell<HandleType>,
        pub initial_rotation: Cell<f64>,
        pub zoom: Cell<f64>,
        pub panning: Cell<bool>,
        pub pan_start_x: Cell<f64>,
        pub pan_start_y: Cell<f64>,
    }

    impl Default for EditorCanvas {
        fn default() -> Self {
            Self {
                image: RefCell::new(None),
                shapes: RefCell::new(Vec::new()),
                redo_stack: RefCell::new(Vec::new()),
                current_shape: RefCell::new(None),
                tool: RefCell::new(Tool::default()),
                drawing: Cell::new(false),
                selected_index: Cell::new(None),
                dragging: Cell::new(false),
                drag_start_x: Cell::new(0.0),
                drag_start_y: Cell::new(0.0),
                drag_offset_x: Cell::new(0.0),
                drag_offset_y: Cell::new(0.0),
                text_input_active: Cell::new(false),
                text_input_pos: RefCell::new(None),
                text_input_buffer: RefCell::new(String::new()),
                active_handle: Cell::new(HandleType::None),
                initial_rotation: Cell::new(0.0),
                zoom: Cell::new(1.0),
                panning: Cell::new(false),
                pan_start_x: Cell::new(0.0),
                pan_start_y: Cell::new(0.0),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for EditorCanvas {
        const NAME: &'static str = "EditorCanvas";
        type Type = super::EditorCanvas;
        type ParentType = DrawingArea;
    }

    impl ObjectImpl for EditorCanvas {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_drawing();
            self.obj().setup_events();
        }
    }

    impl WidgetImpl for EditorCanvas {}
    impl DrawingAreaImpl for EditorCanvas {}
}

glib::wrapper! {
    pub struct EditorCanvas(ObjectSubclass<imp::EditorCanvas>)
        @extends DrawingArea, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Default for EditorCanvas {
    fn default() -> Self {
        Self::new()
    }
}

impl EditorCanvas {
    pub fn new() -> Self {
        let canvas: Self = glib::Object::builder().build();
        canvas.set_focusable(true);
        canvas.set_can_focus(true);
        canvas
    }

    fn setup_drawing(&self) {
        self.set_draw_func(|widget, ctx, width, height| {
            let canvas = widget.downcast_ref::<EditorCanvas>().unwrap();
            canvas.draw(ctx, width, height);
        });
    }

    fn setup_events(&self) {
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
        key.connect_key_pressed(move |_, keyval, _, _| canvas.on_key_pressed(keyval));

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

    fn on_key_pressed(&self, keyval: gtk4::gdk::Key) -> glib::Propagation {
        let imp = self.imp();

        if !imp.text_input_active.get() {
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
                imp.redo_stack.borrow_mut().clear();
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
                imp.redo_stack.borrow_mut().clear();
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
                let shapes = imp.shapes.borrow();
                if idx < shapes.len() {
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
            imp.shapes.borrow_mut().push(shape);
        }

        self.queue_draw();
    }

    fn hit_test(&self, x: f64, y: f64) -> Option<usize> {
        let shapes = self.imp().shapes.borrow();

        for (idx, shape) in shapes.iter().enumerate().rev() {
            if shape.contains_point(x, y) {
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
        let (cx, cy) = shape.center();

        let (test_x, test_y) = if shape.rotation.abs() > 0.001 {
            let dx = x - cx;
            let dy = y - cy;
            let cos_r = (-shape.rotation).cos();
            let sin_r = (-shape.rotation).sin();
            (cx + dx * cos_r - dy * sin_r, cy + dx * sin_r + dy * cos_r)
        } else {
            (x, y)
        };

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

    fn finish_text_input(&self) {
        let imp = self.imp();

        if let Some((x, y)) = imp.text_input_pos.borrow_mut().take() {
            let text = imp.text_input_buffer.borrow().clone();
            if !text.is_empty() {
                let tool = imp.tool.borrow();
                let shape = tool.create_text_shape(x, y, text);
                imp.redo_stack.borrow_mut().clear();
                imp.shapes.borrow_mut().push(shape);
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

    fn draw(&self, ctx: &cairo::Context, _width: i32, _height: i32) {
        let imp = self.imp();
        let zoom = imp.zoom.get();

        ctx.save().ok();
        ctx.scale(zoom, zoom);

        if let Some(ref pixbuf) = *imp.image.borrow() {
            gtk4::prelude::GdkCairoContextExt::set_source_pixbuf(ctx, pixbuf, 0.0, 0.0);
            let _ = ctx.paint();
        }

        let selected_idx = imp.selected_index.get();
        let dragging = imp.dragging.get();
        let offset_x = imp.drag_offset_x.get();
        let offset_y = imp.drag_offset_y.get();
        let active_handle = imp.active_handle.get();
        let is_moving = dragging && active_handle == HandleType::None;

        for (idx, shape) in imp.shapes.borrow().iter().enumerate() {
            let is_selected = selected_idx == Some(idx);

            if is_selected && is_moving {
                ctx.save().ok();
                ctx.translate(offset_x, offset_y);
                shape.draw(ctx);
                ctx.restore().ok();
            } else {
                shape.draw(ctx);
            }

            if is_selected {
                self.draw_selection_handles(
                    ctx,
                    shape,
                    if is_moving {
                        (offset_x, offset_y)
                    } else {
                        (0.0, 0.0)
                    },
                );
            }
        }

        if let Some(ref shape) = *imp.current_shape.borrow() {
            shape.draw(ctx);
        }

        if imp.text_input_active.get() {
            if let Some((x, y)) = *imp.text_input_pos.borrow() {
                self.draw_text_cursor(ctx, x, y);
            }
        }

        ctx.restore().ok();
    }

    fn draw_selection_handles(&self, ctx: &cairo::Context, shape: &Shape, offset: (f64, f64)) {
        let bounds = shape.bounds();
        let (min_x, min_y, max_x, max_y) = (
            bounds.0 + offset.0,
            bounds.1 + offset.1,
            bounds.2 + offset.0,
            bounds.3 + offset.1,
        );

        let (cx, cy) = ((min_x + max_x) / 2.0, (min_y + max_y) / 2.0);
        let has_rotation = shape.rotation.abs() > 0.001;

        if has_rotation {
            ctx.save().ok();
            ctx.translate(cx, cy);
            ctx.rotate(shape.rotation);
            ctx.translate(-cx, -cy);
        }

        ctx.set_source_rgba(0.2, 0.5, 1.0, 0.8);
        ctx.set_line_width(1.5);
        ctx.set_dash(&[4.0, 4.0], 0.0);
        ctx.rectangle(
            min_x - 3.0,
            min_y - 3.0,
            max_x - min_x + 6.0,
            max_y - min_y + 6.0,
        );
        let _ = ctx.stroke();
        ctx.set_dash(&[], 0.0);

        let handle_size = 6.0;
        ctx.set_source_rgba(0.2, 0.5, 1.0, 1.0);

        for (hx, hy) in [
            (min_x, min_y),
            (max_x, min_y),
            (min_x, max_y),
            (max_x, max_y),
        ] {
            ctx.rectangle(
                hx - handle_size / 2.0,
                hy - handle_size / 2.0,
                handle_size,
                handle_size,
            );
            let _ = ctx.fill();
        }

        let center_x = (min_x + max_x) / 2.0;
        let rotation_y = min_y - 25.0;

        ctx.set_source_rgba(0.2, 0.7, 0.3, 1.0);
        ctx.set_line_width(1.5);
        ctx.move_to(center_x, min_y);
        ctx.line_to(center_x, rotation_y + 6.0);
        let _ = ctx.stroke();

        ctx.arc(center_x, rotation_y, 6.0, 0.0, 2.0 * PI);
        let _ = ctx.fill();

        if has_rotation {
            ctx.restore().ok();
        }
    }

    fn draw_text_cursor(&self, ctx: &cairo::Context, x: f64, y: f64) {
        let imp = self.imp();
        let tool = imp.tool.borrow();
        let text = imp.text_input_buffer.borrow();

        ctx.set_source_rgba(0.0, 0.0, 0.0, 0.5);
        ctx.rectangle(x - 2.0, y - 2.0, 200.0, tool.font_size + 8.0);
        let _ = ctx.fill();

        ctx.set_source_rgba(tool.color.r, tool.color.g, tool.color.b, tool.color.a);
        ctx.select_font_face("Sans", cairo::FontSlant::Normal, cairo::FontWeight::Bold);
        ctx.set_font_size(tool.font_size);
        ctx.move_to(x, y + tool.font_size);
        let _ = ctx.show_text(&text);

        let cursor_x = if let Ok(extents) = ctx.text_extents(&text) {
            x + extents.width() + 2.0
        } else {
            x + 2.0
        };
        ctx.set_source_rgba(1.0, 1.0, 1.0, 1.0);
        ctx.set_line_width(2.0);
        ctx.move_to(cursor_x, y);
        ctx.line_to(cursor_x, y + tool.font_size + 4.0);
        let _ = ctx.stroke();
    }

    pub fn set_image(&self, data: &[u8]) {
        let bytes = glib::Bytes::from(data);
        let stream = MemoryInputStream::from_bytes(&bytes);

        if let Ok(pixbuf) = Pixbuf::from_stream(&stream, Cancellable::NONE) {
            self.set_content_width(pixbuf.width());
            self.set_content_height(pixbuf.height());
            *self.imp().image.borrow_mut() = Some(pixbuf);
            self.imp().shapes.borrow_mut().clear();
            self.imp().redo_stack.borrow_mut().clear();
            self.imp().selected_index.set(None);
            self.queue_draw();
        }
    }

    pub fn set_tool_type(&self, tool_type: ToolType) {
        let imp = self.imp();

        if imp.text_input_active.get() {
            self.finish_text_input();
        }

        imp.tool.borrow_mut().tool_type = tool_type;
        imp.selected_index.set(None);
        self.queue_draw();
    }

    pub fn set_color(&self, color: super::shapes::Color) {
        self.imp().tool.borrow_mut().color = color;
    }

    pub fn set_stroke_width(&self, width: f64) {
        self.imp().tool.borrow_mut().stroke_width = width;
    }

    #[allow(dead_code)]
    pub fn set_filled(&self, filled: bool) {
        self.imp().tool.borrow_mut().filled = filled;
    }

    pub fn clear_shapes(&self) {
        let imp = self.imp();
        let shapes = imp.shapes.borrow().clone();
        if !shapes.is_empty() {
            imp.redo_stack.borrow_mut().extend(shapes);
            imp.shapes.borrow_mut().clear();
        }
        imp.selected_index.set(None);
        self.queue_draw();
    }

    pub fn undo(&self) {
        let imp = self.imp();
        if let Some(shape) = imp.shapes.borrow_mut().pop() {
            imp.redo_stack.borrow_mut().push(shape);
        }
        imp.selected_index.set(None);
        self.queue_draw();
    }

    pub fn redo(&self) {
        let imp = self.imp();
        if let Some(shape) = imp.redo_stack.borrow_mut().pop() {
            imp.shapes.borrow_mut().push(shape);
        }
        self.queue_draw();
    }

    pub fn delete_selected(&self) {
        let imp = self.imp();
        if let Some(idx) = imp.selected_index.get() {
            let mut shapes = imp.shapes.borrow_mut();
            if idx < shapes.len() {
                let removed = shapes.remove(idx);
                imp.redo_stack.borrow_mut().push(removed);
                imp.selected_index.set(None);
            }
        }
        self.queue_draw();
    }

    pub fn get_image_data(&self) -> Option<Vec<u8>> {
        let imp = self.imp();
        let pixbuf = imp.image.borrow();
        let pixbuf = pixbuf.as_ref()?;

        let width = pixbuf.width();
        let height = pixbuf.height();

        let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, width, height).ok()?;
        let ctx = cairo::Context::new(&surface).ok()?;

        gtk4::prelude::GdkCairoContextExt::set_source_pixbuf(&ctx, pixbuf, 0.0, 0.0);
        let _ = ctx.paint();

        for shape in imp.shapes.borrow().iter() {
            shape.draw(&ctx);
        }

        let _ = pixbuf;

        let mut data = Vec::new();
        surface.write_to_png(&mut data).ok()?;

        Some(data)
    }

    fn screen_to_canvas(&self, x: f64, y: f64) -> (f64, f64) {
        let zoom = self.imp().zoom.get();
        (x / zoom, y / zoom)
    }

    fn update_content_size(&self) {
        let imp = self.imp();
        let zoom = imp.zoom.get();

        if let Some(ref pixbuf) = *imp.image.borrow() {
            let width = (pixbuf.width() as f64 * zoom) as i32;
            let height = (pixbuf.height() as f64 * zoom) as i32;
            self.set_content_width(width);
            self.set_content_height(height);
        }
    }

    pub fn zoom_in(&self) {
        let imp = self.imp();
        let current = imp.zoom.get();
        let new_zoom = (current * 1.25).min(5.0);
        imp.zoom.set(new_zoom);
        self.update_content_size();
        self.queue_draw();
    }

    pub fn zoom_out(&self) {
        let imp = self.imp();
        let current = imp.zoom.get();
        let new_zoom = (current / 1.25).max(0.1);
        imp.zoom.set(new_zoom);
        self.update_content_size();
        self.queue_draw();
    }

    pub fn zoom_reset(&self) {
        self.imp().zoom.set(1.0);
        self.update_content_size();
        self.queue_draw();
    }

    pub fn pan(&self, delta_x: f64, delta_y: f64) {
        if let Some(parent) = self.parent() {
            if let Some(scrolled) = parent.parent() {
                if let Ok(scrolled_window) = scrolled.downcast::<gtk4::ScrolledWindow>() {
                    let h_adj = scrolled_window.hadjustment();
                    let v_adj = scrolled_window.vadjustment();

                    let new_h = h_adj.value() + delta_x;
                    let new_v = v_adj.value() + delta_y;

                    h_adj.set_value(new_h.clamp(h_adj.lower(), h_adj.upper() - h_adj.page_size()));
                    v_adj.set_value(new_v.clamp(v_adj.lower(), v_adj.upper() - v_adj.page_size()));
                }
            }
        }
    }

    pub fn get_zoom(&self) -> f64 {
        self.imp().zoom.get()
    }
}
