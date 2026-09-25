use super::Field;
use crate::settings::{LuminanceProfile, PeakNits, Settings};

const PEAK: f32 = 1000.0;

#[test]
fn sliders_report_their_position_within_the_range() {
    let settings = Settings {
        saturation: 0.5,
        ..Settings::default()
    };

    assert_eq!(Field::Saturation.fraction(&settings, PEAK), 0.25);
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

    assert_eq!(Field::Temperature.text(&settings, PEAK), "6500");
    assert_eq!(Field::PaperWhite.text(&settings, PEAK), "203");
}

#[test]
fn unit_scale_parameters_are_shown_with_two_decimals() {
    let settings = Settings::default();

    assert_eq!(Field::Exposure.text(&settings, PEAK), "-0.06");
    assert_eq!(Field::HighlightRolloff.text(&settings, PEAK), "0.18");
}

#[test]
fn the_automatic_peak_shows_the_value_in_effect() {
    let settings = Settings::default();

    assert_eq!(Field::PeakNits.text(&settings, 1499.0), "auto 1499");
    assert_eq!(Field::PeakNits.text(&settings, 800.0), "auto 800");
}

#[test]
fn a_fixed_peak_ignores_what_the_display_reported() {
    let settings = Settings {
        hdr_peak_nits: PeakNits::Fixed(1100.0),
        ..Settings::default()
    };

    assert_eq!(Field::PeakNits.text(&settings, 1499.0), "1100");
    assert_eq!(Field::PeakNits.value(&settings, 1499.0), 1100.0);
}

#[test]
fn the_automatic_peak_slider_follows_the_value_in_effect() {
    let settings = Settings::default();

    let reported = Field::PeakNits.fraction(&settings, 1499.0);
    let fallback = Field::PeakNits.fraction(&settings, 1000.0);

    assert!(reported > fallback);
    assert_eq!(Field::PeakNits.value(&settings, 1499.0), 1499.0);
}

#[test]
fn switches_are_shown_as_words() {
    let mut settings = Settings::default();
    assert_eq!(Field::Enabled.text(&settings, PEAK), "sim");

    settings.enabled = false;

    assert_eq!(Field::Enabled.text(&settings, PEAK), "nao");
}

#[test]
fn a_locked_peak_slider_still_reports_a_position() {
    let settings = Settings::default();

    assert!(Field::PeakNits.fraction(&settings, PEAK) > 0.0);
    assert!(!Field::PeakNits.is_editable(&settings));
}

#[test]
fn the_profile_row_shows_the_profile_in_force() {
    for profile in [
        LuminanceProfile::Low,
        LuminanceProfile::Neutral,
        LuminanceProfile::High,
    ] {
        let settings = Settings {
            luminance_profile: profile,
            ..Settings::default()
        };

        assert_eq!(
            Field::LuminanceProfile.text(&settings, PEAK),
            profile.label()
        );
    }
}

#[test]
fn only_the_profile_row_reports_a_choice() {
    let settings = Settings {
        luminance_profile: LuminanceProfile::High,
        ..Settings::default()
    };

    assert_eq!(
        Field::LuminanceProfile.choice(&settings),
        LuminanceProfile::High.index()
    );
    assert_eq!(Field::Exposure.choice(&settings), 0);
    assert_eq!(Field::Enabled.choice(&settings), 0);
}

#[test]
fn the_profile_row_is_editable_and_has_no_slider_position() {
    let settings = Settings::default();

    assert!(Field::LuminanceProfile.is_editable(&settings));
    assert_eq!(Field::LuminanceProfile.fraction(&settings, PEAK), 0.0);
}
