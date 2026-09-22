use windows::Win32::Graphics::Dxgi::{IDXGIOutput6, IDXGISwapChain};
use windows::core::Interface;

#[derive(Clone, Copy)]
pub struct DisplayLuminance {
    pub peak_nits: f32,
    pub full_frame_nits: f32,
}

pub fn query(swap_chain: &IDXGISwapChain) -> Option<DisplayLuminance> {
    let output = unsafe { swap_chain.GetContainingOutput() }.ok()?;
    let output: IDXGIOutput6 = output.cast().ok()?;
    let description = unsafe { output.GetDesc1() }.ok()?;
    Some(DisplayLuminance {
        peak_nits: description.MaxLuminance,
        full_frame_nits: description.MaxFullFrameLuminance,
    })
}
