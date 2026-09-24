use crate::settings::{PeakNits, Settings};

pub fn serialize(settings: &Settings) -> String {
    format!(
        "enabled={}\nforce_hdr={}\nexposure={}\ncontrast={}\nsaturation={}\n\
         temperature={}\ntint={}\nhighlight_rolloff={}\n\
         hdr_paper_white_nits={}\nhdr_peak_nits={}\n",
        settings.enabled,
        settings.force_hdr,
        settings.exposure,
        settings.contrast,
        settings.saturation,
        settings.temperature,
        settings.tint,
        settings.highlight_rolloff,
        settings.hdr_paper_white_nits,
        peak(settings.hdr_peak_nits),
    )
}

fn peak(nits: PeakNits) -> String {
    match nits {
        PeakNits::Auto => PeakNits::Auto.to_string(),
        PeakNits::Fixed(value) => value.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::serialize;
    use crate::settings::{PeakNits, Settings};

    fn round_trip(settings: Settings) -> Settings {
        Settings::parse(&serialize(&settings))
    }

    #[test]
    fn the_default_configuration_survives_a_round_trip() {
        assert_eq!(round_trip(Settings::default()), Settings::default());
    }

    #[test]
    fn edited_values_survive_a_round_trip() {
        let settings = Settings {
            enabled: false,
            force_hdr: false,
            exposure: -0.375,
            contrast: 1.07,
            saturation: 1.125,
            temperature: 7200.0,
            tint: -0.25,
            highlight_rolloff: 0.35,
            hdr_paper_white_nits: 220.0,
            hdr_peak_nits: PeakNits::Fixed(1499.0),
        };

        assert_eq!(round_trip(settings), settings);
    }

    #[test]
    fn a_fractional_peak_is_not_rounded_away() {
        let settings = Settings {
            hdr_peak_nits: PeakNits::Fixed(1499.5),
            ..Settings::default()
        };

        assert!(serialize(&settings).contains("hdr_peak_nits=1499.5\n"));
        assert_eq!(round_trip(settings), settings);
    }

    #[test]
    fn the_automatic_peak_is_written_as_a_keyword() {
        let text = serialize(&Settings::default());

        assert!(text.contains("hdr_peak_nits=auto\n"));
    }

    #[test]
    fn every_supported_key_is_written() {
        let text = serialize(&Settings::default());
        let keys = [
            "enabled",
            "force_hdr",
            "exposure",
            "contrast",
            "saturation",
            "temperature",
            "tint",
            "highlight_rolloff",
            "hdr_paper_white_nits",
            "hdr_peak_nits",
        ];

        for key in keys {
            assert!(text.contains(&format!("{key}=")), "{key}");
        }
    }
}
