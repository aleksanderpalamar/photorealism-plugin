use super::font::{self, CELL_HEIGHT, CELL_WIDTH};
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

pub fn push_centered(into: &mut Vec<Primitive>, area: Rect, scale: f32, text: &str, color: Color) {
    let width = font::text_width(text) as f32 * scale;
    let row = area.centered_row(CELL_HEIGHT as f32 * scale);
    push_text(
        into,
        row,
        area.x + (area.width - width) / 2.0,
        scale,
        text,
        color,
    );
}

pub fn push_option(into: &mut Vec<Primitive>, area: Rect, text: &str, scale: f32) {
    let row = area.centered_row(CELL_HEIGHT as f32 * scale);
    push_text(
        into,
        row,
        area.x + CELL_WIDTH as f32 * scale / 2.0,
        scale,
        text,
        super::palette::VALUE,
    );
}
