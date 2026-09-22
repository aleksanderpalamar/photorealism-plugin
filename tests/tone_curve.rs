const SHADER: &str = include_str!("../shaders/photorealism.hlsl");
const KNEE_EXPRESSION: &str = "float knee = limit * lerp(1.0, 0.5, amount);";
const TOLERANCE: f32 = 1e-4;
const CLAMP_EPSILON: f32 = 2e-4;

fn knee(limit: f32, amount: f32) -> f32 {
    limit * (1.0 - 0.5 * amount.clamp(0.0, 1.0))
}

fn rolloff(color: f32, limit: f32, amount: f32) -> f32 {
    let knee = knee(limit, amount);
    let headroom = (limit - knee).max(0.0001);
    let excess = (color - knee).max(0.0);
    let compressed = knee + headroom * (1.0 - (-excess / headroom).exp());
    color.min(compressed)
}

fn relative_peak(peak_nits: f32, paper_white_nits: f32) -> f32 {
    peak_nits / paper_white_nits
}

#[test]
fn the_shader_uses_the_same_knee_as_this_mirror() {
    assert!(SHADER.contains(KNEE_EXPRESSION));
}

#[test]
fn the_sdr_knee_responds_to_the_amount() {
    assert!((knee(1.0, 0.0) - 1.0).abs() < TOLERANCE);
    assert!((knee(1.0, 0.35) - 0.825).abs() < TOLERANCE);
    assert!((knee(1.0, 1.0) - 0.5).abs() < TOLERANCE);
}

#[test]
fn the_hdr_knee_of_the_validated_configuration_is_preserved() {
    let limit = relative_peak(1000.0, 203.0);

    assert!((knee(limit, 0.18) - 4.482759).abs() < TOLERANCE);
}

#[test]
fn a_low_peak_display_keeps_the_knee_proportional() {
    let limit = relative_peak(400.0, 203.0);

    assert!((knee(limit, 1.0) / limit - 0.5).abs() < TOLERANCE);
}

#[test]
fn a_zero_amount_clips_at_the_limit() {
    assert!((rolloff(3.0, 1.0, 0.0) - 1.0).abs() < CLAMP_EPSILON);
    assert!((rolloff(0.5, 1.0, 0.0) - 0.5).abs() < TOLERANCE);
}

#[test]
fn no_amount_pushes_the_output_past_the_limit() {
    let limits = [1.0, relative_peak(400.0, 203.0), relative_peak(4000.0, 203.0)];
    for limit in limits {
        for step in 0..=10 {
            let amount = step as f32 / 10.0;
            assert!(rolloff(limit * 4.0, limit, amount) <= limit + CLAMP_EPSILON);
        }
    }
}

#[test]
fn values_below_the_knee_are_untouched() {
    let limit = relative_peak(1000.0, 203.0);
    let below = knee(limit, 0.35) - 0.1;

    assert!((rolloff(below, limit, 0.35) - below).abs() < TOLERANCE);
}
