use super::Interaction;
use crate::menu::action::Action;
use crate::menu::choice::Choice;
use crate::menu::field::Field;
use crate::menu::geometry::Point;
use crate::menu::geometry::Rect;
use crate::menu::layout::Layout;
use crate::menu::pointer::{Button, Pointer};
use crate::menu::row::Row;
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

fn profile_row() -> Row {
    layout()
        .rows
        .iter()
        .find(|row| row.field == Field::LuminanceProfile)
        .copied()
        .expect("linha do perfil")
}

fn closed_box() -> Rect {
    Choice::build(&profile_row(), 3).closed
}

fn open_list(interaction: &mut Interaction) {
    let point = Point {
        x: closed_box().x + 4.0,
        y: closed_box().y + 2.0,
    };
    interaction.update(&layout(), pointer_at(point, true), &Settings::default());
    interaction.update(&layout(), pointer_at(point, false), &Settings::default());
}

#[test]
fn pressing_the_closed_box_opens_the_list() {
    let mut interaction = Interaction::default();
    let point = Point {
        x: closed_box().x + 4.0,
        y: closed_box().y + 2.0,
    };

    let action = interaction.update(&layout(), pointer_at(point, true), &Settings::default());

    assert_eq!(action, Action::Idle);
    assert_eq!(interaction.open(), Some(Field::LuminanceProfile));
}

#[test]
fn pressing_an_item_selects_that_profile() {
    let choice = Choice::build(&profile_row(), 3);

    for index in 0..3 {
        let mut interaction = Interaction::default();
        open_list(&mut interaction);
        let item = choice.items[index];
        let point = Point {
            x: item.x + 4.0,
            y: item.y + item.height / 2.0,
        };

        let action = interaction.update(&layout(), pointer_at(point, true), &Settings::default());

        assert_eq!(action, Action::Select(Field::LuminanceProfile, index));
        assert_eq!(interaction.open(), None);
    }
}

#[test]
fn pressing_outside_the_open_list_closes_it() {
    let mut interaction = Interaction::default();
    open_list(&mut interaction);
    let away = Point {
        x: 1500.0,
        y: 900.0,
    };

    let action = interaction.update(&layout(), pointer_at(away, true), &Settings::default());

    assert_eq!(action, Action::Idle);
    assert_eq!(interaction.open(), None);
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
