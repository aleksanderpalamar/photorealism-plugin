const SHADER: &str = include_str!("../shaders/photorealism.hlsl");
const OVERLAY: &str = include_str!("../shaders/overlay.hlsl");
const REGISTER_COMPONENTS: usize = 4;

const EXPECTED_FIELDS: [(&str, &str); 14] = [
    ("float", "Exposure"),
    ("float", "Contrast"),
    ("float", "Saturation"),
    ("float", "Temperature"),
    ("float", "Tint"),
    ("float", "HighlightRolloff"),
    ("float", "InputNeedsSrgbDecode"),
    ("float", "OutputNeedsSrgbEncode"),
    ("float", "OutputMode"),
    ("float", "HdrPaperWhiteNits"),
    ("float", "HdrPeakNits"),
    ("float", "Padding"),
    ("float3", "LuminanceWeights"),
    ("float", "WeightsPadding"),
];

fn settings_fields() -> Vec<(String, String)> {
    fields(SHADER, "cbuffer SettingsBuffer")
}

fn overlay_fields() -> Vec<(String, String)> {
    fields(OVERLAY, "cbuffer OverlayBuffer")
}

fn overlay_vertex_fields() -> Vec<(String, String)> {
    fields(OVERLAY, "struct OverlayVertex")
}

fn fields(source: &str, declaration: &str) -> Vec<(String, String)> {
    let start = source.find(declaration).expect("declaracao");
    let body = &source[start..];
    let end = body.find("};").expect("fim da declaracao");
    body[..end]
        .lines()
        .filter_map(|line| line.trim().strip_suffix(';'))
        .filter_map(|line| line.split_once(' '))
        .map(|(kind, name)| (kind.to_owned(), name.to_owned()))
        .collect()
}

fn components(kind: &str) -> usize {
    match kind {
        "float" => 1,
        "float2" => 2,
        "float3" => 3,
        "float4" => 4,
        other => panic!("tipo nao suportado no cbuffer: {other}"),
    }
}

#[test]
fn the_constant_buffer_matches_the_rust_structure() {
    let fields = settings_fields();
    let expected: Vec<(String, String)> = EXPECTED_FIELDS
        .iter()
        .map(|(kind, name)| ((*kind).to_owned(), (*name).to_owned()))
        .collect();

    assert_eq!(fields, expected);
}

#[test]
fn no_field_straddles_a_constant_register() {
    let mut offset = 0;
    for (kind, name) in settings_fields() {
        let size = components(&kind);
        let position = offset % REGISTER_COMPONENTS;
        assert!(
            size == 1 || position + size <= REGISTER_COMPONENTS,
            "{name} atravessa a fronteira do registrador"
        );
        offset += size;
    }
}

#[test]
fn the_constant_buffer_fills_whole_registers() {
    let total: usize = settings_fields()
        .iter()
        .map(|(kind, _)| components(kind))
        .sum();

    assert_eq!(total % REGISTER_COMPONENTS, 0);
}

const EXPECTED_OVERLAY_FIELDS: [(&str, &str); 4] = [
    ("float", "OutputMode"),
    ("float", "HdrPaperWhiteNits"),
    ("float", "HdrPeakNits"),
    ("float", "OutputNeedsSrgbEncode"),
];

const EXPECTED_VERTEX_FIELDS: [(&str, &str); 3] = [
    ("float4", "position_uv"),
    ("float4", "color"),
    ("float4", "shape"),
];

fn named(expected: &[(&str, &str)]) -> Vec<(String, String)> {
    expected
        .iter()
        .map(|(kind, name)| ((*kind).to_owned(), (*name).to_owned()))
        .collect()
}

#[test]
fn the_overlay_constant_buffer_matches_the_rust_structure() {
    assert_eq!(overlay_fields(), named(&EXPECTED_OVERLAY_FIELDS));
}

#[test]
fn the_overlay_constant_buffer_fills_one_whole_register() {
    let total: usize = overlay_fields()
        .iter()
        .map(|(kind, _)| components(kind))
        .sum();

    assert_eq!(total, REGISTER_COMPONENTS);
}

#[test]
fn the_overlay_vertex_matches_the_rust_structure() {
    assert_eq!(overlay_vertex_fields(), named(&EXPECTED_VERTEX_FIELDS));
}

#[test]
fn every_overlay_vertex_field_fills_a_whole_register() {
    for (kind, name) in overlay_vertex_fields() {
        assert_eq!(components(&kind), REGISTER_COMPONENTS, "{name}");
    }
}
