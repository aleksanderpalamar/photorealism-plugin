use super::choice::Choice;
use super::field::Control;
use super::font;
use super::geometry::Rect;
use super::palette;
use super::primitive::Primitive;
use super::row::Row;
use super::text::{push_centered, push_option, push_text};
use crate::settings::Settings;

const CARET_ROWS: usize = 4;

pub fn push_row(
    into: &mut Vec<Primitive>,
    row: &Row,
    settings: &Settings,
    resolved_peak: f32,
    scale: f32,
) {
    let editable = row.field.is_editable(settings);
    push_text(
        into,
        row.label,
        row.label.x,
        scale,
        row.field.label(),
        palette::label(editable),
    );
    let control = row.field.control();
    match control {
        Control::Switch => push_switch(into, row, settings),
        Control::Choice(states) => push_closed(into, row, settings, states, scale),
        Control::Slider(_) => push_slider(into, row, settings, resolved_peak, editable),
    }
    if let Control::Choice(_) = control {
        push_reset(into, row, scale);
        return;
    }
    let text = row.field.text(settings, resolved_peak);
    let width = font::text_width(&text) as f32 * scale;
    push_text(
        into,
        row.value,
        row.value.right() - width,
        scale,
        &text,
        palette::value(editable),
    );
    push_reset(into, row, scale);
}

fn push_reset(into: &mut Vec<Primitive>, row: &Row, scale: f32) {
    into.push(Primitive::Rectangle {
        rect: row.reset,
        color: palette::TRACK,
    });
    push_centered(into, row.reset, scale, "R", palette::LABEL);
}

fn push_slider(
    into: &mut Vec<Primitive>,
    row: &Row,
    settings: &Settings,
    resolved_peak: f32,
    editable: bool,
) {
    let fraction = row.field.fraction(settings, resolved_peak);
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

fn push_closed(
    into: &mut Vec<Primitive>,
    row: &Row,
    settings: &Settings,
    states: usize,
    scale: f32,
) {
    let closed = Choice::build(row, states).closed;
    into.push(Primitive::Rectangle {
        rect: closed,
        color: palette::TRACK,
    });
    push_option(
        into,
        closed,
        row.field.choice_label(row.field.choice(settings)),
        scale,
    );
    push_caret(into, closed, scale);
}

fn push_caret(into: &mut Vec<Primitive>, closed: Rect, scale: f32) {
    let width = CARET_ROWS as f32 * 2.0 * scale;
    let left = closed.right() - width - scale * 2.0;
    let top = closed.y + (closed.height - CARET_ROWS as f32 * scale) / 2.0;
    for step in 0..CARET_ROWS {
        let inset = step as f32 * scale;
        into.push(Primitive::Rectangle {
            rect: Rect {
                x: left + inset,
                y: top + inset,
                width: width - inset * 2.0,
                height: scale,
            },
            color: palette::LABEL,
        });
    }
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

#[cfg(test)]
#[path = "draw_row_tests.rs"]
mod tests;
