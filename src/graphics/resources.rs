use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::Dxgi::Common::{
    DXGI_FORMAT, DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_FORMAT_B8G8R8A8_UNORM_SRGB,
    DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_FORMAT_R8G8B8A8_UNORM_SRGB, DXGI_FORMAT_R10G10B10A2_UNORM,
    DXGI_FORMAT_R16G16B16A16_FLOAT,
};

use super::constants::ShaderSettings;
use super::shaders::{self, Source};
use crate::color::OutputMode;

pub struct FrameResources {
    pub width: u32,
    pub height: u32,
    pub source: ID3D11Texture2D,
    pub source_view: ID3D11ShaderResourceView,
    pub output_view: ID3D11RenderTargetView,
    pub manual_srgb: bool,
    pub output_mode: OutputMode,
}

pub fn texture_description(texture: &ID3D11Texture2D) -> D3D11_TEXTURE2D_DESC {
    let mut description = D3D11_TEXTURE2D_DESC::default();
    unsafe { texture.GetDesc(&mut description) };
    description
}

pub fn create_frame_resources(
    device: &ID3D11Device,
    back_buffer: &ID3D11Texture2D,
    description: &D3D11_TEXTURE2D_DESC,
) -> windows::core::Result<FrameResources> {
    let mut source_description = *description;
    source_description.BindFlags = D3D11_BIND_SHADER_RESOURCE.0 as u32;
    source_description.MiscFlags = 0;
    let mut source = None;
    unsafe { device.CreateTexture2D(&source_description, None, Some(&mut source))? };
    let source = source.ok_or_else(windows::core::Error::from_win32)?;
    let mut source_view = None;
    unsafe { device.CreateShaderResourceView(&source, None, Some(&mut source_view))? };
    let mut output_view = None;
    unsafe { device.CreateRenderTargetView(back_buffer, None, Some(&mut output_view))? };
    Ok(FrameResources {
        width: description.Width,
        height: description.Height,
        source,
        source_view: source_view.ok_or_else(windows::core::Error::from_win32)?,
        output_view: output_view.ok_or_else(windows::core::Error::from_win32)?,
        manual_srgb: needs_manual_srgb(description.Format),
        output_mode: output_mode(description.Format),
    })
}

pub fn is_supported_format(format: DXGI_FORMAT) -> bool {
    matches!(
        format,
        DXGI_FORMAT_R8G8B8A8_UNORM
            | DXGI_FORMAT_R8G8B8A8_UNORM_SRGB
            | DXGI_FORMAT_B8G8R8A8_UNORM
            | DXGI_FORMAT_B8G8R8A8_UNORM_SRGB
            | DXGI_FORMAT_R10G10B10A2_UNORM
            | DXGI_FORMAT_R16G16B16A16_FLOAT
    )
}

fn output_mode(format: DXGI_FORMAT) -> OutputMode {
    match format {
        DXGI_FORMAT_R10G10B10A2_UNORM => OutputMode::Hdr10,
        DXGI_FORMAT_R16G16B16A16_FLOAT => OutputMode::ScRgb,
        _ => OutputMode::Sdr,
    }
}

fn needs_manual_srgb(format: DXGI_FORMAT) -> bool {
    matches!(
        format,
        DXGI_FORMAT_R8G8B8A8_UNORM | DXGI_FORMAT_B8G8R8A8_UNORM
    )
}

pub fn create_vertex_shader(device: &ID3D11Device) -> windows::core::Result<ID3D11VertexShader> {
    let code = shaders::compile(Source::Photorealism, b"VSMain\0", b"vs_5_0\0")?;
    let mut shader = None;
    unsafe { device.CreateVertexShader(&code, None, Some(&mut shader))? };
    shader.ok_or_else(windows::core::Error::from_win32)
}

pub fn create_pixel_shader(device: &ID3D11Device) -> windows::core::Result<ID3D11PixelShader> {
    let code = shaders::compile(Source::Photorealism, b"PSMain\0", b"ps_5_0\0")?;
    let mut shader = None;
    unsafe { device.CreatePixelShader(&code, None, Some(&mut shader))? };
    shader.ok_or_else(windows::core::Error::from_win32)
}

pub fn create_sampler(device: &ID3D11Device) -> windows::core::Result<ID3D11SamplerState> {
    let description = D3D11_SAMPLER_DESC {
        Filter: D3D11_FILTER_MIN_MAG_MIP_LINEAR,
        AddressU: D3D11_TEXTURE_ADDRESS_CLAMP,
        AddressV: D3D11_TEXTURE_ADDRESS_CLAMP,
        AddressW: D3D11_TEXTURE_ADDRESS_CLAMP,
        MaxLOD: f32::MAX,
        ..Default::default()
    };
    let mut sampler = None;
    unsafe { device.CreateSamplerState(&description, Some(&mut sampler))? };
    sampler.ok_or_else(windows::core::Error::from_win32)
}

pub fn create_constant_buffer(device: &ID3D11Device) -> windows::core::Result<ID3D11Buffer> {
    let description = D3D11_BUFFER_DESC {
        ByteWidth: size_of::<ShaderSettings>() as u32,
        Usage: D3D11_USAGE_DEFAULT,
        BindFlags: D3D11_BIND_CONSTANT_BUFFER.0 as u32,
        ..Default::default()
    };
    let mut buffer = None;
    unsafe { device.CreateBuffer(&description, None, Some(&mut buffer))? };
    buffer.ok_or_else(windows::core::Error::from_win32)
}
