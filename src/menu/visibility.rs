#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Visibility {
    #[default]
    Hidden,
    Visible,
}

impl Visibility {
    pub fn toggled(self) -> Self {
        match self {
            Self::Hidden => Self::Visible,
            Self::Visible => Self::Hidden,
        }
    }

    pub fn is_visible(self) -> bool {
        matches!(self, Self::Visible)
    }
}

#[cfg(test)]
mod tests {
    use super::Visibility;

    #[test]
    fn starts_hidden() {
        assert_eq!(Visibility::default(), Visibility::Hidden);
        assert!(!Visibility::default().is_visible());
    }

    #[test]
    fn toggling_twice_returns_to_the_start() {
        let once = Visibility::default().toggled();

        assert!(once.is_visible());
        assert_eq!(once.toggled(), Visibility::default());
    }
}
