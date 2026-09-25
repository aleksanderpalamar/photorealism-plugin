use super::action::Action;
use super::choice::Choice;
use super::field::{Control, Field};
use super::geometry::Rect;
use super::layout::Layout;
use super::pointer::Pointer;
use super::row::Row;

pub fn flip(row: Row, area: Rect, pointer: Pointer) -> Action {
    if !area.contains(pointer.position()) {
        return Action::Idle;
    }
    Action::Flip(row.field)
}

pub fn slide(layout: &Layout, field: Field, pointer: Pointer) -> Action {
    let Some(row) = layout.rows.iter().find(|row| row.field == field) else {
        return Action::Idle;
    };
    Action::Slide(field, row.track.fraction_at(pointer.position().x))
}

pub fn row_under(layout: &Layout, pointer: Pointer) -> Option<Row> {
    layout
        .rows
        .iter()
        .find(|row| row.bounds.contains(pointer.position()))
        .copied()
}

pub fn pick(layout: &Layout, field: Field, pointer: Pointer) -> Action {
    let Control::Choice(count) = field.control() else {
        return Action::Idle;
    };
    let Some(row) = layout.rows.iter().find(|row| row.field == field) else {
        return Action::Idle;
    };
    let Some(index) = Choice::build(row, count).item_at(pointer.position()) else {
        return Action::Idle;
    };
    Action::Select(field, index)
}
