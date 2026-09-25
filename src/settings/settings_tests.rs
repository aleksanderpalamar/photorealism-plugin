use super::{LuminanceProfile, PeakNits, Settings};

#[test]
fn parses_supported_values() {
    let settings = Settings::parse(
        "enabled=false\nforce_hdr=false\nexposure=1.25\ncontrast=1.1\nsaturation=0.8\n\
         temperature=7200\ntint=-0.2\nhighlight_rolloff=0.4\n\
         hdr_paper_white_nits=250\nhdr_peak_nits=1200\nluminance_profile=high",
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
    assert_eq!(settings.luminance_profile, LuminanceProfile::High);
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
    let settings =
        Settings::parse("enabled=yes\ncontrast=invalid\nfuture=10\nluminance_profile=media");

    assert_eq!(settings, Settings::default());
}

#[test]
fn the_default_profile_leaves_the_exposure_untouched() {
    let settings = Settings::default();

    assert_eq!(settings.luminance_profile, LuminanceProfile::Neutral);
    assert_eq!(settings.effective_exposure(), settings.exposure);
}

#[test]
fn the_profile_offsets_the_exposure_in_both_directions() {
    let base = Settings {
        exposure: 0.5,
        ..Settings::default()
    };

    let brighter = Settings {
        luminance_profile: LuminanceProfile::High,
        ..base
    };
    let darker = Settings {
        luminance_profile: LuminanceProfile::Low,
        ..base
    };

    assert_eq!(brighter.effective_exposure(), 1.0);
    assert_eq!(darker.effective_exposure(), 0.0);
}

#[test]
fn the_offset_saturates_at_the_exposure_limits() {
    let brightest = Settings {
        exposure: 3.8,
        luminance_profile: LuminanceProfile::High,
        ..Settings::default()
    };
    let darkest = Settings {
        exposure: -3.8,
        luminance_profile: LuminanceProfile::Low,
        ..Settings::default()
    };

    assert_eq!(brightest.effective_exposure(), 4.0);
    assert_eq!(darkest.effective_exposure(), -4.0);
}

const SHADER: &str = include_str!("../../shaders/photorealism.hlsl");
const EXPOSURE_EXPRESSION: &str = "color = max(color * exp2(Exposure), 0.0);";
const CONTRAST_EXPRESSION: &str = "color = pivot * pow(max(color, 0.000001) / pivot, Contrast);";
const PIVOT: f32 = 0.18;
const PAPER_WHITE: f32 = 203.0;
const TOLERANCE: f32 = 0.05;

fn graded(relative: f32, settings: Settings) -> f32 {
    let exposed = relative * settings.effective_exposure().exp2();
    PIVOT * (exposed.max(0.000001) / PIVOT).powf(settings.contrast) * PAPER_WHITE
}

#[test]
fn the_shader_still_grades_the_way_this_mirror_does() {
    assert!(SHADER.contains(EXPOSURE_EXPRESSION));
    assert!(SHADER.contains(CONTRAST_EXPRESSION));
}

#[test]
fn the_documented_luminance_matches_the_shipped_defaults() {
    let white = [135.9, 191.5, 269.9];
    let grey = [24.9, 35.1, 49.4];

    for (index, profile) in LuminanceProfile::ALL.into_iter().enumerate() {
        let settings = Settings {
            luminance_profile: profile,
            ..Settings::default()
        };

        assert!(
            (graded(1.0, settings) - white[index]).abs() < TOLERANCE,
            "{profile:?}: {}",
            graded(1.0, settings)
        );
        assert!(
            (graded(PIVOT, settings) - grey[index]).abs() < TOLERANCE,
            "{profile:?}: {}",
            graded(PIVOT, settings)
        );
    }
}
