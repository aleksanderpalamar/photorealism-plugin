use super::field::{Control, Field};
use crate::settings::{LuminanceProfile, PeakNits, Settings};

const DEFAULT_FIXED_PEAK: f32 = 1000.0;
const UNIT_DECIMALS: f32 = 1000.0;

impl Field {
    pub fn set_fraction(self, settings: &mut Settings, fraction: f32) {
        let Control::Slider(range) = self.control() else {
            return;
        };
        self.assign(settings, quantize(self, range.value_at(fraction)));
    }

    pub fn flip(self, settings: &mut Settings) {
        match self {
            Self::Enabled => settings.enabled = !settings.enabled,
            Self::AutomaticPeak => settings.hdr_peak_nits = flipped_peak(settings),
            _ => {}
        }
    }

    pub fn set_choice(self, settings: &mut Settings, index: usize) {
        let Self::LuminanceProfile = self else {
            return;
        };
        let Some(profile) = LuminanceProfile::ALL.get(index) else {
            return;
        };
        settings.luminance_profile = *profile;
    }

    pub fn reset(self, settings: &mut Settings) {
        let defaults = Settings::default();
        match self {
            Self::Enabled => settings.enabled = defaults.enabled,
            Self::AutomaticPeak | Self::PeakNits => settings.hdr_peak_nits = defaults.hdr_peak_nits,
            Self::LuminanceProfile => settings.luminance_profile = defaults.luminance_profile,
            _ => self.assign(settings, self.value(&defaults, DEFAULT_FIXED_PEAK)),
        }
    }

    fn assign(self, settings: &mut Settings, value: f32) {
        match self {
            Self::Exposure => settings.exposure = value,
            Self::Contrast => settings.contrast = value,
            Self::Saturation => settings.saturation = value,
            Self::Temperature => settings.temperature = value,
            Self::Tint => settings.tint = value,
            Self::HighlightRolloff => settings.highlight_rolloff = value,
            Self::PaperWhite => settings.hdr_paper_white_nits = value,
            Self::PeakNits => settings.hdr_peak_nits = PeakNits::Fixed(value),
            Self::Enabled | Self::AutomaticPeak | Self::LuminanceProfile => {}
        }
    }
}

fn flipped_peak(settings: &Settings) -> PeakNits {
    match settings.hdr_peak_nits {
        PeakNits::Auto => PeakNits::Fixed(DEFAULT_FIXED_PEAK),
        PeakNits::Fixed(_) => PeakNits::Auto,
    }
}

fn quantize(field: Field, value: f32) -> f32 {
    match field {
        Field::Temperature | Field::PaperWhite | Field::PeakNits => value.round(),
        _ => (value * UNIT_DECIMALS).round() / UNIT_DECIMALS,
    }
}

#[cfg(test)]
mod tests {
    use super::Field;
    use crate::settings::{LuminanceProfile, PeakNits, Settings};

    const PEAK: f32 = 1000.0;

    #[test]
    fn a_fraction_written_to_a_slider_can_be_read_back() {
        let mut settings = Settings::default();

        Field::Saturation.set_fraction(&mut settings, 0.25);

        assert_eq!(settings.saturation, 0.5);
        assert_eq!(Field::Saturation.fraction(&settings, PEAK), 0.25);
    }

    #[test]
    fn nits_and_temperature_are_quantized_to_whole_numbers() {
        let mut settings = Settings::default();

        Field::Temperature.set_fraction(&mut settings, 0.317);
        Field::PaperWhite.set_fraction(&mut settings, 0.317);

        assert_eq!(settings.temperature, settings.temperature.round());
        assert_eq!(
            settings.hdr_paper_white_nits,
            settings.hdr_paper_white_nits.round()
        );
    }

    #[test]
    fn flipping_a_switch_alternates_its_state() {
        let mut settings = Settings::default();

        Field::Enabled.flip(&mut settings);
        assert!(!settings.enabled);

        Field::AutomaticPeak.flip(&mut settings);
        assert_eq!(settings.hdr_peak_nits, PeakNits::Fixed(1000.0));
        Field::AutomaticPeak.flip(&mut settings);
        assert_eq!(settings.hdr_peak_nits, PeakNits::Auto);
    }

    #[test]
    fn resetting_restores_the_default_of_that_field_only() {
        let mut settings = Settings::default();
        Field::Exposure.set_fraction(&mut settings, 1.0);
        Field::Contrast.set_fraction(&mut settings, 1.0);

        Field::Exposure.reset(&mut settings);

        assert_eq!(settings.exposure, Settings::default().exposure);
        assert_eq!(settings.contrast, 2.0);
    }

    #[test]
    fn resetting_either_peak_field_restores_the_automatic_mode() {
        let mut settings = Settings::default();
        Field::AutomaticPeak.flip(&mut settings);
        Field::PeakNits.set_fraction(&mut settings, 0.9);

        Field::PeakNits.reset(&mut settings);

        assert_eq!(settings.hdr_peak_nits, PeakNits::Auto);
    }

    #[test]
    fn selecting_an_index_sets_that_profile() {
        let mut settings = Settings::default();

        for (index, profile) in LuminanceProfile::ALL.into_iter().enumerate() {
            Field::LuminanceProfile.set_choice(&mut settings, index);

            assert_eq!(settings.luminance_profile, profile);
        }
    }

    #[test]
    fn an_index_outside_the_list_is_ignored() {
        let mut settings = Settings::default();

        Field::LuminanceProfile.set_choice(&mut settings, LuminanceProfile::COUNT);

        assert_eq!(settings.luminance_profile, LuminanceProfile::Neutral);
    }

    #[test]
    fn selecting_a_profile_leaves_the_exposure_alone() {
        let mut settings = Settings {
            exposure: 0.25,
            ..Settings::default()
        };

        Field::LuminanceProfile.set_choice(&mut settings, 2);

        assert_eq!(settings.exposure, 0.25);
        assert_eq!(settings.effective_exposure(), 0.75);
    }

    #[test]
    fn resetting_the_profile_restores_the_neutral_one() {
        let mut settings = Settings {
            luminance_profile: LuminanceProfile::Low,
            ..Settings::default()
        };

        Field::LuminanceProfile.reset(&mut settings);

        assert_eq!(settings.luminance_profile, LuminanceProfile::Neutral);
    }
}
