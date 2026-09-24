use super::field::{Control, Field};
use crate::settings::{PeakNits, Settings};

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

    pub fn reset(self, settings: &mut Settings) {
        let defaults = Settings::default();
        match self {
            Self::Enabled => settings.enabled = defaults.enabled,
            Self::AutomaticPeak | Self::PeakNits => settings.hdr_peak_nits = defaults.hdr_peak_nits,
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
            Self::Enabled | Self::AutomaticPeak => {}
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
    use crate::settings::{PeakNits, Settings};

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
}
