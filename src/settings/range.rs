#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Range {
    pub minimum: f32,
    pub maximum: f32,
}

impl Range {
    const fn new(minimum: f32, maximum: f32) -> Self {
        Self { minimum, maximum }
    }

    pub fn clamp(self, value: f32) -> f32 {
        value.clamp(self.minimum, self.maximum)
    }

    pub fn fraction_of(self, value: f32) -> f32 {
        let span = self.maximum - self.minimum;
        if span <= 0.0 {
            return 0.0;
        }
        ((value - self.minimum) / span).clamp(0.0, 1.0)
    }
}

pub const EXPOSURE: Range = Range::new(-4.0, 4.0);
pub const CONTRAST: Range = Range::new(0.25, 2.0);
pub const SATURATION: Range = Range::new(0.0, 2.0);
pub const TEMPERATURE: Range = Range::new(2000.0, 12_000.0);
pub const TINT: Range = Range::new(-1.0, 1.0);
pub const HIGHLIGHT_ROLLOFF: Range = Range::new(0.0, 1.0);
pub const PAPER_WHITE_NITS: Range = Range::new(80.0, 500.0);
pub const PEAK_NITS: Range = Range::new(400.0, 10_000.0);

#[cfg(test)]
mod tests {
    use super::{
        CONTRAST, EXPOSURE, HIGHLIGHT_ROLLOFF, PAPER_WHITE_NITS, PEAK_NITS, SATURATION,
        TEMPERATURE, TINT,
    };

    #[test]
    fn clamps_to_the_declared_limits() {
        assert_eq!(EXPOSURE.clamp(9.0), 4.0);
        assert_eq!(EXPOSURE.clamp(-9.0), -4.0);
        assert_eq!(EXPOSURE.clamp(0.5), 0.5);
    }

    #[test]
    fn converts_a_value_into_a_fraction() {
        assert_eq!(EXPOSURE.fraction_of(-4.0), 0.0);
        assert_eq!(EXPOSURE.fraction_of(4.0), 1.0);
        assert_eq!(EXPOSURE.fraction_of(0.0), 0.5);
        assert_eq!(EXPOSURE.fraction_of(100.0), 1.0);
    }

    #[test]
    fn each_parameter_declares_a_usable_range() {
        let all = [
            EXPOSURE,
            CONTRAST,
            SATURATION,
            TEMPERATURE,
            TINT,
            HIGHLIGHT_ROLLOFF,
            PAPER_WHITE_NITS,
            PEAK_NITS,
        ];

        for range in all {
            assert!(range.maximum > range.minimum);
        }
    }
}
