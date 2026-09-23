use super::{Primitive, build};
use crate::menu::draft::Changes;
use crate::menu::geometry::{Point, Rect};
use crate::menu::layout::Layout;
use crate::menu::palette;
use crate::settings::{PeakNits, Settings};

fn layout() -> Layout {
    Layout::build(Point { x: 16.0, y: 16.0 }, 2.0)
}

fn bounds(primitive: &Primitive) -> Rect {
    match primitive {
        Primitive::Rectangle { rect, .. } | Primitive::Glyph { rect, .. } => *rect,
    }
}

fn glyph_count(primitives: &[Primitive]) -> usize {
    primitives
        .iter()
        .filter(|primitive| matches!(primitive, Primitive::Glyph { .. }))
        .count()
}

#[test]
fn the_panel_and_the_title_bar_come_first() {
    let layout = layout();
    let primitives = build(&layout, &Settings::default(), "menu", Changes::None, None);

    assert_eq!(
        primitives[0],
        Primitive::Rectangle {
            rect: layout.panel,
            color: palette::PANEL,
        }
    );
    assert_eq!(bounds(&primitives[1]), layout.title);
}

#[test]
fn nothing_is_drawn_outside_the_panel() {
    let layout = layout();
    let primitives = build(
        &layout,
        &Settings::default(),
        "photorealism-plugin",
        Changes::None,
        None,
    );

    for primitive in &primitives {
        let rect = bounds(primitive);
        assert!(rect.x >= layout.panel.x, "{primitive:?}");
        assert!(rect.right() <= layout.panel.right(), "{primitive:?}");
        assert!(rect.y >= layout.panel.y, "{primitive:?}");
        assert!(rect.bottom() <= layout.panel.bottom(), "{primitive:?}");
    }
}

#[test]
fn a_longer_title_produces_more_glyphs() {
    let layout = layout();
    let short = build(&layout, &Settings::default(), "a", Changes::None, None);
    let long = build(&layout, &Settings::default(), "abcd", Changes::None, None);

    assert_eq!(glyph_count(&long) - glyph_count(&short), 3);
}

#[test]
fn characters_without_a_glyph_are_skipped() {
    let layout = layout();
    let plain = build(&layout, &Settings::default(), "abc", Changes::None, None);
    let accented = build(&layout, &Settings::default(), "abcá", Changes::None, None);

    assert_eq!(glyph_count(&plain), glyph_count(&accented));
}

#[test]
fn a_locked_row_is_painted_with_the_locked_color() {
    let layout = layout();
    let automatic = build(&layout, &Settings::default(), "menu", Changes::None, None);
    let fixed = build(
        &layout,
        &Settings {
            hdr_peak_nits: PeakNits::Fixed(1200.0),
            ..Settings::default()
        },
        "menu",
        Changes::None,
        None,
    );

    let locked = |primitives: &[Primitive]| {
        primitives
            .iter()
            .filter(|primitive| match primitive {
                Primitive::Rectangle { color, .. } | Primitive::Glyph { color, .. } => {
                    *color == palette::LOCKED
                }
            })
            .count()
    };

    assert!(locked(&automatic) > locked(&fixed));
}

#[test]
fn turning_a_switch_off_removes_its_inner_mark() {
    let layout = layout();
    let on = build(&layout, &Settings::default(), "menu", Changes::None, None);
    let off = build(
        &layout,
        &Settings {
            enabled: false,
            ..Settings::default()
        },
        "menu",
        Changes::None,
        None,
    );

    assert_eq!(on.len() - off.len(), 1);
}

#[test]
fn the_pointer_is_drawn_only_when_it_is_given() {
    let layout = layout();
    let without = build(&layout, &Settings::default(), "menu", Changes::None, None);
    let with = build(
        &layout,
        &Settings::default(),
        "menu",
        Changes::None,
        Some(Point { x: 900.0, y: 500.0 }),
    );

    assert!(with.len() > without.len());
}

#[test]
fn the_pointer_may_sit_outside_the_panel() {
    let layout = layout();
    let far = Point {
        x: 1500.0,
        y: 900.0,
    };
    let primitives = build(
        &layout,
        &Settings::default(),
        "menu",
        Changes::None,
        Some(far),
    );
    let last = bounds(primitives.last().expect("primitivas"));

    assert!(last.x >= far.x);
    assert!(last.y >= far.y);
}
