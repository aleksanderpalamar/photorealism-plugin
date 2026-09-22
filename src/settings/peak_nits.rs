use std::fmt;

const MINIMUM_NITS: f32 = 400.0;
const MAXIMUM_NITS: f32 = 10_000.0;
const DEFAULT_NITS: f32 = 1000.0;
const MINIMUM_REPORTED_NITS: f32 = 250.0;
const AUTO_KEYWORD: &str = "auto";

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PeakNits {
    Auto,
    Fixed(f32),
}

impl PeakNits {
    pub fn parse(value: &str, fallback: Self) -> Self {
        if value == AUTO_KEYWORD {
            return Self::Auto;
        }
        let Ok(parsed) = value.parse::<f32>() else {
            return fallback;
        };
        Self::Fixed(parsed.clamp(MINIMUM_NITS, MAXIMUM_NITS))
    }

    pub fn resolve(self, reported: Option<f32>) -> f32 {
        match self {
            Self::Fixed(nits) => nits,
            Self::Auto => reported
                .filter(|nits| *nits >= MINIMUM_REPORTED_NITS)
                .map_or(DEFAULT_NITS, |nits| nits.clamp(MINIMUM_NITS, MAXIMUM_NITS)),
        }
    }
}

impl fmt::Display for PeakNits {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Auto => formatter.write_str(AUTO_KEYWORD),
            Self::Fixed(nits) => write!(formatter, "{nits:.0}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PeakNits;

    #[test]
    fn parses_the_automatic_keyword() {
        assert_eq!(PeakNits::parse("auto", PeakNits::Fixed(1000.0)), PeakNits::Auto);
    }

    #[test]
    fn parses_and_clamps_explicit_values() {
        assert_eq!(PeakNits::parse("1200", PeakNits::Auto), PeakNits::Fixed(1200.0));
        assert_eq!(PeakNits::parse("20000", PeakNits::Auto), PeakNits::Fixed(10_000.0));
        assert_eq!(PeakNits::parse("100", PeakNits::Auto), PeakNits::Fixed(400.0));
    }

    #[test]
    fn keeps_the_fallback_for_invalid_values() {
        assert_eq!(PeakNits::parse("brilhante", PeakNits::Auto), PeakNits::Auto);
    }

    #[test]
    fn explicit_values_ignore_what_the_display_reports() {
        assert_eq!(PeakNits::Fixed(800.0).resolve(Some(1400.0)), 800.0);
    }

    #[test]
    fn automatic_uses_the_reported_peak() {
        assert_eq!(PeakNits::Auto.resolve(Some(1400.0)), 1400.0);
    }

    #[test]
    fn automatic_clamps_the_reported_peak() {
        assert_eq!(PeakNits::Auto.resolve(Some(300.0)), 400.0);
        assert_eq!(PeakNits::Auto.resolve(Some(20_000.0)), 10_000.0);
    }

    #[test]
    fn automatic_falls_back_when_the_display_reports_sdr_luminance() {
        assert_eq!(PeakNits::Auto.resolve(Some(80.0)), 1000.0);
        assert_eq!(PeakNits::Auto.resolve(None), 1000.0);
    }

    #[test]
    fn displays_the_configured_form() {
        assert_eq!(PeakNits::Auto.to_string(), "auto");
        assert_eq!(PeakNits::Fixed(1000.0).to_string(), "1000");
    }
}
