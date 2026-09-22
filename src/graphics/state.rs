use windows::Win32::Graphics::Direct3D::D3D_PRIMITIVE_TOPOLOGY;
use windows::Win32::Graphics::Direct3D11::*;

pub struct PipelineState {
    input_layout: Option<ID3D11InputLayout>,
    topology: D3D_PRIMITIVE_TOPOLOGY,
    vertex_shader: Option<ID3D11VertexShader>,
    pixel_shader: Option<ID3D11PixelShader>,
    geometry_shader: Option<ID3D11GeometryShader>,
    hull_shader: Option<ID3D11HullShader>,
    domain_shader: Option<ID3D11DomainShader>,
    blend_state: Option<ID3D11BlendState>,
    blend_factor: [f32; 4],
    sample_mask: u32,
    depth_state: Option<ID3D11DepthStencilState>,
    stencil_reference: u32,
    rasterizer_state: Option<ID3D11RasterizerState>,
    render_target: [Option<ID3D11RenderTargetView>; 1],
    depth_target: Option<ID3D11DepthStencilView>,
    shader_resource: [Option<ID3D11ShaderResourceView>; 1],
    sampler: [Option<ID3D11SamplerState>; 1],
    constant_buffer: [Option<ID3D11Buffer>; 1],
    viewports: Vec<D3D11_VIEWPORT>,
}

impl PipelineState {
    pub unsafe fn capture(context: &ID3D11DeviceContext) -> Self {
        let mut state = Self {
            input_layout: unsafe { context.IAGetInputLayout().ok() },
            topology: unsafe { context.IAGetPrimitiveTopology() },
            vertex_shader: None,
            pixel_shader: None,
            geometry_shader: None,
            hull_shader: None,
            domain_shader: None,
            blend_state: None,
            blend_factor: [0.0; 4],
            sample_mask: 0,
            depth_state: None,
            stencil_reference: 0,
            rasterizer_state: None,
            render_target: [None],
            depth_target: None,
            shader_resource: [None],
            sampler: [None],
            constant_buffer: [None],
            viewports: Vec::new(),
        };
        unsafe { state.capture_shaders(context) };
        unsafe { state.capture_bindings(context) };
        unsafe { state.capture_viewports(context) };
        state
    }

    unsafe fn capture_shaders(&mut self, context: &ID3D11DeviceContext) {
        unsafe {
            context.VSGetShader(&mut self.vertex_shader, None, None);
            context.PSGetShader(&mut self.pixel_shader, None, None);
            context.GSGetShader(&mut self.geometry_shader, None, None);
            context.HSGetShader(&mut self.hull_shader, None, None);
            context.DSGetShader(&mut self.domain_shader, None, None);
        }
    }

    unsafe fn capture_bindings(&mut self, context: &ID3D11DeviceContext) {
        unsafe {
            context.OMGetRenderTargets(Some(&mut self.render_target), Some(&mut self.depth_target));
            context.OMGetBlendState(
                Some(&mut self.blend_state),
                Some(&mut self.blend_factor),
                Some(&mut self.sample_mask),
            );
            context.OMGetDepthStencilState(
                Some(&mut self.depth_state),
                Some(&mut self.stencil_reference),
            );
            self.rasterizer_state = context.RSGetState().ok();
            context.PSGetShaderResources(0, Some(&mut self.shader_resource));
            context.PSGetSamplers(0, Some(&mut self.sampler));
            context.PSGetConstantBuffers(0, Some(&mut self.constant_buffer));
        }
    }

    unsafe fn capture_viewports(&mut self, context: &ID3D11DeviceContext) {
        let mut count = 0;
        unsafe { context.RSGetViewports(&mut count, None) };
        if count == 0 {
            return;
        }
        self.viewports
            .resize(count as usize, D3D11_VIEWPORT::default());
        unsafe { context.RSGetViewports(&mut count, Some(self.viewports.as_mut_ptr())) };
    }

    pub unsafe fn restore(self, context: &ID3D11DeviceContext) {
        unsafe {
            context.IASetInputLayout(self.input_layout.as_ref());
            context.IASetPrimitiveTopology(self.topology);
            context.VSSetShader(self.vertex_shader.as_ref(), None);
            context.PSSetShader(self.pixel_shader.as_ref(), None);
            context.GSSetShader(self.geometry_shader.as_ref(), None);
            context.HSSetShader(self.hull_shader.as_ref(), None);
            context.DSSetShader(self.domain_shader.as_ref(), None);
            context.OMSetRenderTargets(Some(&self.render_target), self.depth_target.as_ref());
            context.OMSetBlendState(
                self.blend_state.as_ref(),
                Some(&self.blend_factor),
                self.sample_mask,
            );
            context.OMSetDepthStencilState(self.depth_state.as_ref(), self.stencil_reference);
            context.RSSetState(self.rasterizer_state.as_ref());
            context.PSSetShaderResources(0, Some(&self.shader_resource));
            context.PSSetSamplers(0, Some(&self.sampler));
            context.PSSetConstantBuffers(0, Some(&self.constant_buffer));
            context.RSSetViewports(Some(&self.viewports));
        }
    }
}
