use gtk4::prelude::*;
use gtk4::subclass::prelude::*;

use super::EditorCanvas;

impl EditorCanvas {
    pub(super) fn screen_to_canvas(&self, x: f64, y: f64) -> (f64, f64) {
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
