use super::font::Cell;
use super::geometry::Rect;
use super::palette::Color;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Primitive {
    Rectangle {
        rect: Rect,
        color: Color,
    },
    Glyph {
        rect: Rect,
        cell: Cell,
        color: Color,
    },
}
