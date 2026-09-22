use super::resources::FrameResources;
use crate::settings::Settings;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ShaderSettings {
    exposure: f32,
    contrast: f32,
    saturation: f32,
    temperature: f32,
    tint: f32,
    highlight_rolloff: f32,
    input_needs_srgb_decode: f32,
    output_needs_srgb_encode: f32,
    output_mode: f32,
    hdr_paper_white_nits: f32,
    hdr_peak_nits: f32,
    padding: f32,
    luminance_weights: [f32; 3],
    weights_padding: f32,
}

impl ShaderSettings {
    pub fn new(value: Settings, frame: &FrameResources, reported_peak: Option<f32>) -> Self {
        let conversion = if frame.manual_srgb { 1.0 } else { 0.0 };
        Self {
            exposure: value.exposure,
            contrast: value.contrast,
            saturation: value.saturation,
            temperature: value.temperature,
            tint: value.tint,
            highlight_rolloff: value.highlight_rolloff,
            input_needs_srgb_decode: conversion,
            output_needs_srgb_encode: conversion,
            output_mode: frame.output_mode.shader_value(),
            hdr_paper_white_nits: value.hdr_paper_white_nits,
            hdr_peak_nits: value.hdr_peak_nits.resolve(reported_peak),
            padding: 0.0,
            luminance_weights: frame.output_mode.luminance_weights(),
            weights_padding: 0.0,
        }
    }
}
