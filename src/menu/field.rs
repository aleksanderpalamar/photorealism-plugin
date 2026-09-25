use crate::settings::{LuminanceProfile, Range, range};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    Enabled,
    Exposure,
    LuminanceProfile,
    Contrast,
    Saturation,
    Temperature,
    Tint,
    HighlightRolloff,
    PaperWhite,
    AutomaticPeak,
    PeakNits,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Control {
    Slider(Range),
    Switch,
    Choice(usize),
}

impl Field {
    pub const ALL: [Self; 11] = [
        Self::Enabled,
        Self::Exposure,
        Self::LuminanceProfile,
        Self::Contrast,
        Self::Saturation,
        Self::Temperature,
        Self::Tint,
        Self::HighlightRolloff,
        Self::PaperWhite,
        Self::AutomaticPeak,
        Self::PeakNits,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Enabled => "Ativado",
            Self::Exposure => "Exposicao",
            Self::LuminanceProfile => "Perfil de luminancia",
            Self::Contrast => "Contraste",
            Self::Saturation => "Saturacao",
            Self::Temperature => "Temperatura",
            Self::Tint => "Tint",
            Self::HighlightRolloff => "Joelho de altas luzes",
            Self::PaperWhite => "Paper white",
            Self::AutomaticPeak => "Pico automatico",
            Self::PeakNits => "Pico",
        }
    }

    pub fn control(self) -> Control {
        match self {
            Self::Enabled | Self::AutomaticPeak => Control::Switch,
            Self::LuminanceProfile => Control::Choice(LuminanceProfile::COUNT),
            Self::Exposure => Control::Slider(range::EXPOSURE),
            Self::Contrast => Control::Slider(range::CONTRAST),
            Self::Saturation => Control::Slider(range::SATURATION),
            Self::Temperature => Control::Slider(range::TEMPERATURE),
            Self::Tint => Control::Slider(range::TINT),
            Self::HighlightRolloff => Control::Slider(range::HIGHLIGHT_ROLLOFF),
            Self::PaperWhite => Control::Slider(range::PAPER_WHITE_NITS),
            Self::PeakNits => Control::Slider(range::PEAK_NITS),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Control, Field};

    #[test]
    fn every_listed_field_is_distinct() {
        let all = Field::ALL;

        for (index, field) in all.iter().enumerate() {
            assert!(!all[..index].contains(field), "{field:?}");
        }
    }

    #[test]
    fn every_field_has_a_non_empty_label() {
        for field in Field::ALL {
            assert!(!field.label().is_empty(), "{field:?}");
        }
    }

    #[test]
    fn labels_do_not_repeat() {
        let labels: Vec<&str> = Field::ALL.iter().map(|field| field.label()).collect();

        for (index, label) in labels.iter().enumerate() {
            assert!(!labels[..index].contains(label), "{label}");
        }
    }

    #[test]
    fn only_the_two_switches_are_not_sliders() {
        let switches: Vec<Field> = Field::ALL
            .into_iter()
            .filter(|field| matches!(field.control(), Control::Switch))
            .collect();

        assert_eq!(switches, vec![Field::Enabled, Field::AutomaticPeak]);
    }

    #[test]
    fn only_the_profile_offers_a_list() {
        let cycles: Vec<Field> = Field::ALL
            .into_iter()
            .filter(|field| matches!(field.control(), Control::Choice(_)))
            .collect();

        assert_eq!(cycles, vec![Field::LuminanceProfile]);
    }

    #[test]
    fn a_list_declares_more_than_two_states() {
        let Control::Choice(states) = Field::LuminanceProfile.control() else {
            panic!("o perfil precisa oferecer uma lista");
        };

        assert!(states > 2);
    }

    #[test]
    fn every_slider_declares_a_usable_range() {
        for field in Field::ALL {
            let Control::Slider(range) = field.control() else {
                continue;
            };
            assert!(range.maximum > range.minimum, "{field:?}");
        }
    }
}
