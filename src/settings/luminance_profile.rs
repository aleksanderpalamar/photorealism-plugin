use std::fmt;

const LOW_KEYWORD: &str = "low";
const NEUTRAL_KEYWORD: &str = "neutral";
const HIGH_KEYWORD: &str = "high";
const LOW_OFFSET: f32 = -0.5;
const NEUTRAL_OFFSET: f32 = 0.0;
const HIGH_OFFSET: f32 = 0.5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LuminanceProfile {
    Low,
    Neutral,
    High,
}

impl LuminanceProfile {
    pub const COUNT: usize = 3;

    pub fn parse(value: &str, fallback: Self) -> Self {
        match value {
            LOW_KEYWORD => Self::Low,
            NEUTRAL_KEYWORD => Self::Neutral,
            HIGH_KEYWORD => Self::High,
            _ => fallback,
        }
    }

    pub fn exposure_offset(self) -> f32 {
        match self {
            Self::Low => LOW_OFFSET,
            Self::Neutral => NEUTRAL_OFFSET,
            Self::High => HIGH_OFFSET,
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Low => Self::Neutral,
            Self::Neutral => Self::High,
            Self::High => Self::Low,
        }
    }

    pub fn index(self) -> usize {
        match self {
            Self::Low => 0,
            Self::Neutral => 1,
            Self::High => 2,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Low => "baixa",
            Self::Neutral => "neutra",
            Self::High => "alta",
        }
    }
}

impl fmt::Display for LuminanceProfile {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Low => LOW_KEYWORD,
            Self::Neutral => NEUTRAL_KEYWORD,
            Self::High => HIGH_KEYWORD,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::LuminanceProfile;

    const ALL: [LuminanceProfile; LuminanceProfile::COUNT] = [
        LuminanceProfile::Low,
        LuminanceProfile::Neutral,
        LuminanceProfile::High,
    ];

    #[test]
    fn each_keyword_selects_its_profile() {
        let other = LuminanceProfile::High;

        assert_eq!(LuminanceProfile::parse("low", other), LuminanceProfile::Low);
        assert_eq!(
            LuminanceProfile::parse("neutral", other),
            LuminanceProfile::Neutral
        );
        assert_eq!(
            LuminanceProfile::parse("high", LuminanceProfile::Low),
            LuminanceProfile::High
        );
    }

    #[test]
    fn an_unknown_keyword_keeps_the_fallback() {
        let fallback = LuminanceProfile::Neutral;

        assert_eq!(LuminanceProfile::parse("media", fallback), fallback);
        assert_eq!(LuminanceProfile::parse("", fallback), fallback);
        assert_eq!(LuminanceProfile::parse("LOW", fallback), fallback);
    }

    #[test]
    fn the_neutral_profile_does_not_move_the_exposure() {
        assert_eq!(LuminanceProfile::Neutral.exposure_offset(), 0.0);
    }

    #[test]
    fn the_offsets_grow_with_the_profile() {
        assert!(LuminanceProfile::Low.exposure_offset() < 0.0);
        assert!(LuminanceProfile::High.exposure_offset() > 0.0);
        assert_eq!(
            LuminanceProfile::Low.exposure_offset(),
            -LuminanceProfile::High.exposure_offset()
        );
    }

    #[test]
    fn advancing_three_times_returns_to_the_start() {
        for profile in ALL {
            assert_eq!(profile.next().next().next(), profile);
        }
    }

    #[test]
    fn advancing_walks_through_every_profile() {
        let mut seen = vec![LuminanceProfile::Low];
        let mut profile = LuminanceProfile::Low;

        for _ in 1..LuminanceProfile::COUNT {
            profile = profile.next();
            assert!(!seen.contains(&profile), "{profile:?}");
            seen.push(profile);
        }

        assert_eq!(seen.len(), LuminanceProfile::COUNT);
    }

    #[test]
    fn advancing_moves_the_index_by_one() {
        for profile in ALL {
            let expected = (profile.index() + 1) % LuminanceProfile::COUNT;

            assert_eq!(profile.next().index(), expected);
        }
    }

    #[test]
    fn every_index_stays_inside_the_count() {
        for profile in ALL {
            assert!(profile.index() < LuminanceProfile::COUNT);
        }
    }

    #[test]
    fn the_written_keyword_can_be_read_back() {
        for profile in ALL {
            assert_eq!(
                LuminanceProfile::parse(&profile.to_string(), profile.next()),
                profile
            );
        }
    }

    #[test]
    fn every_profile_has_its_own_label() {
        for profile in ALL {
            assert!(!profile.label().is_empty());
            assert_ne!(profile.label(), profile.next().label());
        }
    }
}
