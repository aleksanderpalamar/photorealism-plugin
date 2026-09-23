use super::draft::{Changes, Draft};
use super::interaction::Interaction;
use super::panel;
use super::pointer::{Button, Pointer};
use super::resolve::{self, Request, Update};
use super::vertex::{Vertex, Viewport};
use super::visibility::Visibility;
use crate::settings::Settings;

#[derive(Clone, Copy, Debug, Default)]
pub struct Session {
    visibility: Visibility,
    pointer: Pointer,
    interaction: Interaction,
    draft: Draft,
}

impl Session {
    pub fn is_visible(&self) -> bool {
        self.visibility.is_visible()
    }

    pub fn toggle(&mut self, viewport: Viewport) {
        self.visibility = self.visibility.toggled();
        if self.visibility.is_visible() {
            self.pointer.center(viewport);
        }
    }

    pub fn move_pointer(&mut self, horizontal: f32, vertical: f32, viewport: Viewport) {
        self.pointer.move_by(horizontal, vertical, viewport);
    }

    pub fn set_button(&mut self, pressed: bool) {
        self.pointer.set_button(Button::from_pressed(pressed));
    }

    pub fn update(&mut self, stored: Settings, viewport: Viewport) -> Update {
        let current = self.draft.effective(stored);
        if !self.is_visible() {
            return Update {
                settings: current,
                request: Request::None,
            };
        }
        let layout = panel::layout_for(viewport);
        let action = self.interaction.update(&layout, self.pointer, &current);
        resolve::resolve(action, &mut self.draft, stored, current)
    }

    pub fn saved(&mut self) {
        self.draft.discard();
    }

    pub fn changes(&self) -> Changes {
        self.draft.changes()
    }

    pub fn vertices(&self, settings: &Settings, viewport: Viewport, title: &str) -> Vec<Vertex> {
        panel::vertices(
            settings,
            self.pointer.position(),
            self.changes(),
            viewport,
            title,
        )
    }
}

#[cfg(test)]
#[path = "session_tests.rs"]
mod tests;
