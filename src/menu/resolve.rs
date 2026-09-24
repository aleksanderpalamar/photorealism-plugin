use super::action::{self, Action};
use super::draft::Draft;
use crate::settings::Settings;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Request {
    #[default]
    None,
    Save,
}

#[derive(Clone, Copy, Debug)]
pub struct Update {
    pub settings: Settings,
    pub request: Request,
}

pub fn resolve(action: Action, draft: &mut Draft, stored: Settings, current: Settings) -> Update {
    match action {
        Action::Save => Update {
            settings: current,
            request: Request::Save,
        },
        Action::Discard => {
            draft.discard();
            Update {
                settings: stored,
                request: Request::None,
            }
        }
        Action::Idle => Update {
            settings: current,
            request: Request::None,
        },
        edit => {
            action::apply(edit, draft.edit(stored));
            Update {
                settings: draft.effective(stored),
                request: Request::None,
            }
        }
    }
}
