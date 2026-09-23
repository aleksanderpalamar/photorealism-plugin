use super::field::Control;
use super::font;
use super::geometry::Rect;
use super::palette;
use super::primitive::Primitive;
use super::row::Row;
use super::text::{push_centered, push_text};
use crate::settings::Settings;

pub fn push_row(into: &mut Vec<Primitive>, row: &Row, settings: &Settings, scale: f32) {
    let editable = row.field.is_editable(settings);
    push_text(
        into,
        row.label,
        row.label.x,
        scale,
        row.field.label(),
        palette::label(editable),
    );
    match row.field.control() {
        Control::Switch => push_switch(into, row, settings),
        Control::Slider(_) => push_slider(into, row, settings, editable),
    }
    let text = row.field.text(settings);
    let width = font::text_width(&text) as f32 * scale;
    push_text(
        into,
        row.value,
        row.value.right() - width,
        scale,
        &text,
        palette::value(editable),
    );
    into.push(Primitive::Rectangle {
        rect: row.reset,
        color: palette::TRACK,
    });
    push_centered(into, row.reset, scale, "R", palette::LABEL);
}

fn push_slider(into: &mut Vec<Primitive>, row: &Row, settings: &Settings, editable: bool) {
    let fraction = row.field.fraction(settings);
    into.push(Primitive::Rectangle {
        rect: row.track,
        color: palette::TRACK,
    });
    into.push(Primitive::Rectangle {
        rect: Rect {
            width: row.track.width * fraction,
            ..row.track
        },
        color: palette::fill(editable),
    });
    into.push(Primitive::Rectangle {
        rect: row.handle(fraction),
        color: palette::handle(editable),
    });
}

fn push_switch(into: &mut Vec<Primitive>, row: &Row, settings: &Settings) {
    let bounds = row.switch_box();
    into.push(Primitive::Rectangle {
        rect: bounds,
        color: palette::TRACK,
    });
    if !row.field.switch(settings) {
        return;
    }
    let inset = bounds.width / 4.0;
    into.push(Primitive::Rectangle {
        rect: Rect {
            x: bounds.x + inset,
            y: bounds.y + inset,
            width: bounds.width - inset * 2.0,
            height: bounds.height - inset * 2.0,
        },
        color: palette::FILL,
    });
}
