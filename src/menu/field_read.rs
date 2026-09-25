use super::field::{Control, Field};
use crate::settings::{LuminanceProfile, PeakNits, Settings};

impl Field {
    pub fn is_editable(self, settings: &Settings) -> bool {
        let Self::PeakNits = self else {
            return true;
        };
        !matches!(settings.hdr_peak_nits, PeakNits::Auto)
    }

    pub fn value(self, settings: &Settings, resolved_peak: f32) -> f32 {
        match self {
            Self::Enabled | Self::AutomaticPeak | Self::LuminanceProfile => 0.0,
            Self::Exposure => settings.exposure,
            Self::Contrast => settings.contrast,
            Self::Saturation => settings.saturation,
            Self::Temperature => settings.temperature,
            Self::Tint => settings.tint,
            Self::HighlightRolloff => settings.highlight_rolloff,
            Self::PaperWhite => settings.hdr_paper_white_nits,
            Self::PeakNits => fixed_peak(settings, resolved_peak),
        }
    }

    pub fn switch(self, settings: &Settings) -> bool {
        match self {
            Self::Enabled => settings.enabled,
            Self::AutomaticPeak => matches!(settings.hdr_peak_nits, PeakNits::Auto),
            _ => false,
        }
    }

    pub fn choice_label(self, index: usize) -> &'static str {
        let Self::LuminanceProfile = self else {
            return "";
        };
        LuminanceProfile::ALL
            .get(index)
            .map_or("", |profile| profile.label())
    }

    pub fn choice(self, settings: &Settings) -> usize {
        match self {
            Self::LuminanceProfile => settings.luminance_profile.index(),
            _ => 0,
        }
    }

    pub fn fraction(self, settings: &Settings, resolved_peak: f32) -> f32 {
        let Control::Slider(range) = self.control() else {
            return 0.0;
        };
        range.fraction_of(self.value(settings, resolved_peak))
    }

    pub fn text(self, settings: &Settings, resolved_peak: f32) -> String {
        match self {
            Self::Enabled | Self::AutomaticPeak => switch_text(self.switch(settings)),
            Self::Temperature | Self::PaperWhite => {
                format!("{:.0}", self.value(settings, resolved_peak))
            }
            Self::LuminanceProfile => settings.luminance_profile.label().to_owned(),
            Self::PeakNits => peak_text(settings, resolved_peak),
            _ => format!("{:.2}", self.value(settings, resolved_peak)),
        }
    }
}

fn switch_text(on: bool) -> String {
    let text = if on { "sim" } else { "nao" };
    text.to_owned()
}

fn fixed_peak(settings: &Settings, resolved_peak: f32) -> f32 {
    match settings.hdr_peak_nits {
        PeakNits::Fixed(nits) => nits,
        PeakNits::Auto => resolved_peak,
    }
}

fn peak_text(settings: &Settings, resolved_peak: f32) -> String {
    match settings.hdr_peak_nits {
        PeakNits::Fixed(nits) => format!("{nits:.0}"),
        PeakNits::Auto => format!("auto {resolved_peak:.0}"),
    }
}

#[cfg(test)]
#[path = "field_read_tests.rs"]
mod tests;
