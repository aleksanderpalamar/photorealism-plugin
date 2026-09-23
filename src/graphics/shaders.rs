use std::slice;

use windows::Win32::Graphics::Direct3D::Fxc::D3DCompile;
use windows::Win32::Graphics::Direct3D::ID3DBlob;
use windows::core::PCSTR;

use crate::logging;

const COLOR_SPACE: &[u8] = include_bytes!("../../shaders/color_space.hlsl");
const PHOTOREALISM: &[u8] = include_bytes!("../../shaders/photorealism.hlsl");
const OVERLAY: &[u8] = include_bytes!("../../shaders/overlay.hlsl");

#[derive(Clone, Copy)]
pub enum Source {
    Photorealism,
    Overlay,
}

impl Source {
    fn body(self) -> &'static [u8] {
        match self {
            Self::Photorealism => PHOTOREALISM,
            Self::Overlay => OVERLAY,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Photorealism => "photorealism",
            Self::Overlay => "overlay",
        }
    }
}

pub fn compile(
    source: Source,
    entry: &'static [u8],
    target: &'static [u8],
) -> windows::core::Result<Vec<u8>> {
    let text = assemble(source);
    let mut code: Option<ID3DBlob> = None;
    let mut errors: Option<ID3DBlob> = None;
    let result = unsafe {
        D3DCompile(
            text.as_ptr().cast(),
            text.len(),
            PCSTR::null(),
            None,
            None,
            PCSTR(entry.as_ptr()),
            PCSTR(target.as_ptr()),
            0,
            0,
            &mut code,
            Some(&mut errors),
        )
    };
    if let Err(error) = result {
        report(source, errors.as_ref());
        return Err(error);
    }
    let blob = code.ok_or_else(windows::core::Error::from_win32)?;
    let bytes =
        unsafe { slice::from_raw_parts(blob.GetBufferPointer().cast(), blob.GetBufferSize()) };
    Ok(bytes.to_vec())
}

fn assemble(source: Source) -> Vec<u8> {
    let body = source.body();
    let mut text = Vec::with_capacity(COLOR_SPACE.len() + body.len());
    text.extend_from_slice(COLOR_SPACE);
    text.extend_from_slice(body);
    text
}

fn report(source: Source, errors: Option<&ID3DBlob>) {
    let Some(blob) = errors else {
        logging::write(&format!("Falha ao compilar o shader {}.", source.name()));
        return;
    };
    let bytes = unsafe {
        slice::from_raw_parts(blob.GetBufferPointer().cast::<u8>(), blob.GetBufferSize())
    };
    logging::write(&format!(
        "Falha ao compilar o shader {}: {}",
        source.name(),
        String::from_utf8_lossy(bytes).trim(),
    ));
}
