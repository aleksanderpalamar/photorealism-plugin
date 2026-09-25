use super::action::Action;
use super::choice::Choice;
use super::field::{Control, Field};
use super::hit::{flip, pick, row_under, slide};
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

#[derive(Clone, Copy, Debug, Default)]
pub struct Interaction {
    grab: Grab,
    held: bool,
    open: Option<Field>,
}

impl Interaction {
    pub fn open(&self) -> Option<Field> {
        self.open
    }

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
        if let Some(field) = self.open.take() {
            return pick(layout, field, pointer);
        }
        if layout.save.contains(pointer.position()) {
            return Action::Save;
        }
        if layout.discard.contains(pointer.position()) {
            return Action::Discard;
        }
        let Some(row) = row_under(layout, pointer) else {
            return Action::Idle;
        };
        if row.reset.contains(pointer.position()) {
            return Action::Reset(row.field);
        }
        if !row.field.is_editable(settings) {
            return Action::Idle;
        }
        match row.field.control() {
            Control::Switch => flip(row, row.switch_box(), pointer),
            Control::Choice(states) => self.unfold(row, states, pointer),
            Control::Slider(_) => self.start_drag(row, pointer),
        }
    }

    fn unfold(&mut self, row: Row, states: usize, pointer: Pointer) -> Action {
        if !Choice::build(&row, states)
            .closed
            .contains(pointer.position())
        {
            return Action::Idle;
        }
        self.open = Some(row.field);
        Action::Idle
    }

    fn start_drag(&mut self, row: Row, pointer: Pointer) -> Action {
        if !row.slider_area().contains(pointer.position()) {
            return Action::Idle;
        }
        self.grab = Grab::Dragging(row.field);
        Action::Slide(row.field, row.track.fraction_at(pointer.position().x))
    }
}

#[cfg(test)]
#[path = "interaction_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "interaction_press_tests.rs"]
mod press_tests;

#[cfg(test)]
#[path = "interaction_choice_tests.rs"]
mod choice_tests;
