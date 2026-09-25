use super::choice::Choice;
use super::field::{Control, Field};
use super::geometry::Point;
use super::layout::Layout;
use super::palette;
use super::primitive::Primitive;
use super::text::push_option;

pub fn push_open(into: &mut Vec<Primitive>, layout: &Layout, field: Field, pointer: Option<Point>) {
    let Control::Choice(states) = field.control() else {
        return;
    };
    let Some(row) = layout.rows.iter().find(|row| row.field == field) else {
        return;
    };
    let choice = Choice::build(row, states);
    into.push(Primitive::Rectangle {
        rect: choice.list(),
        color: palette::POPUP,
    });
    for (index, item) in choice.items.iter().enumerate() {
        if pointer.is_some_and(|point| item.contains(point)) {
            into.push(Primitive::Rectangle {
                rect: *item,
                color: palette::HIGHLIGHT,
            });
        }
        push_option(into, *item, field.choice_label(index), layout.scale);
    }
}

#[cfg(test)]
#[path = "draw_choice_tests.rs"]
mod tests;
