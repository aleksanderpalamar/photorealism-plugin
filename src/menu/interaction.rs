use super::field::{Control, Field};
use super::layout::Layout;
use super::pointer::Pointer;
use super::row::Row;
use crate::settings::Settings;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum Grab {
    #[default]
    Idle,
    Dragging(Field),
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Action {
    #[default]
    Idle,
    Slide(Field, f32),
    Flip(Field),
}

impl Action {
    pub fn is_idle(self) -> bool {
        matches!(self, Self::Idle)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Interaction {
    grab: Grab,
    held: bool,
}

impl Interaction {
    pub fn update(&mut self, layout: &Layout, pointer: Pointer, settings: &Settings) -> Action {
        let pressed = pointer.button().is_pressed();
        let started = pressed && !self.held;
        self.held = pressed;
        if !pressed {
            self.grab = Grab::Idle;
            return Action::Idle;
        }
        if let Grab::Dragging(field) = self.grab {
            return slide(layout, field, pointer);
        }
        if !started {
            return Action::Idle;
        }
        self.press(layout, pointer, settings)
    }

    fn press(&mut self, layout: &Layout, pointer: Pointer, settings: &Settings) -> Action {
        let Some(row) = row_under(layout, pointer) else {
            return Action::Idle;
        };
        if !row.field.is_editable(settings) {
            return Action::Idle;
        }
        match row.field.control() {
            Control::Switch => flip(row, pointer),
            Control::Slider(_) => self.start_drag(row, pointer),
        }
    }

    fn start_drag(&mut self, row: Row, pointer: Pointer) -> Action {
        if !row.slider_area().contains(pointer.position()) {
            return Action::Idle;
        }
        self.grab = Grab::Dragging(row.field);
        Action::Slide(row.field, row.track.fraction_at(pointer.position().x))
    }
}

fn flip(row: Row, pointer: Pointer) -> Action {
    if !row.switch_box().contains(pointer.position()) {
        return Action::Idle;
    }
    Action::Flip(row.field)
}

fn slide(layout: &Layout, field: Field, pointer: Pointer) -> Action {
    let Some(row) = layout.rows.iter().find(|row| row.field == field) else {
        return Action::Idle;
    };
    Action::Slide(field, row.track.fraction_at(pointer.position().x))
}

fn row_under(layout: &Layout, pointer: Pointer) -> Option<Row> {
    layout
        .rows
        .iter()
        .find(|row| row.bounds.contains(pointer.position()))
        .copied()
}

pub fn apply(action: Action, settings: &mut Settings) {
    match action {
        Action::Idle => {}
        Action::Slide(field, fraction) => field.set_fraction(settings, fraction),
        Action::Flip(field) => field.flip(settings),
    }
}

#[cfg(test)]
mod tests {
    use super::{Action, Interaction, apply};
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
        pointer.move_by(point.x, point.y, crate::menu::vertex::Viewport {
            width: 1920.0,
            height: 1080.0,
        });
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
    fn applying_an_action_writes_the_settings() {
        let mut settings = Settings::default();

        apply(Action::Slide(Field::Saturation, 1.0), &mut settings);
        apply(Action::Flip(Field::Enabled), &mut settings);
        apply(Action::Idle, &mut settings);

        assert_eq!(settings.saturation, 2.0);
        assert!(!settings.enabled);
    }
}
