use super::font::{self, CELL_WIDTH};
use super::geometry::Rect;
use super::palette::Color;
use super::primitive::Primitive;

pub fn push_text(
    into: &mut Vec<Primitive>,
    area: Rect,
    start: f32,
    scale: f32,
    text: &str,
    color: Color,
) {
    let advance = CELL_WIDTH as f32 * scale;
    for (index, character) in text.chars().enumerate() {
        let Some(cell) = font::cell_of(character) else {
            continue;
        };
        into.push(Primitive::Glyph {
            rect: Rect {
                x: start + advance * index as f32,
                y: area.y,
                width: advance,
                height: area.height,
            },
            cell,
            color,
        });
    }
}
