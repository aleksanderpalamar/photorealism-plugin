mod luminance_profile;
mod peak_nits;
pub mod range;

pub use luminance_profile::LuminanceProfile;
pub use peak_nits::PeakNits;
pub use range::Range;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub enabled: bool,
    pub force_hdr: bool,
    pub exposure: f32,
    pub contrast: f32,
    pub saturation: f32,
    pub temperature: f32,
    pub tint: f32,
    pub highlight_rolloff: f32,
    pub hdr_paper_white_nits: f32,
    pub hdr_peak_nits: PeakNits,
    pub luminance_profile: LuminanceProfile,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            enabled: true,
            force_hdr: true,
            exposure: -0.06,
            contrast: 0.99,
            saturation: 1.0,
            temperature: 6500.0,
            tint: 0.0,
            highlight_rolloff: 0.18,
            hdr_paper_white_nits: 203.0,
            hdr_peak_nits: PeakNits::Auto,
            luminance_profile: LuminanceProfile::Neutral,
        }
    }
}

impl Settings {
    pub fn effective_exposure(self) -> f32 {
        range::EXPOSURE.clamp(self.exposure + self.luminance_profile.exposure_offset())
    }

    pub fn parse(contents: &str) -> Self {
        contents.lines().fold(Self::default(), Self::apply_line)
    }

    fn apply_line(mut settings: Self, line: &str) -> Self {
        let Some((key, value)) = parse_assignment(line) else {
            return settings;
        };
        settings.apply(key, value);
        settings
    }

    fn apply(&mut self, key: &str, value: &str) {
        match key {
            "enabled" => self.enabled = parse_bool(value).unwrap_or(self.enabled),
            "force_hdr" => self.force_hdr = parse_bool(value).unwrap_or(self.force_hdr),
            "exposure" => self.exposure = parse_f32(value, range::EXPOSURE, self.exposure),
            "contrast" => self.contrast = parse_f32(value, range::CONTRAST, self.contrast),
            "saturation" => self.saturation = parse_f32(value, range::SATURATION, self.saturation),
            "temperature" => {
                self.temperature = parse_f32(value, range::TEMPERATURE, self.temperature)
            }
            "tint" => self.tint = parse_f32(value, range::TINT, self.tint),
            "highlight_rolloff" => {
                self.highlight_rolloff =
                    parse_f32(value, range::HIGHLIGHT_ROLLOFF, self.highlight_rolloff)
            }
            "hdr_paper_white_nits" => {
                self.hdr_paper_white_nits =
                    parse_f32(value, range::PAPER_WHITE_NITS, self.hdr_paper_white_nits)
            }
            "hdr_peak_nits" => self.hdr_peak_nits = PeakNits::parse(value, self.hdr_peak_nits),
            "luminance_profile" => {
                self.luminance_profile = LuminanceProfile::parse(value, self.luminance_profile)
            }
            _ => {}
        }
    }
}

fn parse_assignment(line: &str) -> Option<(&str, &str)> {
    let content = line.split('#').next()?.trim();
    let (key, value) = content.split_once('=')?;
    Some((key.trim(), value.trim()))
}

fn parse_bool(value: &str) -> Option<bool> {
    match value {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

fn parse_f32(value: &str, range: Range, fallback: f32) -> f32 {
    let Ok(parsed) = value.parse::<f32>() else {
        return fallback;
    };
    range.clamp(parsed)
}

#[cfg(test)]
#[path = "settings_tests.rs"]
mod tests;
