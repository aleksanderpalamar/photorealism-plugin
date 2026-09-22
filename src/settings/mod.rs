mod peak_nits;

pub use peak_nits::PeakNits;

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
        }
    }
}

impl Settings {
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
            "exposure" => self.exposure = parse_f32(value, -4.0, 4.0, self.exposure),
            "contrast" => self.contrast = parse_f32(value, 0.25, 2.0, self.contrast),
            "saturation" => self.saturation = parse_f32(value, 0.0, 2.0, self.saturation),
            "temperature" => self.temperature = parse_f32(value, 2000.0, 12000.0, self.temperature),
            "tint" => self.tint = parse_f32(value, -1.0, 1.0, self.tint),
            "highlight_rolloff" => {
                self.highlight_rolloff = parse_f32(value, 0.0, 1.0, self.highlight_rolloff)
            }
            "hdr_paper_white_nits" => {
                self.hdr_paper_white_nits = parse_f32(value, 80.0, 500.0, self.hdr_paper_white_nits)
            }
            "hdr_peak_nits" => self.hdr_peak_nits = PeakNits::parse(value, self.hdr_peak_nits),
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

fn parse_f32(value: &str, minimum: f32, maximum: f32, fallback: f32) -> f32 {
    let Ok(parsed) = value.parse::<f32>() else {
        return fallback;
    };
    parsed.clamp(minimum, maximum)
}

#[cfg(test)]
mod tests {
    use super::{PeakNits, Settings};

    #[test]
    fn parses_supported_values() {
        let settings = Settings::parse(
            "enabled=false\nforce_hdr=false\nexposure=1.25\ncontrast=1.1\nsaturation=0.8\n\
             temperature=7200\ntint=-0.2\nhighlight_rolloff=0.4\n\
             hdr_paper_white_nits=250\nhdr_peak_nits=1200",
        );

        assert!(!settings.enabled);
        assert!(!settings.force_hdr);
        assert_eq!(settings.exposure, 1.25);
        assert_eq!(settings.contrast, 1.1);
        assert_eq!(settings.saturation, 0.8);
        assert_eq!(settings.temperature, 7200.0);
        assert_eq!(settings.tint, -0.2);
        assert_eq!(settings.highlight_rolloff, 0.4);
        assert_eq!(settings.hdr_paper_white_nits, 250.0);
        assert_eq!(settings.hdr_peak_nits, PeakNits::Fixed(1200.0));
    }

    #[test]
    fn clamps_values_to_safe_ranges() {
        let settings = Settings::parse(
            "exposure=8\ncontrast=-1\ntemperature=500\n\
             hdr_paper_white_nits=20\nhdr_peak_nits=20000",
        );

        assert_eq!(settings.exposure, 4.0);
        assert_eq!(settings.contrast, 0.25);
        assert_eq!(settings.temperature, 2000.0);
        assert_eq!(settings.hdr_paper_white_nits, 80.0);
        assert_eq!(settings.hdr_peak_nits, PeakNits::Fixed(10_000.0));
    }

    #[test]
    fn reads_the_automatic_peak_keyword() {
        let settings = Settings::parse("hdr_peak_nits=1200\nhdr_peak_nits=auto");

        assert_eq!(settings.hdr_peak_nits, PeakNits::Auto);
    }

    #[test]
    fn ignores_invalid_and_unknown_values() {
        let settings = Settings::parse("enabled=yes\ncontrast=invalid\nfuture=10");

        assert_eq!(settings, Settings::default());
    }
}
