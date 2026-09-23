use super::Interaction;
use crate::menu::action::Action;
use crate::menu::field::Field;
use crate::menu::geometry::Point;
use crate::menu::layout::Layout;
use crate::menu::pointer::{Button, Pointer};
use crate::settings::{PeakNits, Settings};

fn layout() -> Layout {
    Layout::build(Point { x: 24.0, y: 24.0 }, 2.0)
}

fn pointer_at(point: Point, pressed: bool) -> Pointer {
    let mut pointer = Pointer::default();
    pointer.move_by(
        point.x,
        point.y,
        crate::menu::vertex::Viewport {
            width: 1920.0,
            height: 1080.0,
        },
    );
    pointer.set_button(Button::from_pressed(pressed));
    pointer
}

fn center_of_track(field: Field) -> Point {
    let layout = layout();
    let row = layout
        .rows
        .iter()
        .find(|row| row.field == field)
        .copied()
        .expect("linha");
    Point {
        x: row.track.x + row.track.width / 2.0,
        y: row.bounds.y + row.bounds.height / 2.0,
    }
}

#[test]
fn a_released_pointer_does_nothing() {
    let mut interaction = Interaction::default();

    let action = interaction.update(
        &layout(),
        pointer_at(center_of_track(Field::Exposure), false),
        &Settings::default(),
    );

    assert_eq!(action, Action::Idle);
}

#[test]
fn pressing_on_a_track_slides_that_field() {
    let mut interaction = Interaction::default();

    let action = interaction.update(
        &layout(),
        pointer_at(center_of_track(Field::Exposure), true),
        &Settings::default(),
    );

    assert_eq!(action, Action::Slide(Field::Exposure, 0.5));
}

#[test]
fn a_drag_keeps_the_field_after_leaving_the_row() {
    let mut interaction = Interaction::default();
    let start = center_of_track(Field::Exposure);
    interaction.update(&layout(), pointer_at(start, true), &Settings::default());

    let away = Point {
        x: start.x + 40.0,
        y: start.y + 400.0,
    };
    let action = interaction.update(&layout(), pointer_at(away, true), &Settings::default());

    assert!(matches!(action, Action::Slide(Field::Exposure, _)));
}

#[test]
fn releasing_ends_the_drag() {
    let mut interaction = Interaction::default();
    let start = center_of_track(Field::Exposure);
    interaction.update(&layout(), pointer_at(start, true), &Settings::default());
    interaction.update(&layout(), pointer_at(start, false), &Settings::default());

    let action = interaction.update(&layout(), pointer_at(start, true), &Settings::default());

    assert_eq!(action, Action::Slide(Field::Exposure, 0.5));
}

#[test]
fn a_locked_row_ignores_the_press() {
    let mut interaction = Interaction::default();

    let action = interaction.update(
        &layout(),
        pointer_at(center_of_track(Field::PeakNits), true),
        &Settings::default(),
    );

    assert_eq!(action, Action::Idle);
}

#[test]
fn unlocking_the_peak_makes_its_slider_respond() {
    let settings = Settings {
        hdr_peak_nits: PeakNits::Fixed(1000.0),
        ..Settings::default()
    };
    let mut interaction = Interaction::default();

    let action = interaction.update(
        &layout(),
        pointer_at(center_of_track(Field::PeakNits), true),
        &settings,
    );

    assert!(matches!(action, Action::Slide(Field::PeakNits, _)));
}
