mod resources;

use windows::Win32::Graphics::Direct3D::D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST;
use windows::Win32::Graphics::Direct3D11::*;

use super::resources::FrameResources;
use super::shaders::{self, Source};
use crate::menu::Vertex;
use crate::settings::Settings;
use resources::VertexBuffer;

const INITIAL_CAPACITY: usize = 4096;

#[repr(C)]
#[derive(Clone, Copy)]
struct OverlayConstants {
    output_mode: f32,
    hdr_paper_white_nits: f32,
    hdr_peak_nits: f32,
    output_needs_srgb_encode: f32,
}

pub struct Overlay {
    vertex_shader: ID3D11VertexShader,
    pixel_shader: ID3D11PixelShader,
    font_view: ID3D11ShaderResourceView,
    sampler: ID3D11SamplerState,
    blend: ID3D11BlendState,
    constants: ID3D11Buffer,
    vertices: VertexBuffer,
}

impl Overlay {
    pub fn new(device: &ID3D11Device) -> windows::core::Result<Self> {
        Ok(Self {
            vertex_shader: create_vertex_shader(device)?,
            pixel_shader: create_pixel_shader(device)?,
            font_view: resources::create_font_view(device)?,
            sampler: resources::create_sampler(device)?,
            blend: resources::create_blend_state(device)?,
            constants: resources::create_constant_buffer::<OverlayConstants>(device)?,
            vertices: resources::create_vertex_buffer(device, INITIAL_CAPACITY)?,
        })
    }

    pub unsafe fn draw(
        &mut self,
        device: &ID3D11Device,
        context: &ID3D11DeviceContext,
        frame: &FrameResources,
        settings: Settings,
        peak_nits: f32,
        vertices: &[Vertex],
    ) -> windows::core::Result<()> {
        if vertices.is_empty() {
            return Ok(());
        }
        self.reserve(device, vertices.len())?;
        unsafe { self.upload(context, vertices)? };
        unsafe { self.update_constants(context, frame, settings, peak_nits) };
        unsafe { self.bind(context, frame) };
        unsafe { context.Draw(vertices.len() as u32, 0) };
        unsafe { context.VSSetShaderResources(0, Some(&[None])) };
        Ok(())
    }

    fn reserve(&mut self, device: &ID3D11Device, needed: usize) -> windows::core::Result<()> {
        if needed <= self.vertices.capacity {
            return Ok(());
        }
        self.vertices = resources::create_vertex_buffer(device, needed.next_power_of_two())?;
        Ok(())
    }

    unsafe fn upload(
        &self,
        context: &ID3D11DeviceContext,
        vertices: &[Vertex],
    ) -> windows::core::Result<()> {
        let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
        unsafe {
            context.Map(
                &self.vertices.buffer,
                0,
                D3D11_MAP_WRITE_DISCARD,
                0,
                Some(&mut mapped),
            )?;
            std::ptr::copy_nonoverlapping(
                vertices.as_ptr(),
                mapped.pData.cast::<Vertex>(),
                vertices.len(),
            );
            context.Unmap(&self.vertices.buffer, 0);
        }
        Ok(())
    }

    unsafe fn update_constants(
        &self,
        context: &ID3D11DeviceContext,
        frame: &FrameResources,
        settings: Settings,
        peak_nits: f32,
    ) {
        let constants = OverlayConstants {
            output_mode: frame.output_mode.shader_value(),
            hdr_paper_white_nits: settings.hdr_paper_white_nits,
            hdr_peak_nits: peak_nits,
            output_needs_srgb_encode: if frame.manual_srgb { 1.0 } else { 0.0 },
        };
        unsafe {
            context.UpdateSubresource(
                &self.constants,
                0,
                None,
                (&constants as *const OverlayConstants).cast(),
                0,
                0,
            );
        }
    }

    unsafe fn bind(&self, context: &ID3D11DeviceContext, frame: &FrameResources) {
        let viewport = D3D11_VIEWPORT {
            Width: frame.width as f32,
            Height: frame.height as f32,
            MaxDepth: 1.0,
            ..Default::default()
        };
        unsafe {
            context.IASetInputLayout(None);
            context.IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
            context.VSSetShader(&self.vertex_shader, None);
            context.PSSetShader(&self.pixel_shader, None);
            context.GSSetShader(None, None);
            context.HSSetShader(None, None);
            context.DSSetShader(None, None);
            context.OMSetBlendState(&self.blend, None, u32::MAX);
            context.OMSetDepthStencilState(None, 0);
            context.RSSetState(None);
            context.OMSetRenderTargets(Some(&[Some(frame.output_view.clone())]), None);
            context.VSSetShaderResources(0, Some(&[Some(self.vertices.view.clone())]));
            context.PSSetShaderResources(0, Some(&[Some(self.font_view.clone())]));
            context.PSSetSamplers(0, Some(&[Some(self.sampler.clone())]));
            context.PSSetConstantBuffers(0, Some(&[Some(self.constants.clone())]));
            context.RSSetViewports(Some(&[viewport]));
        }
    }
}

fn create_vertex_shader(device: &ID3D11Device) -> windows::core::Result<ID3D11VertexShader> {
    let code = shaders::compile(Source::Overlay, b"VSOverlay\0", b"vs_5_0\0")?;
    let mut shader = None;
    unsafe { device.CreateVertexShader(&code, None, Some(&mut shader))? };
    shader.ok_or_else(windows::core::Error::from_win32)
}

fn create_pixel_shader(device: &ID3D11Device) -> windows::core::Result<ID3D11PixelShader> {
    let code = shaders::compile(Source::Overlay, b"PSOverlay\0", b"ps_5_0\0")?;
    let mut shader = None;
    unsafe { device.CreatePixelShader(&code, None, Some(&mut shader))? };
    shader.ok_or_else(windows::core::Error::from_win32)
}
