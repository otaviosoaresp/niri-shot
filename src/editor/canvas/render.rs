use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use std::f64::consts::PI;

use super::{EditorCanvas, HandleType};
use crate::editor::shapes::Shape;

impl EditorCanvas {
    pub(super) fn setup_drawing(&self) {
        self.set_draw_func(|widget, ctx, width, height| {
            let canvas = widget.downcast_ref::<EditorCanvas>().unwrap();
            canvas.draw(ctx, width, height);
        });
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
}
