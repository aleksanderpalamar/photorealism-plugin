use super::draw_row::push_row;
use super::layout::Layout;
use super::palette;
use super::primitive::Primitive;
use super::text::push_text;
use crate::settings::Settings;

pub fn build(
    layout: &Layout,
    settings: &Settings,
    resolved_peak: f32,
    title: &str,
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
        push_row(&mut primitives, row, settings, resolved_peak, layout.scale);
    }
    primitives
}

#[cfg(test)]
mod tests {
    use super::{Primitive, build};
    use crate::menu::geometry::{Point, Rect};
    use crate::menu::layout::Layout;
    use crate::menu::palette;

    const PEAK: f32 = 1000.0;
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
        let primitives = build(&layout, &Settings::default(), PEAK, "menu");

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
        let primitives = build(&layout, &Settings::default(), PEAK, "photorealism-plugin");

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
        let short = build(&layout, &Settings::default(), PEAK, "a");
        let long = build(&layout, &Settings::default(), PEAK, "abcd");

        assert_eq!(glyph_count(&long) - glyph_count(&short), 3);
    }

    #[test]
    fn characters_without_a_glyph_are_skipped() {
        let layout = layout();
        let plain = build(&layout, &Settings::default(), PEAK, "abc");
        let accented = build(&layout, &Settings::default(), PEAK, "abcá");

        assert_eq!(glyph_count(&plain), glyph_count(&accented));
    }

    #[test]
    fn a_locked_row_is_painted_with_the_locked_color() {
        let layout = layout();
        let automatic = build(&layout, &Settings::default(), PEAK, "menu");
        let fixed = build(
            &layout,
            &Settings {
                hdr_peak_nits: PeakNits::Fixed(1200.0),
                ..Settings::default()
            },
            PEAK,
            "menu",
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
        let on = build(&layout, &Settings::default(), PEAK, "menu");
        let off = build(
            &layout,
            &Settings {
                enabled: false,
                ..Settings::default()
            },
            PEAK,
            "menu",
        );

        assert_eq!(on.len() - off.len(), 1);
    }
}
