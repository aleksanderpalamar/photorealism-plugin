use super::action::Action;
use super::field::Field;
use super::layout::Layout;
use super::pointer::Pointer;
use super::row::Row;

pub fn flip(row: Row, pointer: Pointer) -> Action {
    if !row.switch_box().contains(pointer.position()) {
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
