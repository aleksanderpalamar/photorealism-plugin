use windows::Win32::Graphics::Direct3D11::{D3D11_TEXTURE2D_DESC, ID3D11Device, ID3D11Texture2D};

use super::display::DisplayLuminance;
use super::resources::{FrameResources, create_frame_resources};
use crate::logging;

pub fn ensure(
    device: &ID3D11Device,
    frame: &mut Option<FrameResources>,
    back_buffer: &ID3D11Texture2D,
    description: &D3D11_TEXTURE2D_DESC,
) -> windows::core::Result<()> {
    let matches = frame.as_ref().is_some_and(|frame| {
        frame.width == description.Width && frame.height == description.Height
    });
    if matches {
        return Ok(());
    }
    let created = create_frame_resources(device, back_buffer, description)?;
    logging::write(&format!(
        "Backbuffer {}x{} formato={} modo={}.",
        description.Width,
        description.Height,
        description.Format.0,
        created.output_mode.name(),
    ));
    *frame = Some(created);
    Ok(())
}

pub fn report_display(display: Option<&DisplayLuminance>) {
    let Some(display) = display else {
        logging::write("Display nao informou luminancia; hdr_peak_nits=auto usa 1000 nits.");
        return;
    };
    logging::write(&format!(
        "Display informou pico={:.0} nits e tela cheia={:.0} nits.",
        display.peak_nits, display.full_frame_nits,
    ));
}
