use super::field::{Control, Field};
use crate::settings::{PeakNits, Settings};

const DEFAULT_FIXED_PEAK: f32 = 1000.0;
const UNIT_DECIMALS: f32 = 1000.0;

impl Field {
    pub fn is_editable(self, settings: &Settings) -> bool {
        let Self::PeakNits = self else {
            return true;
        };
        !matches!(settings.hdr_peak_nits, PeakNits::Auto)
    }

    pub fn value(self, settings: &Settings) -> f32 {
        match self {
            Self::Enabled | Self::AutomaticPeak => 0.0,
            Self::Exposure => settings.exposure,
            Self::Contrast => settings.contrast,
            Self::Saturation => settings.saturation,
            Self::Temperature => settings.temperature,
            Self::Tint => settings.tint,
            Self::HighlightRolloff => settings.highlight_rolloff,
            Self::PaperWhite => settings.hdr_paper_white_nits,
            Self::PeakNits => fixed_peak(settings),
        }
    }

    pub fn switch(self, settings: &Settings) -> bool {
        match self {
            Self::Enabled => settings.enabled,
            Self::AutomaticPeak => matches!(settings.hdr_peak_nits, PeakNits::Auto),
            _ => false,
        }
    }

    pub fn fraction(self, settings: &Settings) -> f32 {
        let Control::Slider(range) = self.control() else {
            return 0.0;
        };
        range.fraction_of(self.value(settings))
    }

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

    pub fn text(self, settings: &Settings) -> String {
        match self {
            Self::Enabled | Self::AutomaticPeak => switch_text(self.switch(settings)),
            Self::Temperature | Self::PaperWhite => format!("{:.0}", self.value(settings)),
            Self::PeakNits => settings.hdr_peak_nits.to_string(),
            _ => format!("{:.2}", self.value(settings)),
        }
    }
}

fn switch_text(on: bool) -> String {
    let text = if on { "sim" } else { "nao" };
    text.to_owned()
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

fn fixed_peak(settings: &Settings) -> f32 {
    match settings.hdr_peak_nits {
        PeakNits::Fixed(nits) => nits,
        PeakNits::Auto => DEFAULT_FIXED_PEAK,
    }
}

#[cfg(test)]
mod tests {
    use super::Field;
    use crate::settings::{PeakNits, Settings};

    #[test]
    fn a_fraction_written_to_a_slider_can_be_read_back() {
        let mut settings = Settings::default();

        Field::Saturation.set_fraction(&mut settings, 0.25);

        assert_eq!(settings.saturation, 0.5);
        assert_eq!(Field::Saturation.fraction(&settings), 0.25);
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
    fn sliders_report_their_position_within_the_range() {
        let settings = Settings {
            saturation: 0.5,
            ..Settings::default()
        };

        assert_eq!(Field::Saturation.fraction(&settings), 0.25);
    }

    #[test]
    fn switches_report_their_state() {
        let settings = Settings::default();

        assert!(Field::Enabled.switch(&settings));
        assert!(Field::AutomaticPeak.switch(&settings));
        assert!(!Field::Exposure.switch(&settings));
    }

    #[test]
    fn the_peak_slider_is_locked_while_the_peak_is_automatic() {
        let mut settings = Settings::default();
        assert!(!Field::PeakNits.is_editable(&settings));

        settings.hdr_peak_nits = PeakNits::Fixed(1200.0);

        assert!(Field::PeakNits.is_editable(&settings));
        assert!(Field::Exposure.is_editable(&settings));
    }

    #[test]
    fn nits_and_temperature_are_shown_without_decimals() {
        let settings = Settings::default();

        assert_eq!(Field::Temperature.text(&settings), "6500");
        assert_eq!(Field::PaperWhite.text(&settings), "203");
    }

    #[test]
    fn unit_scale_parameters_are_shown_with_two_decimals() {
        let settings = Settings::default();

        assert_eq!(Field::Exposure.text(&settings), "-0.06");
        assert_eq!(Field::HighlightRolloff.text(&settings), "0.18");
    }

    #[test]
    fn the_peak_shows_the_keyword_while_automatic() {
        let mut settings = Settings::default();
        assert_eq!(Field::PeakNits.text(&settings), "auto");

        settings.hdr_peak_nits = PeakNits::Fixed(1499.0);

        assert_eq!(Field::PeakNits.text(&settings), "1499");
    }

    #[test]
    fn switches_are_shown_as_words() {
        let mut settings = Settings::default();
        assert_eq!(Field::Enabled.text(&settings), "sim");

        settings.enabled = false;

        assert_eq!(Field::Enabled.text(&settings), "nao");
    }

    #[test]
    fn a_locked_peak_slider_still_reports_a_position() {
        let settings = Settings::default();

        assert!(Field::PeakNits.fraction(&settings) > 0.0);
    }
}
