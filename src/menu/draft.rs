use crate::settings::Settings;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Changes {
    #[default]
    None,
    Pending,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Draft {
    edited: Option<Settings>,
}

impl Draft {
    pub fn effective(&self, stored: Settings) -> Settings {
        self.edited.unwrap_or(stored)
    }

    pub fn edit(&mut self, stored: Settings) -> &mut Settings {
        self.edited.get_or_insert(stored)
    }

    pub fn changes(&self) -> Changes {
        match self.edited {
            Some(_) => Changes::Pending,
            None => Changes::None,
        }
    }

    pub fn discard(&mut self) {
        self.edited = None;
    }
}

#[cfg(test)]
mod tests {
    use super::{Changes, Draft};
    use crate::settings::Settings;

    fn stored() -> Settings {
        Settings {
            exposure: 0.5,
            ..Settings::default()
        }
    }

    #[test]
    fn an_untouched_draft_follows_the_stored_configuration() {
        assert_eq!(Draft::default().effective(stored()), stored());
    }

    #[test]
    fn editing_seeds_the_draft_from_the_stored_configuration() {
        let mut draft = Draft::default();

        draft.edit(stored()).contrast = 1.4;

        assert_eq!(draft.effective(stored()).exposure, 0.5);
        assert_eq!(draft.effective(stored()).contrast, 1.4);
    }

    #[test]
    fn the_draft_wins_over_a_later_reload_from_disk() {
        let mut draft = Draft::default();
        draft.edit(stored()).contrast = 1.4;

        let reloaded = Settings {
            contrast: 0.3,
            ..Settings::default()
        };

        assert_eq!(draft.effective(reloaded).contrast, 1.4);
    }

    #[test]
    fn an_untouched_draft_reports_no_changes() {
        assert_eq!(Draft::default().changes(), Changes::None);
    }

    #[test]
    fn discarding_returns_to_the_stored_configuration() {
        let mut draft = Draft::default();
        draft.edit(stored()).contrast = 1.4;
        assert_eq!(draft.changes(), Changes::Pending);

        draft.discard();

        assert_eq!(draft.effective(stored()), stored());
        assert_eq!(draft.changes(), Changes::None);
    }
}
