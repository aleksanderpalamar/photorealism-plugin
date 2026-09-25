use super::Interaction;
use crate::menu::action::Action;
use crate::menu::field::Field;
use crate::menu::geometry::Point;
use crate::menu::layout::Layout;
use crate::menu::pointer::{Button, Pointer};
use crate::settings::Settings;

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

#[test]
fn pressing_a_switch_box_flips_it() {
    let layout = layout();
    let row = layout.rows[0];
    let point = Point {
        x: row.switch_box().x + 2.0,
        y: row.switch_box().y + 2.0,
    };
    let mut interaction = Interaction::default();

    let action = interaction.update(&layout, pointer_at(point, true), &Settings::default());

    assert_eq!(action, Action::Flip(Field::Enabled));
}

#[test]
fn pressing_a_label_does_not_move_anything() {
    let layout = layout();
    let row = layout.rows[1];
    let point = Point {
        x: row.label.x + 4.0,
        y: row.bounds.y + row.bounds.height / 2.0,
    };
    let mut interaction = Interaction::default();

    let action = interaction.update(&layout, pointer_at(point, true), &Settings::default());

    assert_eq!(action, Action::Idle);
}

#[test]
fn pressing_outside_every_row_does_nothing() {
    let mut interaction = Interaction::default();
    let point = Point {
        x: 1500.0,
        y: 900.0,
    };

    let action = interaction.update(&layout(), pointer_at(point, true), &Settings::default());

    assert_eq!(action, Action::Idle);
}

#[test]
fn pressing_the_reset_button_resets_that_row() {
    let layout = layout();
    let row = layout.rows[1];
    let point = Point {
        x: row.reset.x + 2.0,
        y: row.bounds.y + row.bounds.height / 2.0,
    };
    let mut interaction = Interaction::default();

    let action = interaction.update(&layout, pointer_at(point, true), &Settings::default());

    assert_eq!(action, Action::Reset(Field::Exposure));
}

#[test]
fn a_locked_row_can_still_be_reset() {
    let layout = layout();
    let row = layout
        .rows
        .iter()
        .find(|row| row.field == Field::PeakNits)
        .copied()
        .expect("linha");
    let point = Point {
        x: row.reset.x + 2.0,
        y: row.bounds.y + row.bounds.height / 2.0,
    };
    let mut interaction = Interaction::default();

    let action = interaction.update(&layout, pointer_at(point, true), &Settings::default());

    assert_eq!(action, Action::Reset(Field::PeakNits));
}

#[test]
fn the_footer_buttons_report_their_own_actions() {
    let layout = layout();
    let save = Point {
        x: layout.save.x + 4.0,
        y: layout.save.y + 2.0,
    };
    let discard = Point {
        x: layout.discard.x + 4.0,
        y: layout.discard.y + 2.0,
    };

    let mut interaction = Interaction::default();
    assert_eq!(
        interaction.update(&layout, pointer_at(save, true), &Settings::default()),
        Action::Save
    );

    let mut interaction = Interaction::default();
    assert_eq!(
        interaction.update(&layout, pointer_at(discard, true), &Settings::default()),
        Action::Discard
    );
}

#[test]
fn saving_fires_only_once_per_press() {
    let layout = layout();
    let save = Point {
        x: layout.save.x + 4.0,
        y: layout.save.y + 2.0,
    };
    let mut interaction = Interaction::default();
    interaction.update(&layout, pointer_at(save, true), &Settings::default());

    let action = interaction.update(&layout, pointer_at(save, true), &Settings::default());

    assert_eq!(action, Action::Idle);
}

fn profile_row() -> crate::menu::row::Row {
    layout()
        .rows
        .iter()
        .find(|row| row.field == Field::LuminanceProfile)
        .copied()
        .expect("linha do perfil")
}

#[test]
fn pressing_the_profile_area_advances_it() {
    let row = profile_row();
    let point = Point {
        x: row.track.x + 4.0,
        y: row.bounds.y + row.bounds.height / 2.0,
    };
    let mut interaction = Interaction::default();

    let action = interaction.update(&layout(), pointer_at(point, true), &Settings::default());

    assert_eq!(action, Action::Flip(Field::LuminanceProfile));
}

#[test]
fn pressing_the_profile_label_does_nothing() {
    let row = profile_row();
    let point = Point {
        x: row.label.x + 4.0,
        y: row.bounds.y + row.bounds.height / 2.0,
    };
    let mut interaction = Interaction::default();

    let action = interaction.update(&layout(), pointer_at(point, true), &Settings::default());

    assert_eq!(action, Action::Idle);
}

#[test]
fn holding_the_button_over_the_profile_does_not_drag() {
    let row = profile_row();
    let point = Point {
        x: row.track.x + 4.0,
        y: row.bounds.y + row.bounds.height / 2.0,
    };
    let mut interaction = Interaction::default();
    interaction.update(&layout(), pointer_at(point, true), &Settings::default());

    let away = Point {
        x: row.track.right() - 2.0,
        y: point.y,
    };
    let action = interaction.update(&layout(), pointer_at(away, true), &Settings::default());

    assert_eq!(action, Action::Idle);
}
