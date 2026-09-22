use std::slice;

use windows::Win32::Graphics::Direct3D::Fxc::D3DCompile;
use windows::Win32::Graphics::Direct3D::ID3DBlob;
use windows::core::PCSTR;

const SOURCE: &[u8] = include_bytes!("../../shaders/photorealism.hlsl");

pub fn compile(entry: &'static [u8], target: &'static [u8]) -> windows::core::Result<Vec<u8>> {
    let mut code: Option<ID3DBlob> = None;
    unsafe {
        D3DCompile(
            SOURCE.as_ptr().cast(),
            SOURCE.len(),
            PCSTR::null(),
            None,
            None,
            PCSTR(entry.as_ptr()),
            PCSTR(target.as_ptr()),
            0,
            0,
            &mut code,
            None,
        )?;
    }
    let blob = code.ok_or_else(windows::core::Error::from_win32)?;
    let bytes =
        unsafe { slice::from_raw_parts(blob.GetBufferPointer().cast(), blob.GetBufferSize()) };
    Ok(bytes.to_vec())
}
