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
        self.interaction = Interaction::default();
        self.pointer.center(viewport);
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

    pub fn settle(&mut self, stored: Settings) {
        if self.draft.effective(stored) != stored {
            return;
        }
        self.draft.discard();
    }

    pub fn changes(&self) -> Changes {
        self.draft.changes()
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
            self.changes(),
            viewport,
            title,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Session;
    use crate::menu::draft::Changes;
    use crate::menu::resolve::Request;
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
        assert_eq!(session.update(stored(), viewport()).settings, stored());
    }

    #[test]
    fn opening_centers_the_pointer() {
        let mut session = Session::default();

        session.toggle(viewport());

        assert!(session.is_visible());
        assert_eq!(session.update(stored(), viewport()).settings, stored());
    }

    #[test]
    fn an_open_menu_without_input_keeps_following_the_file() {
        let mut session = Session::default();
        session.toggle(viewport());

        let reloaded = Settings {
            contrast: 1.5,
            ..Settings::default()
        };

        assert_eq!(session.update(reloaded, viewport()).settings.contrast, 1.5);
    }

    #[test]
    fn dragging_a_slider_changes_that_parameter() {
        let mut session = Session::default();
        session.toggle(viewport());
        press_first_track(&mut session, 1.0);

        let settings = session.update(stored(), viewport()).settings;

        assert_eq!(settings.exposure, 4.0);
    }

    #[test]
    fn an_edited_parameter_survives_a_reload_from_disk() {
        let mut session = Session::default();
        session.toggle(viewport());
        press_first_track(&mut session, 1.0);
        session.update(stored(), viewport());
        session.set_button(false);
        session.update(stored(), viewport());

        let reloaded = Settings {
            contrast: 1.5,
            ..Settings::default()
        };
        let settings = session.update(reloaded, viewport()).settings;

        assert_eq!(settings.exposure, 4.0);
        assert_eq!(settings.contrast, 0.99);
    }

    #[test]
    fn closing_the_menu_cancels_an_active_drag() {
        let mut session = Session::default();
        session.toggle(viewport());
        press_first_track(&mut session, 0.0);
        assert_eq!(session.update(stored(), viewport()).settings.exposure, -4.0);

        session.toggle(viewport());
        session.toggle(viewport());
        session.set_button(true);

        assert_eq!(session.update(stored(), viewport()).settings.exposure, -4.0);
    }

    #[test]
    fn reopening_starts_from_a_released_pointer() {
        let mut session = Session::default();
        session.toggle(viewport());
        press_first_track(&mut session, 1.0);
        session.update(stored(), viewport());

        session.toggle(viewport());
        session.toggle(viewport());

        assert_eq!(session.update(stored(), viewport()).settings.exposure, 4.0);
    }

    #[test]
    fn a_closed_menu_keeps_what_was_edited() {
        let mut session = Session::default();
        session.toggle(viewport());
        press_first_track(&mut session, 1.0);
        session.update(stored(), viewport());

        session.toggle(viewport());

        assert_eq!(session.update(stored(), viewport()).settings.exposure, 4.0);
    }

    #[test]
    fn the_panel_only_draws_while_it_is_open() {
        let session = Session::default();

        assert!(
            !session
                .vertices(&stored(), 1000.0, viewport(), "menu")
                .is_empty()
        );
    }

    fn press(session: &mut Session, x: f32, y: f32) {
        session.move_pointer(-f32::MAX, -f32::MAX, viewport());
        session.move_pointer(x, y, viewport());
        session.set_button(true);
    }

    #[test]
    fn pressing_save_asks_the_runtime_to_write() {
        let mut session = Session::default();
        session.toggle(viewport());
        let layout = crate::menu::panel::layout_for(viewport());
        press(&mut session, layout.save.x + 4.0, layout.save.y + 2.0);

        let update = session.update(stored(), viewport());

        assert_eq!(update.request, Request::Save);
    }

    #[test]
    fn pressing_discard_returns_to_the_stored_configuration() {
        let mut session = Session::default();
        session.toggle(viewport());
        press_first_track(&mut session, 1.0);
        session.update(stored(), viewport());
        session.set_button(false);
        session.update(stored(), viewport());

        let layout = crate::menu::panel::layout_for(viewport());
        press(&mut session, layout.discard.x + 4.0, layout.discard.y + 2.0);
        let update = session.update(stored(), viewport());

        assert_eq!(update.settings, stored());
        assert_eq!(update.request, Request::None);
    }

    #[test]
    fn the_draft_survives_until_the_file_carries_it() {
        let mut session = Session::default();
        session.toggle(viewport());
        press_first_track(&mut session, 1.0);
        let edited = session.update(stored(), viewport()).settings;
        session.set_button(false);

        session.settle(stored());

        assert_eq!(session.changes(), Changes::Pending);
        assert_eq!(session.update(stored(), viewport()).settings, edited);
    }

    #[test]
    fn the_draft_is_released_once_the_file_matches_it() {
        let mut session = Session::default();
        session.toggle(viewport());
        press_first_track(&mut session, 1.0);
        let edited = session.update(stored(), viewport()).settings;
        session.set_button(false);

        session.settle(edited);

        assert_eq!(session.changes(), Changes::None);
        assert_eq!(session.update(edited, viewport()).settings, edited);
    }

    #[test]
    fn an_edit_marks_the_changes_as_pending() {
        let mut session = Session::default();
        session.toggle(viewport());
        assert_eq!(session.changes(), Changes::None);

        press_first_track(&mut session, 1.0);
        session.update(stored(), viewport());

        assert_eq!(session.changes(), Changes::Pending);
    }
}
