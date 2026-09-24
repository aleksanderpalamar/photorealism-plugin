use windows::Win32::Graphics::Direct3D::D3D11_SRV_DIMENSION_BUFFER;
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::Dxgi::Common::{
    DXGI_FORMAT_R8_UNORM, DXGI_FORMAT_UNKNOWN, DXGI_SAMPLE_DESC,
};

use crate::menu::{self, Vertex};

pub struct VertexBuffer {
    pub buffer: ID3D11Buffer,
    pub view: ID3D11ShaderResourceView,
    pub capacity: usize,
}

pub fn create_font_view(device: &ID3D11Device) -> windows::core::Result<ID3D11ShaderResourceView> {
    let pixels = menu::atlas();
    let description = D3D11_TEXTURE2D_DESC {
        Width: menu::ATLAS_WIDTH as u32,
        Height: menu::ATLAS_HEIGHT as u32,
        MipLevels: 1,
        ArraySize: 1,
        Format: DXGI_FORMAT_R8_UNORM,
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
        },
        Usage: D3D11_USAGE_DEFAULT,
        BindFlags: D3D11_BIND_SHADER_RESOURCE.0 as u32,
        ..Default::default()
    };
    let initial = D3D11_SUBRESOURCE_DATA {
        pSysMem: pixels.as_ptr().cast(),
        SysMemPitch: menu::ATLAS_WIDTH as u32,
        SysMemSlicePitch: 0,
    };
    let mut texture = None;
    unsafe { device.CreateTexture2D(&description, Some(&initial), Some(&mut texture))? };
    let texture = texture.ok_or_else(windows::core::Error::from_win32)?;
    let mut view = None;
    unsafe { device.CreateShaderResourceView(&texture, None, Some(&mut view))? };
    view.ok_or_else(windows::core::Error::from_win32)
}

pub fn create_sampler(device: &ID3D11Device) -> windows::core::Result<ID3D11SamplerState> {
    let description = D3D11_SAMPLER_DESC {
        Filter: D3D11_FILTER_MIN_MAG_MIP_POINT,
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

pub fn create_blend_state(device: &ID3D11Device) -> windows::core::Result<ID3D11BlendState> {
    let mut description = D3D11_BLEND_DESC::default();
    description.RenderTarget[0] = D3D11_RENDER_TARGET_BLEND_DESC {
        BlendEnable: true.into(),
        SrcBlend: D3D11_BLEND_SRC_ALPHA,
        DestBlend: D3D11_BLEND_INV_SRC_ALPHA,
        BlendOp: D3D11_BLEND_OP_ADD,
        SrcBlendAlpha: D3D11_BLEND_ONE,
        DestBlendAlpha: D3D11_BLEND_INV_SRC_ALPHA,
        BlendOpAlpha: D3D11_BLEND_OP_ADD,
        RenderTargetWriteMask: D3D11_COLOR_WRITE_ENABLE_ALL.0 as u8,
    };
    let mut state = None;
    unsafe { device.CreateBlendState(&description, Some(&mut state))? };
    state.ok_or_else(windows::core::Error::from_win32)
}

pub fn create_constant_buffer<T>(device: &ID3D11Device) -> windows::core::Result<ID3D11Buffer> {
    let description = D3D11_BUFFER_DESC {
        ByteWidth: size_of::<T>() as u32,
        Usage: D3D11_USAGE_DEFAULT,
        BindFlags: D3D11_BIND_CONSTANT_BUFFER.0 as u32,
        ..Default::default()
    };
    let mut buffer = None;
    unsafe { device.CreateBuffer(&description, None, Some(&mut buffer))? };
    buffer.ok_or_else(windows::core::Error::from_win32)
}

pub fn create_vertex_buffer(
    device: &ID3D11Device,
    capacity: usize,
) -> windows::core::Result<VertexBuffer> {
    let stride = size_of::<Vertex>() as u32;
    let description = D3D11_BUFFER_DESC {
        ByteWidth: stride * capacity as u32,
        Usage: D3D11_USAGE_DYNAMIC,
        BindFlags: D3D11_BIND_SHADER_RESOURCE.0 as u32,
        CPUAccessFlags: D3D11_CPU_ACCESS_WRITE.0 as u32,
        MiscFlags: D3D11_RESOURCE_MISC_BUFFER_STRUCTURED.0 as u32,
        StructureByteStride: stride,
    };
    let mut buffer = None;
    unsafe { device.CreateBuffer(&description, None, Some(&mut buffer))? };
    let buffer = buffer.ok_or_else(windows::core::Error::from_win32)?;

    let mut view_description = D3D11_SHADER_RESOURCE_VIEW_DESC {
        Format: DXGI_FORMAT_UNKNOWN,
        ViewDimension: D3D11_SRV_DIMENSION_BUFFER,
        ..Default::default()
    };
    view_description.Anonymous.Buffer.Anonymous1.FirstElement = 0;
    view_description.Anonymous.Buffer.Anonymous2.NumElements = capacity as u32;
    let mut view = None;
    unsafe { device.CreateShaderResourceView(&buffer, Some(&view_description), Some(&mut view))? };
    Ok(VertexBuffer {
        buffer,
        view: view.ok_or_else(windows::core::Error::from_win32)?,
        capacity,
    })
}
