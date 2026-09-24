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
    assert!(last.bounds.bottom() <= layout.save.y);
    assert!(layout.rows[0].bounds.y >= layout.title.bottom());
}

#[test]
fn every_row_keeps_label_track_and_value_inside_the_panel() {
    let layout = layout(2.0);

    for row in &layout.rows {
        assert!(row.label.x >= layout.panel.x);
        assert!(row.label.right() <= row.track.x);
        assert!(row.track.right() <= row.value.x);
        assert!(row.value.right() <= row.reset.x);
        assert!(row.reset.right() <= layout.panel.right());
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

#[test]
fn the_footer_keeps_both_buttons_inside_the_panel() {
    let layout = layout(2.0);

    assert!(layout.save.x >= layout.panel.x);
    assert!(layout.save.right() <= layout.discard.x);
    assert!(layout.discard.right() <= layout.panel.right());
    assert!(layout.save.bottom() <= layout.panel.bottom());
}

#[test]
fn the_footer_buttons_share_the_same_width() {
    let layout = layout(2.0);

    assert_eq!(layout.save.width, layout.discard.width);
    assert!(layout.save.width > 0.0);
}

#[test]
fn the_footer_sits_below_every_row() {
    let layout = layout(2.0);
    let last = layout.rows.last().expect("linhas");

    assert!(layout.save.y >= last.bounds.bottom());
}
