use super::push_row;
use crate::menu::field::Field;
use crate::menu::geometry::Point;
use crate::menu::layout::Layout;
use crate::menu::palette;
use crate::menu::primitive::Primitive;
use crate::menu::row::Row;
use crate::settings::{LuminanceProfile, Settings};

const PEAK: f32 = 1000.0;

fn profile_row() -> Row {
    Layout::build(Point { x: 24.0, y: 24.0 }, 2.0)
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
    push_row(&mut primitives, &profile_row(), &settings, PEAK, 2.0);
    primitives
}

fn boxes(primitives: &[Primitive]) -> Vec<f32> {
    let size = profile_row().switch_box().width;
    primitives
        .iter()
        .filter_map(|primitive| match primitive {
            Primitive::Rectangle { rect, .. } if rect.width == size => Some(rect.x),
            _ => None,
        })
        .collect()
}

fn filled(primitives: &[Primitive]) -> Vec<f32> {
    primitives
        .iter()
        .filter_map(|primitive| match primitive {
            Primitive::Rectangle { rect, color } if *color == palette::FILL => Some(rect.x),
            _ => None,
        })
        .collect()
}

#[test]
fn the_profile_row_draws_one_box_per_state() {
    for profile in [
        LuminanceProfile::Low,
        LuminanceProfile::Neutral,
        LuminanceProfile::High,
    ] {
        assert_eq!(boxes(&drawn(profile)).len(), LuminanceProfile::COUNT);
    }
}

#[test]
fn exactly_one_box_is_lit() {
    for profile in [
        LuminanceProfile::Low,
        LuminanceProfile::Neutral,
        LuminanceProfile::High,
    ] {
        assert_eq!(filled(&drawn(profile)).len(), 1);
    }
}

#[test]
fn advancing_the_profile_moves_the_lit_box() {
    let low = filled(&drawn(LuminanceProfile::Low));
    let neutral = filled(&drawn(LuminanceProfile::Neutral));
    let high = filled(&drawn(LuminanceProfile::High));

    assert!(low[0] < neutral[0]);
    assert!(neutral[0] < high[0]);
}

#[test]
fn the_boxes_stay_in_the_same_places() {
    let low = boxes(&drawn(LuminanceProfile::Low));
    let high = boxes(&drawn(LuminanceProfile::High));

    assert_eq!(low, high);
}
