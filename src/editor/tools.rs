use super::shapes::{Shape, ShapeType, Color};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolType {
    Select,
    Rectangle,
    Circle,
    Line,
    Arrow,
    FreeHand,
    Text,
    Blur,
    Highlight,
}

impl Default for ToolType {
    fn default() -> Self {
        Self::Select
    }
}

pub struct Tool {
    pub tool_type: ToolType,
    pub color: Color,
    pub stroke_width: f64,
    pub filled: bool,
    pub font_size: f64,
}

impl Default for Tool {
    fn default() -> Self {
        Self {
            tool_type: ToolType::Select,
            color: Color::new(1.0, 0.0, 0.0, 1.0),
            stroke_width: 3.0,
            filled: false,
            font_size: 20.0,
        }
    }
}

impl Tool {
    pub fn create_shape(&self, start_x: f64, start_y: f64, end_x: f64, end_y: f64) -> Option<Shape> {
        let shape_type = match self.tool_type {
            ToolType::Rectangle => ShapeType::Rectangle,
            ToolType::Circle => ShapeType::Ellipse,
            ToolType::Line => ShapeType::Line,
            ToolType::Arrow => ShapeType::Arrow,
            ToolType::Blur => ShapeType::Blur,
            ToolType::Highlight => ShapeType::Highlight,
            _ => return None,
        };

        Some(Shape {
            shape_type,
            start_x,
            start_y,
            end_x,
            end_y,
            color: self.color,
            stroke_width: self.stroke_width,
            filled: self.filled,
            font_size: self.font_size,
            ..Default::default()
        })
    }

    pub fn create_text_shape(&self, x: f64, y: f64, text: String) -> Shape {
        Shape {
            shape_type: ShapeType::Text,
            start_x: x,
            start_y: y,
            end_x: x,
            end_y: y,
            color: self.color,
            stroke_width: self.stroke_width,
            filled: false,
            font_size: self.font_size,
            text,
            ..Default::default()
        }
    }
}
