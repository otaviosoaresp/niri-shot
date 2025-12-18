use std::f64::consts::PI;

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

impl Color {
    pub fn new(r: f64, g: f64, b: f64, a: f64) -> Self {
        Self { r, g, b, a }
    }

    #[allow(dead_code)]
    pub fn red() -> Self {
        Self::new(1.0, 0.0, 0.0, 1.0)
    }

    #[allow(dead_code)]
    pub fn green() -> Self {
        Self::new(0.0, 0.8, 0.0, 1.0)
    }

    #[allow(dead_code)]
    pub fn blue() -> Self {
        Self::new(0.0, 0.4, 1.0, 1.0)
    }

    #[allow(dead_code)]
    pub fn yellow() -> Self {
        Self::new(1.0, 0.8, 0.0, 1.0)
    }

    #[allow(dead_code)]
    pub fn black() -> Self {
        Self::new(0.0, 0.0, 0.0, 1.0)
    }

    #[allow(dead_code)]
    pub fn white() -> Self {
        Self::new(1.0, 1.0, 1.0, 1.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeType {
    Rectangle,
    Ellipse,
    Line,
    Arrow,
    FreeHand,
    Text,
    Blur,
    Highlight,
}

#[derive(Debug, Clone)]
pub struct Shape {
    pub shape_type: ShapeType,
    pub start_x: f64,
    pub start_y: f64,
    pub end_x: f64,
    pub end_y: f64,
    pub color: Color,
    pub stroke_width: f64,
    pub filled: bool,
    pub points: Vec<(f64, f64)>,
    pub text: String,
    pub font_size: f64,
    pub rotation: f64,
}

impl Default for Shape {
    fn default() -> Self {
        Self {
            shape_type: ShapeType::Rectangle,
            start_x: 0.0,
            start_y: 0.0,
            end_x: 0.0,
            end_y: 0.0,
            color: Color::new(1.0, 0.0, 0.0, 1.0),
            stroke_width: 3.0,
            filled: false,
            points: Vec::new(),
            text: String::new(),
            font_size: 20.0,
            rotation: 0.0,
        }
    }
}

impl Shape {
    pub fn draw(&self, ctx: &cairo::Context) {
        let has_rotation = self.rotation.abs() > 0.001;

        if has_rotation {
            let (cx, cy) = self.center();
            ctx.save().ok();
            ctx.translate(cx, cy);
            ctx.rotate(self.rotation);
            ctx.translate(-cx, -cy);
        }

        ctx.set_source_rgba(self.color.r, self.color.g, self.color.b, self.color.a);
        ctx.set_line_width(self.stroke_width);

        match self.shape_type {
            ShapeType::Rectangle => self.draw_rectangle(ctx),
            ShapeType::Ellipse => self.draw_ellipse(ctx),
            ShapeType::Line => self.draw_line(ctx),
            ShapeType::Arrow => self.draw_arrow(ctx),
            ShapeType::FreeHand => self.draw_freehand(ctx),
            ShapeType::Text => self.draw_text(ctx),
            ShapeType::Blur => self.draw_blur_placeholder(ctx),
            ShapeType::Highlight => self.draw_highlight(ctx),
        }

        if has_rotation {
            ctx.restore().ok();
        }
    }

    fn draw_rectangle(&self, ctx: &cairo::Context) {
        let x = self.start_x.min(self.end_x);
        let y = self.start_y.min(self.end_y);
        let width = (self.end_x - self.start_x).abs();
        let height = (self.end_y - self.start_y).abs();

        ctx.rectangle(x, y, width, height);

        if self.filled {
            let _ = ctx.fill();
        } else {
            let _ = ctx.stroke();
        }
    }

    fn draw_ellipse(&self, ctx: &cairo::Context) {
        let cx = (self.start_x + self.end_x) / 2.0;
        let cy = (self.start_y + self.end_y) / 2.0;
        let rx = (self.end_x - self.start_x).abs() / 2.0;
        let ry = (self.end_y - self.start_y).abs() / 2.0;

        if rx > 0.0 && ry > 0.0 {
            ctx.save().ok();
            ctx.translate(cx, cy);
            ctx.scale(rx, ry);
            ctx.arc(0.0, 0.0, 1.0, 0.0, 2.0 * PI);
            ctx.restore().ok();

            if self.filled {
                let _ = ctx.fill();
            } else {
                let _ = ctx.stroke();
            }
        }
    }

    fn draw_line(&self, ctx: &cairo::Context) {
        ctx.move_to(self.start_x, self.start_y);
        ctx.line_to(self.end_x, self.end_y);
        let _ = ctx.stroke();
    }

    fn draw_arrow(&self, ctx: &cairo::Context) {
        ctx.move_to(self.start_x, self.start_y);
        ctx.line_to(self.end_x, self.end_y);
        let _ = ctx.stroke();

        let arrow_length = 15.0;
        let arrow_angle = PI / 6.0;

        let angle = (self.end_y - self.start_y).atan2(self.end_x - self.start_x);

        let x1 = self.end_x - arrow_length * (angle - arrow_angle).cos();
        let y1 = self.end_y - arrow_length * (angle - arrow_angle).sin();
        let x2 = self.end_x - arrow_length * (angle + arrow_angle).cos();
        let y2 = self.end_y - arrow_length * (angle + arrow_angle).sin();

        ctx.move_to(self.end_x, self.end_y);
        ctx.line_to(x1, y1);
        let _ = ctx.stroke();

        ctx.move_to(self.end_x, self.end_y);
        ctx.line_to(x2, y2);
        let _ = ctx.stroke();
    }

    fn draw_freehand(&self, ctx: &cairo::Context) {
        if self.points.is_empty() {
            return;
        }

        ctx.move_to(self.points[0].0, self.points[0].1);
        for point in &self.points[1..] {
            ctx.line_to(point.0, point.1);
        }
        let _ = ctx.stroke();
    }

    fn draw_text(&self, ctx: &cairo::Context) {
        if self.text.is_empty() {
            return;
        }

        ctx.select_font_face("Sans", cairo::FontSlant::Normal, cairo::FontWeight::Bold);
        ctx.set_font_size(self.font_size);
        ctx.move_to(self.start_x, self.start_y + self.font_size);
        let _ = ctx.show_text(&self.text);
    }

    fn draw_blur_placeholder(&self, ctx: &cairo::Context) {
        let x = self.start_x.min(self.end_x);
        let y = self.start_y.min(self.end_y);
        let width = (self.end_x - self.start_x).abs();
        let height = (self.end_y - self.start_y).abs();

        if width < 1.0 || height < 1.0 {
            return;
        }

        let block_size = 10.0;

        ctx.save().ok();

        for row in 0..((height / block_size) as i32 + 1) {
            for col in 0..((width / block_size) as i32 + 1) {
                let bx = x + col as f64 * block_size;
                let by = y + row as f64 * block_size;

                let gray = if (row + col) % 2 == 0 { 0.3 } else { 0.5 };
                ctx.set_source_rgba(gray, gray, gray, 0.8);

                let bw = block_size.min(x + width - bx);
                let bh = block_size.min(y + height - by);

                ctx.rectangle(bx, by, bw, bh);
                let _ = ctx.fill();
            }
        }

        ctx.restore().ok();
    }

    fn draw_highlight(&self, ctx: &cairo::Context) {
        let x = self.start_x.min(self.end_x);
        let y = self.start_y.min(self.end_y);
        let width = (self.end_x - self.start_x).abs();
        let height = (self.end_y - self.start_y).abs();

        ctx.set_source_rgba(self.color.r, self.color.g, self.color.b, 0.3);
        ctx.rectangle(x, y, width, height);
        let _ = ctx.fill();
    }

    pub fn add_point(&mut self, x: f64, y: f64) {
        self.points.push((x, y));
    }

    pub fn bounds(&self) -> (f64, f64, f64, f64) {
        match self.shape_type {
            ShapeType::FreeHand => {
                if self.points.is_empty() {
                    return (self.start_x, self.start_y, self.start_x, self.start_y);
                }
                let mut min_x = f64::MAX;
                let mut min_y = f64::MAX;
                let mut max_x = f64::MIN;
                let mut max_y = f64::MIN;
                for (px, py) in &self.points {
                    min_x = min_x.min(*px);
                    min_y = min_y.min(*py);
                    max_x = max_x.max(*px);
                    max_y = max_y.max(*py);
                }
                (min_x, min_y, max_x, max_y)
            }
            ShapeType::Text => {
                let text_width = self.text.len() as f64 * self.font_size * 0.6;
                let text_height = self.font_size;
                (
                    self.start_x,
                    self.start_y,
                    self.start_x + text_width,
                    self.start_y + text_height,
                )
            }
            ShapeType::Line | ShapeType::Arrow => {
                let min_x = self.start_x.min(self.end_x);
                let min_y = self.start_y.min(self.end_y);
                let max_x = self.start_x.max(self.end_x);
                let max_y = self.start_y.max(self.end_y);
                (min_x, min_y, max_x, max_y)
            }
            _ => {
                let min_x = self.start_x.min(self.end_x);
                let min_y = self.start_y.min(self.end_y);
                let max_x = self.start_x.max(self.end_x);
                let max_y = self.start_y.max(self.end_y);
                (min_x, min_y, max_x, max_y)
            }
        }
    }

    pub fn center(&self) -> (f64, f64) {
        let (min_x, min_y, max_x, max_y) = self.bounds();
        ((min_x + max_x) / 2.0, (min_y + max_y) / 2.0)
    }

    pub fn contains_point(&self, x: f64, y: f64) -> bool {
        let tolerance = self.stroke_width.max(5.0);

        match self.shape_type {
            ShapeType::Rectangle | ShapeType::Blur | ShapeType::Highlight => {
                let (min_x, min_y, max_x, max_y) = self.bounds();
                x >= min_x - tolerance
                    && x <= max_x + tolerance
                    && y >= min_y - tolerance
                    && y <= max_y + tolerance
            }
            ShapeType::Ellipse => {
                let cx = (self.start_x + self.end_x) / 2.0;
                let cy = (self.start_y + self.end_y) / 2.0;
                let rx = (self.end_x - self.start_x).abs() / 2.0 + tolerance;
                let ry = (self.end_y - self.start_y).abs() / 2.0 + tolerance;
                if rx < 1.0 || ry < 1.0 {
                    return false;
                }
                let dx = x - cx;
                let dy = y - cy;
                (dx * dx) / (rx * rx) + (dy * dy) / (ry * ry) <= 1.0
            }
            ShapeType::Line | ShapeType::Arrow => {
                let dist = Self::point_to_line_distance(
                    x,
                    y,
                    self.start_x,
                    self.start_y,
                    self.end_x,
                    self.end_y,
                );
                dist <= tolerance
            }
            ShapeType::FreeHand => {
                for point in &self.points {
                    let dx = x - point.0;
                    let dy = y - point.1;
                    if dx * dx + dy * dy <= tolerance * tolerance {
                        return true;
                    }
                }
                false
            }
            ShapeType::Text => {
                let (min_x, min_y, max_x, max_y) = self.bounds();
                x >= min_x && x <= max_x && y >= min_y && y <= max_y + self.font_size
            }
        }
    }

    fn point_to_line_distance(px: f64, py: f64, x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
        let dx = x2 - x1;
        let dy = y2 - y1;
        let len_sq = dx * dx + dy * dy;

        if len_sq < 0.0001 {
            let ddx = px - x1;
            let ddy = py - y1;
            return (ddx * ddx + ddy * ddy).sqrt();
        }

        let t = ((px - x1) * dx + (py - y1) * dy) / len_sq;
        let t = t.clamp(0.0, 1.0);

        let proj_x = x1 + t * dx;
        let proj_y = y1 + t * dy;

        let ddx = px - proj_x;
        let ddy = py - proj_y;
        (ddx * ddx + ddy * ddy).sqrt()
    }

    pub fn translate(&mut self, dx: f64, dy: f64) {
        self.start_x += dx;
        self.start_y += dy;
        self.end_x += dx;
        self.end_y += dy;

        for point in &mut self.points {
            point.0 += dx;
            point.1 += dy;
        }
    }

    pub fn set_rotation(&mut self, angle: f64) {
        self.rotation = angle;
    }

    pub fn resize_corner(&mut self, corner: u8, new_x: f64, new_y: f64) {
        match self.shape_type {
            ShapeType::FreeHand => {
                self.resize_freehand_proportional(corner, new_x, new_y);
            }
            ShapeType::Text => {
                self.resize_text(corner, new_x, new_y);
            }
            ShapeType::Line | ShapeType::Arrow => {
                self.resize_line(corner, new_x, new_y);
            }
            _ => {
                self.resize_rect_based(corner, new_x, new_y);
            }
        }
    }

    fn resize_rect_based(&mut self, corner: u8, new_x: f64, new_y: f64) {
        match corner {
            0 => {
                self.start_x = new_x;
                self.start_y = new_y;
            }
            1 => {
                self.end_x = new_x;
                self.start_y = new_y;
            }
            2 => {
                self.start_x = new_x;
                self.end_y = new_y;
            }
            3 => {
                self.end_x = new_x;
                self.end_y = new_y;
            }
            _ => {}
        }
    }

    fn resize_line(&mut self, corner: u8, new_x: f64, new_y: f64) {
        match corner {
            0 | 2 => {
                self.start_x = new_x;
                self.start_y = new_y;
            }
            1 | 3 => {
                self.end_x = new_x;
                self.end_y = new_y;
            }
            _ => {}
        }
    }

    fn resize_freehand_proportional(&mut self, corner: u8, new_x: f64, new_y: f64) {
        let (old_min_x, old_min_y, old_max_x, old_max_y) = self.bounds();
        let old_width = (old_max_x - old_min_x).max(1.0);
        let old_height = (old_max_y - old_min_y).max(1.0);

        let (new_min_x, new_min_y, new_max_x, new_max_y) = match corner {
            0 => (new_x, new_y, old_max_x, old_max_y),
            1 => (old_min_x, new_y, new_x, old_max_y),
            2 => (new_x, old_min_y, old_max_x, new_y),
            3 => (old_min_x, old_min_y, new_x, new_y),
            _ => return,
        };

        let new_width = (new_max_x - new_min_x).max(1.0);
        let new_height = (new_max_y - new_min_y).max(1.0);

        let scale_x = new_width / old_width;
        let scale_y = new_height / old_height;

        for point in &mut self.points {
            point.0 = new_min_x + (point.0 - old_min_x) * scale_x;
            point.1 = new_min_y + (point.1 - old_min_y) * scale_y;
        }

        self.start_x = new_min_x;
        self.start_y = new_min_y;
        self.end_x = new_max_x;
        self.end_y = new_max_y;
    }

    fn resize_text(&mut self, corner: u8, new_x: f64, new_y: f64) {
        let (_, old_min_y, _, old_max_y) = self.bounds();
        let old_height = (old_max_y - old_min_y).max(1.0);

        let new_height = match corner {
            0 | 1 => old_max_y - new_y,
            2 | 3 => new_y - old_min_y,
            _ => return,
        };

        let scale = (new_height / old_height).clamp(0.5, 5.0);
        self.font_size = (self.font_size * scale).clamp(8.0, 100.0);

        match corner {
            0 => {
                self.start_x = new_x;
                self.start_y = new_y;
            }
            2 => {
                self.start_x = new_x;
            }
            _ => {}
        }
    }
}
