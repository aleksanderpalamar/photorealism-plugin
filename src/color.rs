const BT709_LUMINANCE: [f32; 3] = [0.2126, 0.7152, 0.0722];
const BT2020_LUMINANCE: [f32; 3] = [0.2627, 0.6780, 0.0593];

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OutputMode {
    Sdr,
    Hdr10,
    ScRgb,
}

impl OutputMode {
    pub fn shader_value(self) -> f32 {
        match self {
            Self::Sdr => 0.0,
            Self::Hdr10 => 1.0,
            Self::ScRgb => 2.0,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Sdr => "SDR",
            Self::Hdr10 => "HDR10-PQ",
            Self::ScRgb => "scRGB",
        }
    }

    pub fn luminance_weights(self) -> [f32; 3] {
        match self {
            Self::Hdr10 => BT2020_LUMINANCE,
            Self::Sdr | Self::ScRgb => BT709_LUMINANCE,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{BT2020_LUMINANCE, BT709_LUMINANCE, OutputMode};

    #[test]
    fn hdr10_uses_the_bt2020_primaries() {
        assert_eq!(OutputMode::Hdr10.luminance_weights(), BT2020_LUMINANCE);
    }

    #[test]
    fn sdr_and_scrgb_use_the_bt709_primaries() {
        assert_eq!(OutputMode::Sdr.luminance_weights(), BT709_LUMINANCE);
        assert_eq!(OutputMode::ScRgb.luminance_weights(), BT709_LUMINANCE);
    }

    #[test]
    fn every_set_of_weights_preserves_neutral_grey() {
        for mode in [OutputMode::Sdr, OutputMode::Hdr10, OutputMode::ScRgb] {
            let total: f32 = mode.luminance_weights().iter().sum();
            assert!((total - 1.0).abs() < 1e-4, "{}", mode.name());
        }
    }

    #[test]
    fn each_mode_has_its_own_shader_value() {
        assert_eq!(OutputMode::Sdr.shader_value(), 0.0);
        assert_eq!(OutputMode::Hdr10.shader_value(), 1.0);
        assert_eq!(OutputMode::ScRgb.shader_value(), 2.0);
    }
}
