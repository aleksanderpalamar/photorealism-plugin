use super::draft::Changes;
use super::draw_row::push_row;
use super::geometry::{Point, Rect};
use super::layout::Layout;
use super::palette::{self, Color};
use super::primitive::Primitive;
use super::text::{push_centered, push_text};
use crate::settings::Settings;

const ARROW_ROWS: usize = 10;

pub fn build(
    layout: &Layout,
    settings: &Settings,
    title: &str,
    changes: Changes,
    pointer: Option<Point>,
) -> Vec<Primitive> {
    let mut primitives = vec![
        Primitive::Rectangle {
            rect: layout.panel,
            color: palette::PANEL,
        },
        Primitive::Rectangle {
            rect: layout.title,
            color: palette::TITLE_BAR,
        },
    ];
    push_text(
        &mut primitives,
        layout.title_text,
        layout.title_text.x,
        layout.scale,
        title,
        palette::TITLE_TEXT,
    );
    for row in &layout.rows {
        push_row(&mut primitives, row, settings, layout.scale);
    }
    push_footer(&mut primitives, layout, changes);
    if let Some(position) = pointer {
        push_pointer(&mut primitives, position, layout.scale);
    }
    primitives
}

fn push_footer(into: &mut Vec<Primitive>, layout: &Layout, changes: Changes) {
    into.push(Primitive::Rectangle {
        rect: layout.save,
        color: palette::save(changes),
    });
    push_centered(
        into,
        layout.save,
        layout.scale,
        "Salvar no cfg",
        palette::VALUE,
    );
    into.push(Primitive::Rectangle {
        rect: layout.discard,
        color: palette::TRACK,
    });
    push_centered(
        into,
        layout.discard,
        layout.scale,
        "Descartar",
        palette::LABEL,
    );
}

fn push_pointer(into: &mut Vec<Primitive>, position: Point, scale: f32) {
    push_arrow(into, position, scale, palette::POINTER_OUTLINE, scale / 2.0);
    push_arrow(into, position, scale, palette::POINTER, 0.0);
}

fn push_arrow(into: &mut Vec<Primitive>, position: Point, scale: f32, color: Color, grow: f32) {
    for row in 0..ARROW_ROWS {
        into.push(Primitive::Rectangle {
            rect: Rect {
                x: position.x - grow,
                y: position.y + row as f32 * scale - grow,
                width: (row + 1) as f32 * scale + grow * 2.0,
                height: scale + grow * 2.0,
            },
            color,
        });
    }
}

#[cfg(test)]
#[path = "draw_list_tests.rs"]
mod tests;
