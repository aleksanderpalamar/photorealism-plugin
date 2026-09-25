use super::{CARET_ROWS, push_row};
use crate::menu::choice::Choice;
use crate::menu::field::Field;
use crate::menu::geometry::Point;
use crate::menu::layout::Layout;
use crate::menu::primitive::Primitive;
use crate::menu::row::Row;
use crate::settings::{LuminanceProfile, Settings};

const PEAK: f32 = 1000.0;
const SCALE: f32 = 2.0;

fn profile_row() -> Row {
    Layout::build(Point { x: 24.0, y: 24.0 }, SCALE)
        .rows
        .iter()
        .find(|row| row.field == Field::LuminanceProfile)
        .copied()
        .expect("linha do perfil")
}

fn drawn(profile: LuminanceProfile) -> Vec<Primitive> {
    let settings = Settings {
        luminance_profile: profile,
        ..Settings::default()
    };
    let mut primitives = Vec::new();
    push_row(&mut primitives, &profile_row(), &settings, PEAK, SCALE);
    primitives
}

fn glyphs(primitives: &[Primitive]) -> usize {
    primitives
        .iter()
        .filter(|primitive| matches!(primitive, Primitive::Glyph { .. }))
        .count()
}

#[test]
fn the_closed_box_is_drawn_once() {
    let closed = Choice::build(&profile_row(), LuminanceProfile::COUNT).closed;

    let boxes = drawn(LuminanceProfile::Neutral)
        .iter()
        .filter(|primitive| match primitive {
            Primitive::Rectangle { rect, .. } => *rect == closed,
            Primitive::Glyph { .. } => false,
        })
        .count();

    assert_eq!(boxes, 1);
}

#[test]
fn the_caret_sits_inside_the_right_half_of_the_box() {
    let closed = Choice::build(&profile_row(), LuminanceProfile::COUNT).closed;
    let middle = closed.x + closed.width / 2.0;

    let caret = drawn(LuminanceProfile::Neutral)
        .iter()
        .filter(|primitive| match primitive {
            Primitive::Rectangle { rect, .. } => rect.x > middle && rect.right() <= closed.right(),
            Primitive::Glyph { .. } => false,
        })
        .count();

    assert_eq!(caret, CARET_ROWS);
}

#[test]
fn the_box_shows_the_profile_in_force() {
    for profile in LuminanceProfile::ALL {
        let expected = Field::LuminanceProfile.label().len() + profile.label().len() + "R".len();

        assert_eq!(glyphs(&drawn(profile)), expected, "{profile:?}");
    }
}

#[test]
fn the_row_does_not_repeat_the_value_in_its_own_column() {
    let neutral = glyphs(&drawn(LuminanceProfile::Neutral));
    let low = glyphs(&drawn(LuminanceProfile::Low));

    assert_eq!(neutral - low, "neutra".len() - "baixa".len());
}
