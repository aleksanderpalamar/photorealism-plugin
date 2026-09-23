mod constants;
mod display;
mod overlay;
mod resources;
mod shaders;
mod state;

use windows::Win32::Graphics::Direct3D::D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST;
use windows::Win32::Graphics::Direct3D11::{
    D3D11_TEXTURE2D_DESC, D3D11_VIEWPORT, ID3D11Buffer, ID3D11Device, ID3D11DeviceContext,
    ID3D11PixelShader, ID3D11SamplerState, ID3D11Texture2D, ID3D11VertexShader,
};
use windows::Win32::Graphics::Dxgi::IDXGISwapChain;

use crate::logging;
use crate::menu::{Session, Viewport};
use crate::settings::Settings;
use constants::ShaderSettings;
use display::DisplayLuminance;
use overlay::Overlay;
use resources::{
    FrameResources, create_constant_buffer, create_frame_resources, create_pixel_shader,
    create_sampler, create_vertex_shader, is_supported_format, texture_description,
};
use state::PipelineState;

const TITLE: &str = concat!("photorealism-plugin ", env!("CARGO_PKG_VERSION"));

pub struct Renderer {
    swap_chain: usize,
    device: ID3D11Device,
    context: ID3D11DeviceContext,
    vertex_shader: ID3D11VertexShader,
    pixel_shader: ID3D11PixelShader,
    sampler: ID3D11SamplerState,
    constants: ID3D11Buffer,
    overlay: Overlay,
    display: Option<DisplayLuminance>,
    frame: Option<FrameResources>,
}

impl Renderer {
    pub unsafe fn new(swap_chain: &IDXGISwapChain) -> windows::core::Result<Self> {
        let device: ID3D11Device = unsafe { swap_chain.GetDevice()? };
        let context = unsafe { device.GetImmediateContext()? };
        let vertex_shader = create_vertex_shader(&device)?;
        let pixel_shader = create_pixel_shader(&device)?;
        let sampler = create_sampler(&device)?;
        let constants = create_constant_buffer(&device)?;
        let overlay = Overlay::new(&device)?;
        let display = display::query(swap_chain);
        report_display(display.as_ref());
        Ok(Self {
            swap_chain: windows::core::Interface::as_raw(swap_chain) as usize,
            device,
            context,
            vertex_shader,
            pixel_shader,
            sampler,
            constants,
            overlay,
            display,
            frame: None,
        })
    }

    pub unsafe fn matches(&self, swap_chain: &IDXGISwapChain) -> bool {
        self.swap_chain == windows::core::Interface::as_raw(swap_chain) as usize
    }

    pub unsafe fn render(
        &mut self,
        swap_chain: &IDXGISwapChain,
        settings: Settings,
        session: &Session,
    ) -> windows::core::Result<()> {
        let back_buffer: ID3D11Texture2D = unsafe { swap_chain.GetBuffer(0)? };
        let description = texture_description(&back_buffer);
        if description.SampleDesc.Count != 1
            || description.Width < 640
            || description.Height < 480
            || !is_supported_format(description.Format)
        {
            return Ok(());
        }
        self.ensure_frame(&back_buffer, &description)?;
        let state = unsafe { PipelineState::capture(&self.context) };
        if settings.enabled {
            unsafe { self.draw(&back_buffer, settings) };
        }
        let menu = unsafe { self.draw_menu(settings, session) };
        unsafe { state.restore(&self.context) };
        menu
    }

    pub fn viewport(&self) -> Option<Viewport> {
        self.frame.as_ref().map(|frame| Viewport {
            width: frame.width as f32,
            height: frame.height as f32,
        })
    }

    unsafe fn draw_menu(
        &mut self,
        settings: Settings,
        session: &Session,
    ) -> windows::core::Result<()> {
        if !session.is_visible() {
            return Ok(());
        }
        let Some(viewport) = self.viewport() else {
            return Ok(());
        };
        let vertices = session.vertices(&settings, viewport, TITLE);
        let Self {
            device,
            context,
            overlay,
            display,
            frame,
            ..
        } = self;
        let Some(frame) = frame.as_ref() else {
            return Ok(());
        };
        let peak = settings
            .hdr_peak_nits
            .resolve(display.map(|display| display.peak_nits));
        unsafe { overlay.draw(device, context, frame, settings, peak, &vertices) }
    }

    fn ensure_frame(
        &mut self,
        back_buffer: &ID3D11Texture2D,
        description: &D3D11_TEXTURE2D_DESC,
    ) -> windows::core::Result<()> {
        let matches = self.frame.as_ref().is_some_and(|frame| {
            frame.width == description.Width && frame.height == description.Height
        });
        if matches {
            return Ok(());
        }
        let frame = create_frame_resources(&self.device, back_buffer, description)?;
        logging::write(&format!(
            "Backbuffer {}x{} formato={} modo={}.",
            description.Width,
            description.Height,
            description.Format.0,
            frame.output_mode.name(),
        ));
        self.frame = Some(frame);
        Ok(())
    }

    unsafe fn draw(&self, back_buffer: &ID3D11Texture2D, settings: Settings) {
        let Some(frame) = self.frame.as_ref() else {
            return;
        };
        let reported_peak = self.display.map(|display| display.peak_nits);
        let constants = ShaderSettings::new(settings, frame, reported_peak);
        unsafe {
            self.context.CopyResource(&frame.source, back_buffer);
            self.context.UpdateSubresource(
                &self.constants,
                0,
                None,
                (&constants as *const ShaderSettings).cast(),
                0,
                0,
            );
            self.bind(frame);
            self.context.Draw(3, 0);
            self.context.PSSetShaderResources(0, Some(&[None]));
        }
    }

    unsafe fn bind(&self, frame: &FrameResources) {
        let viewport = D3D11_VIEWPORT {
            Width: frame.width as f32,
            Height: frame.height as f32,
            MaxDepth: 1.0,
            ..Default::default()
        };
        unsafe {
            self.context.IASetInputLayout(None);
            self.context
                .IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
            self.context.VSSetShader(&self.vertex_shader, None);
            self.context.PSSetShader(&self.pixel_shader, None);
            self.context.GSSetShader(None, None);
            self.context.HSSetShader(None, None);
            self.context.DSSetShader(None, None);
            self.context.OMSetBlendState(None, None, u32::MAX);
            self.context.OMSetDepthStencilState(None, 0);
            self.context.RSSetState(None);
            self.context
                .OMSetRenderTargets(Some(&[Some(frame.output_view.clone())]), None);
            self.context
                .PSSetShaderResources(0, Some(&[Some(frame.source_view.clone())]));
            self.context
                .PSSetSamplers(0, Some(&[Some(self.sampler.clone())]));
            self.context
                .PSSetConstantBuffers(0, Some(&[Some(self.constants.clone())]));
            self.context.RSSetViewports(Some(&[viewport]));
        }
    }
}

fn report_display(display: Option<&DisplayLuminance>) {
    let Some(display) = display else {
        logging::write("Display nao informou luminancia; hdr_peak_nits=auto usa 1000 nits.");
        return;
    };
    logging::write(&format!(
        "Display informou pico={:.0} nits e tela cheia={:.0} nits.",
        display.peak_nits, display.full_frame_nits,
    ));
}
