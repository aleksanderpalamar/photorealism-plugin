use super::field::Field;
use super::font::CELL_HEIGHT;
use super::geometry::{Point, Rect};
use super::row::Row;

const PADDING: f32 = 6.0;
const ROW_HEIGHT: f32 = 12.0;
const TITLE_HEIGHT: f32 = 16.0;
const LABEL_WIDTH: f32 = 132.0;
const TRACK_WIDTH: f32 = 96.0;
const TRACK_HEIGHT: f32 = 3.0;
const VALUE_WIDTH: f32 = 42.0;
const RESET_WIDTH: f32 = 10.0;
const FOOTER_HEIGHT: f32 = 16.0;
const GAP: f32 = 6.0;

#[derive(Clone, Debug, PartialEq)]
pub struct Layout {
    pub scale: f32,
    pub panel: Rect,
    pub title: Rect,
    pub title_text: Rect,
    pub rows: Vec<Row>,
    pub save: Rect,
    pub discard: Rect,
}

impl Layout {
    pub fn build(origin: Point, scale: f32) -> Self {
        let content = LABEL_WIDTH + GAP + TRACK_WIDTH + GAP + VALUE_WIDTH + GAP + RESET_WIDTH;
        let panel = Rect {
            x: origin.x,
            y: origin.y,
            width: (PADDING * 2.0 + content) * scale,
            height: (TITLE_HEIGHT + ROW_HEIGHT * Field::ALL.len() as f32 + FOOTER_HEIGHT) * scale,
        };
        let title = Rect {
            x: panel.x,
            y: panel.y,
            width: panel.width,
            height: TITLE_HEIGHT * scale,
        };
        let rows = rows(panel, title.bottom(), scale);
        let (save, discard) = footer(panel, scale);
        Self {
            scale,
            panel,
            title,
            title_text: inset(title, PADDING * scale).centered_row(CELL_HEIGHT as f32 * scale),
            rows,
            save,
            discard,
        }
    }
}

fn footer(panel: Rect, scale: f32) -> (Rect, Rect) {
    let height = (FOOTER_HEIGHT - PADDING) * scale;
    let available = panel.width - PADDING * 3.0 * scale;
    let width = available / 2.0;
    let top = panel.bottom() - height - PADDING * scale / 2.0;
    let save = Rect {
        x: panel.x + PADDING * scale,
        y: top,
        width,
        height,
    };
    let discard = Rect {
        x: save.right() + PADDING * scale,
        y: top,
        width,
        height,
    };
    (save, discard)
}

fn rows(panel: Rect, top: f32, scale: f32) -> Vec<Row> {
    let left = panel.x + PADDING * scale;
    Field::ALL
        .into_iter()
        .enumerate()
        .map(|(index, field)| {
            let bounds = Rect {
                x: left,
                y: top + ROW_HEIGHT * scale * index as f32,
                width: panel.width - PADDING * 2.0 * scale,
                height: ROW_HEIGHT * scale,
            };
            row(field, bounds, scale)
        })
        .collect()
}

fn row(field: Field, bounds: Rect, scale: f32) -> Row {
    let label = Rect {
        x: bounds.x,
        y: bounds.y,
        width: LABEL_WIDTH * scale,
        height: bounds.height,
    };
    let track = Rect {
        x: label.right() + GAP * scale,
        y: bounds.y,
        width: TRACK_WIDTH * scale,
        height: bounds.height,
    }
    .centered_row(TRACK_HEIGHT * scale);
    let value = Rect {
        x: track.right() + GAP * scale,
        y: bounds.y,
        width: VALUE_WIDTH * scale,
        height: bounds.height,
    };
    Row {
        field,
        bounds,
        label: label.centered_row(CELL_HEIGHT as f32 * scale),
        track,
        value: value.centered_row(CELL_HEIGHT as f32 * scale),
        reset: Rect {
            x: value.right() + GAP * scale,
            y: bounds.y,
            width: RESET_WIDTH * scale,
            height: bounds.height,
        }
        .centered_row(CELL_HEIGHT as f32 * scale),
    }
}

fn inset(rect: Rect, amount: f32) -> Rect {
    Rect {
        x: rect.x + amount,
        y: rect.y,
        width: rect.width - amount * 2.0,
        height: rect.height,
    }
}

#[cfg(test)]
#[path = "layout_tests.rs"]
mod tests;
