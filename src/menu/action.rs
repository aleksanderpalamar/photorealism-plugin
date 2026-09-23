use super::field::Field;
use crate::settings::Settings;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Action {
    #[default]
    Idle,
    Slide(Field, f32),
    Flip(Field),
    Reset(Field),
    Save,
    Discard,
}

pub fn apply(action: Action, settings: &mut Settings) {
    match action {
        Action::Slide(field, fraction) => field.set_fraction(settings, fraction),
        Action::Flip(field) => field.flip(settings),
        Action::Reset(field) => field.reset(settings),
        Action::Idle | Action::Save | Action::Discard => {}
    }
}

#[cfg(test)]
mod tests {
    use super::{Action, apply};
    use crate::menu::field::Field;
    use crate::settings::Settings;

    #[test]
    fn applying_an_action_writes_the_settings() {
        let mut settings = Settings::default();

        apply(Action::Slide(Field::Saturation, 1.0), &mut settings);
        apply(Action::Flip(Field::Enabled), &mut settings);
        apply(Action::Idle, &mut settings);

        assert_eq!(settings.saturation, 2.0);
        assert!(!settings.enabled);
    }

    #[test]
    fn resetting_through_apply_restores_the_default() {
        let mut settings = Settings::default();
        apply(Action::Slide(Field::Contrast, 1.0), &mut settings);

        apply(Action::Reset(Field::Contrast), &mut settings);

        assert_eq!(settings.contrast, Settings::default().contrast);
    }
}
