use gtk4::gdk_pixbuf::Pixbuf;
use gtk4::gio::{Cancellable, MemoryInputStream};
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::{glib, DrawingArea};
use std::cell::{Cell, RefCell};

use super::history::{History, UndoEntry};
use super::shapes::Shape;
use super::tools::{Tool, ToolType};

mod input;
mod render;
mod viewport;

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
        pub history: RefCell<History>,
        pub pending_modify: RefCell<Option<(usize, Shape)>>,
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
                history: RefCell::new(History::default()),
                pending_modify: RefCell::new(None),
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

    pub fn set_image(&self, data: &[u8]) {
        let bytes = glib::Bytes::from(data);
        let stream = MemoryInputStream::from_bytes(&bytes);

        if let Ok(pixbuf) = Pixbuf::from_stream(&stream, Cancellable::NONE) {
            self.set_content_width(pixbuf.width());
            self.set_content_height(pixbuf.height());
            *self.imp().image.borrow_mut() = Some(pixbuf);
            self.imp().shapes.borrow_mut().clear();
            self.imp().history.borrow_mut().clear();
            *self.imp().pending_modify.borrow_mut() = None;
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

    fn push_undo(&self, entry: UndoEntry) {
        self.imp().history.borrow_mut().push(entry);
    }

    /// Append a finished shape to the document and record it as undoable.
    fn commit_shape(&self, shape: Shape) {
        let idx = {
            let mut shapes = self.imp().shapes.borrow_mut();
            shapes.push(shape.clone());
            shapes.len() - 1
        };
        self.push_undo(UndoEntry::Add { idx, shape });
    }

    pub fn undo(&self) {
        let imp = self.imp();
        imp.history.borrow_mut().undo(&mut imp.shapes.borrow_mut());
        *imp.pending_modify.borrow_mut() = None;
        imp.selected_index.set(None);
        self.queue_draw();
    }

    pub fn redo(&self) {
        let imp = self.imp();
        imp.history.borrow_mut().redo(&mut imp.shapes.borrow_mut());
        *imp.pending_modify.borrow_mut() = None;
        imp.selected_index.set(None);
        self.queue_draw();
    }

    pub fn delete_selected(&self) {
        let imp = self.imp();
        if let Some(idx) = imp.selected_index.get() {
            let removed = {
                let mut shapes = imp.shapes.borrow_mut();
                if idx < shapes.len() {
                    Some(shapes.remove(idx))
                } else {
                    None
                }
            };

            if let Some(shape) = removed {
                self.push_undo(UndoEntry::Remove { idx, shape });
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
}
