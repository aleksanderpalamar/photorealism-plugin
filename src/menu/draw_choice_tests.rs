use super::push_open;
use crate::menu::choice::Choice;
use crate::menu::field::Field;
use crate::menu::geometry::{Point, Rect};
use crate::menu::layout::Layout;
use crate::menu::palette;
use crate::menu::primitive::Primitive;
use crate::settings::LuminanceProfile;

fn layout() -> Layout {
    Layout::build(Point { x: 24.0, y: 24.0 }, 2.0)
}

fn items() -> Vec<Rect> {
    let binding = layout();
    let row = binding
        .rows
        .iter()
        .find(|row| row.field == Field::LuminanceProfile)
        .expect("linha do perfil");
    Choice::build(row, LuminanceProfile::COUNT).items
}

fn drawn(pointer: Option<Point>) -> Vec<Primitive> {
    let mut primitives = Vec::new();
    push_open(&mut primitives, &layout(), Field::LuminanceProfile, pointer);
    primitives
}

fn rectangles(primitives: &[Primitive], color: palette::Color) -> usize {
    primitives
        .iter()
        .filter(|primitive| match primitive {
            Primitive::Rectangle { color: found, .. } => *found == color,
            Primitive::Glyph { .. } => false,
        })
        .count()
}

fn glyphs(primitives: &[Primitive]) -> usize {
    primitives
        .iter()
        .filter(|primitive| matches!(primitive, Primitive::Glyph { .. }))
        .count()
}

#[test]
fn the_list_draws_one_opaque_backdrop() {
    let primitives = drawn(None);

    assert_eq!(rectangles(&primitives, palette::POPUP), 1);
    assert_eq!(palette::POPUP.alpha, 1.0);
}

#[test]
fn the_backdrop_covers_every_item() {
    let primitives = drawn(None);
    let Some(Primitive::Rectangle { rect: backdrop, .. }) = primitives.first() else {
        panic!("o fundo da lista precisa vir primeiro");
    };

    for item in items() {
        assert!(item.y >= backdrop.y);
        assert!(item.bottom() <= backdrop.bottom());
    }
}

#[test]
fn every_option_is_written() {
    let expected: usize = LuminanceProfile::ALL
        .into_iter()
        .map(|profile| profile.label().len())
        .sum();

    assert_eq!(glyphs(&drawn(None)), expected);
}

#[test]
fn nothing_is_highlighted_without_a_pointer_over_it() {
    assert_eq!(rectangles(&drawn(None), palette::HIGHLIGHT), 0);
}

#[test]
fn the_item_under_the_pointer_is_highlighted() {
    for item in items() {
        let point = Point {
            x: item.x + item.width / 2.0,
            y: item.y + item.height / 2.0,
        };

        assert_eq!(rectangles(&drawn(Some(point)), palette::HIGHLIGHT), 1);
    }
}

#[test]
fn a_field_without_a_list_draws_nothing() {
    let mut primitives = Vec::new();
    push_open(&mut primitives, &layout(), Field::Exposure, None);

    assert!(primitives.is_empty());
}
