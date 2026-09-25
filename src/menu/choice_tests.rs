use super::Choice;
use crate::menu::field::Field;
use crate::menu::geometry::Point;
use crate::menu::layout::Layout;
use crate::menu::row::Row;

const COUNT: usize = 3;

fn row() -> Row {
    Layout::build(Point { x: 24.0, y: 24.0 }, 2.0)
        .rows
        .iter()
        .find(|row| row.field == Field::LuminanceProfile)
        .copied()
        .expect("linha do perfil")
}

fn choice() -> Choice {
    Choice::build(&row(), COUNT)
}

fn center(rect: crate::menu::geometry::Rect) -> Point {
    Point {
        x: rect.x + rect.width / 2.0,
        y: rect.y + rect.height / 2.0,
    }
}

#[test]
fn the_closed_box_spans_from_the_track_to_the_value() {
    let choice = choice();

    assert_eq!(choice.closed.x, row().track.x);
    assert_eq!(choice.closed.right(), row().value.right());
}

#[test]
fn the_closed_box_fits_inside_its_row() {
    let choice = choice();

    assert!(choice.closed.y > row().bounds.y);
    assert!(choice.closed.bottom() < row().bounds.bottom());
}

#[test]
fn there_is_one_item_per_option() {
    assert_eq!(choice().items.len(), COUNT);
}

#[test]
fn the_list_opens_below_the_closed_box() {
    let choice = choice();

    assert!(choice.items[0].y >= choice.closed.bottom());
}

#[test]
fn the_items_stack_without_overlapping() {
    let choice = choice();

    for pair in choice.items.windows(2) {
        assert!(pair[0].bottom() <= pair[1].y);
    }
}

#[test]
fn every_item_keeps_the_width_of_the_closed_box() {
    let choice = choice();

    for item in &choice.items {
        assert_eq!(item.x, choice.closed.x);
        assert_eq!(item.width, choice.closed.width);
    }
}

#[test]
fn a_point_inside_an_item_finds_its_index() {
    let choice = choice();

    for index in 0..COUNT {
        assert_eq!(choice.item_at(center(choice.items[index])), Some(index));
    }
}

#[test]
fn a_point_outside_the_list_finds_nothing() {
    let choice = choice();

    assert_eq!(choice.item_at(center(choice.closed)), None);
    assert_eq!(choice.item_at(Point { x: 5.0, y: 5.0 }), None);
}

#[test]
fn the_list_covers_every_item() {
    let choice = choice();
    let list = choice.list();

    for item in &choice.items {
        assert!(item.y >= list.y);
        assert!(item.bottom() <= list.bottom());
    }
}
