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
const VALUE_WIDTH: f32 = 60.0;
const GAP: f32 = 6.0;

#[derive(Clone, Debug, PartialEq)]
pub struct Layout {
    pub scale: f32,
    pub panel: Rect,
    pub title: Rect,
    pub title_text: Rect,
    pub rows: Vec<Row>,
}

impl Layout {
    pub fn build(origin: Point, scale: f32) -> Self {
        let content = LABEL_WIDTH + GAP + TRACK_WIDTH + GAP + VALUE_WIDTH;
        let panel = Rect {
            x: origin.x,
            y: origin.y,
            width: (PADDING * 2.0 + content) * scale,
            height: (TITLE_HEIGHT + ROW_HEIGHT * Field::ALL.len() as f32 + PADDING) * scale,
        };
        let title = Rect {
            x: panel.x,
            y: panel.y,
            width: panel.width,
            height: TITLE_HEIGHT * scale,
        };
        Self {
            scale,
            panel,
            title,
            title_text: inset(title, PADDING * scale).centered_row(CELL_HEIGHT as f32 * scale),
            rows: rows(panel, title.bottom(), scale),
        }
    }
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
    Row {
        field,
        bounds,
        label: label.centered_row(CELL_HEIGHT as f32 * scale),
        track,
        value: Rect {
            x: track.right() + GAP * scale,
            y: bounds.y,
            width: VALUE_WIDTH * scale,
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
mod tests {
    use super::{Layout, Point};
    use crate::menu::field::Field;

    fn layout(scale: f32) -> Layout {
        Layout::build(Point { x: 24.0, y: 24.0 }, scale)
    }

    #[test]
    fn there_is_one_row_per_field_in_order() {
        let fields: Vec<Field> = layout(2.0).rows.iter().map(|row| row.field).collect();

        assert_eq!(fields, Field::ALL.to_vec());
    }

    #[test]
    fn the_panel_starts_at_the_requested_origin() {
        let layout = layout(2.0);

        assert_eq!(layout.panel.x, 24.0);
        assert_eq!(layout.panel.y, 24.0);
        assert_eq!(layout.title.y, layout.panel.y);
    }

    #[test]
    fn the_scale_multiplies_every_dimension() {
        let single = layout(1.0);
        let double = layout(2.0);

        assert_eq!(double.panel.width, single.panel.width * 2.0);
        assert_eq!(double.title.height, single.title.height * 2.0);
    }

    #[test]
    fn rows_stack_without_overlapping_inside_the_panel() {
        let layout = layout(2.0);

        for pair in layout.rows.windows(2) {
            assert_eq!(pair[0].bounds.bottom(), pair[1].bounds.y);
        }
        let last = layout.rows.last().expect("linhas");
        assert!(last.bounds.bottom() <= layout.panel.bottom());
        assert!(layout.rows[0].bounds.y >= layout.title.bottom());
    }

    #[test]
    fn every_row_keeps_label_track_and_value_inside_the_panel() {
        let layout = layout(2.0);

        for row in &layout.rows {
            assert!(row.label.x >= layout.panel.x);
            assert!(row.label.right() <= row.track.x);
            assert!(row.track.right() <= row.value.x);
            assert!(row.value.right() <= layout.panel.right());
        }
    }

    #[test]
    fn the_handle_travels_the_track_without_leaving_it() {
        let layout = layout(2.0);
        let row = layout.rows[1];

        assert_eq!(row.handle(0.0).x, row.track.x);
        assert_eq!(row.handle(1.0).right(), row.track.right());
        assert!(row.handle(2.0).right() <= row.track.right());
        assert!(row.handle(-1.0).x >= row.track.x);
    }

    #[test]
    fn the_switch_box_sits_at_the_start_of_the_track() {
        let layout = layout(2.0);
        let row = layout.rows[0];
        let box_rect = row.switch_box();

        assert_eq!(box_rect.x, row.track.x);
        assert!(box_rect.height <= row.bounds.height);
    }
}
