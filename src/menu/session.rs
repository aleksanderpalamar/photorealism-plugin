use super::draft::Draft;
use super::interaction::{self, Interaction};
use super::panel;
use super::pointer::{Button, Pointer};
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
        self.interaction = Interaction::default();
        self.pointer.center(viewport);
    }

    pub fn move_pointer(&mut self, horizontal: f32, vertical: f32, viewport: Viewport) {
        self.pointer.move_by(horizontal, vertical, viewport);
    }

    pub fn set_button(&mut self, pressed: bool) {
        self.pointer.set_button(Button::from_pressed(pressed));
    }

    pub fn settings(&mut self, stored: Settings, viewport: Viewport) -> Settings {
        let current = self.draft.effective(stored);
        if !self.is_visible() {
            return current;
        }
        let layout = panel::layout_for(viewport);
        let action = self.interaction.update(&layout, self.pointer, &current);
        if action.is_idle() {
            return current;
        }
        interaction::apply(action, self.draft.edit(stored));
        self.draft.effective(stored)
    }

    pub fn vertices(
        &self,
        settings: &Settings,
        resolved_peak: f32,
        viewport: Viewport,
        title: &str,
    ) -> Vec<Vertex> {
        panel::vertices(
            settings,
            resolved_peak,
            self.pointer.position(),
            viewport,
            title,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Session;
    use crate::menu::vertex::Viewport;
    use crate::settings::Settings;

    fn viewport() -> Viewport {
        Viewport {
            width: 1920.0,
            height: 1080.0,
        }
    }

    fn stored() -> Settings {
        Settings::default()
    }

    fn press_first_track(session: &mut Session, fraction: f32) {
        let layout = crate::menu::panel::layout_for(viewport());
        let row = layout.rows[1];
        session.move_pointer(-f32::MAX, -f32::MAX, viewport());
        session.move_pointer(
            row.track.x + row.track.width * fraction,
            row.bounds.y + row.bounds.height / 2.0,
            viewport(),
        );
        session.set_button(true);
    }

    #[test]
    fn a_hidden_menu_passes_the_stored_configuration_through() {
        let mut session = Session::default();

        assert!(!session.is_visible());
        assert_eq!(session.settings(stored(), viewport()), stored());
    }

    #[test]
    fn opening_centers_the_pointer() {
        let mut session = Session::default();

        session.toggle(viewport());

        assert!(session.is_visible());
        assert_eq!(session.settings(stored(), viewport()), stored());
    }

    #[test]
    fn an_open_menu_without_input_keeps_following_the_file() {
        let mut session = Session::default();
        session.toggle(viewport());

        let reloaded = Settings {
            contrast: 1.5,
            ..Settings::default()
        };

        assert_eq!(session.settings(reloaded, viewport()).contrast, 1.5);
    }

    #[test]
    fn dragging_a_slider_changes_that_parameter() {
        let mut session = Session::default();
        session.toggle(viewport());
        press_first_track(&mut session, 1.0);

        let settings = session.settings(stored(), viewport());

        assert_eq!(settings.exposure, 4.0);
    }

    #[test]
    fn an_edited_parameter_survives_a_reload_from_disk() {
        let mut session = Session::default();
        session.toggle(viewport());
        press_first_track(&mut session, 1.0);
        session.settings(stored(), viewport());
        session.set_button(false);
        session.settings(stored(), viewport());

        let reloaded = Settings {
            contrast: 1.5,
            ..Settings::default()
        };
        let settings = session.settings(reloaded, viewport());

        assert_eq!(settings.exposure, 4.0);
        assert_eq!(settings.contrast, 0.99);
    }

    #[test]
    fn closing_the_menu_cancels_an_active_drag() {
        let mut session = Session::default();
        session.toggle(viewport());
        press_first_track(&mut session, 0.0);
        assert_eq!(session.settings(stored(), viewport()).exposure, -4.0);

        session.toggle(viewport());
        session.toggle(viewport());
        session.set_button(true);

        assert_eq!(session.settings(stored(), viewport()).exposure, -4.0);
    }

    #[test]
    fn reopening_starts_from_a_released_pointer() {
        let mut session = Session::default();
        session.toggle(viewport());
        press_first_track(&mut session, 1.0);
        session.settings(stored(), viewport());

        session.toggle(viewport());
        session.toggle(viewport());

        assert_eq!(session.settings(stored(), viewport()).exposure, 4.0);
    }

    #[test]
    fn a_closed_menu_keeps_what_was_edited() {
        let mut session = Session::default();
        session.toggle(viewport());
        press_first_track(&mut session, 1.0);
        session.settings(stored(), viewport());

        session.toggle(viewport());

        assert_eq!(session.settings(stored(), viewport()).exposure, 4.0);
    }

    #[test]
    fn the_panel_only_draws_while_it_is_open() {
        let session = Session::default();

        assert!(!session.vertices(&stored(), 1000.0, viewport(), "menu").is_empty());
    }
}
