use std::ffi::c_void;
use std::sync::OnceLock;

use windows::Win32::Foundation::{HINSTANCE, HMODULE, TRUE};
use windows::Win32::System::Environment::SetEnvironmentVariableW;
use windows::Win32::System::LibraryLoader::{
    DisableThreadLibraryCalls, GetProcAddress, LoadLibraryW,
};
use windows::Win32::System::SystemInformation::GetSystemDirectoryW;
use windows::Win32::System::SystemServices::DLL_PROCESS_ATTACH;
use windows::core::{BOOL, GUID, HRESULT, PCSTR, PCWSTR};

use crate::config::{CONFIG_FILE_NAME, ConfigSource, FileConfigSource};
use crate::settings::Settings;
use crate::{hooks, logging};

type CreateFactory = unsafe extern "system" fn(*const GUID, *mut *mut c_void) -> HRESULT;
type CreateFactory2 = unsafe extern "system" fn(u32, *const GUID, *mut *mut c_void) -> HRESULT;

static REAL_DXGI: OnceLock<usize> = OnceLock::new();
static HDR_ENVIRONMENT: OnceLock<()> = OnceLock::new();

fn prepare_dxgi() {
    HDR_ENVIRONMENT.get_or_init(|| {
        let source = FileConfigSource::new(logging::plugin_path(CONFIG_FILE_NAME));
        let settings = source
            .read()
            .map_or_else(Settings::default, |contents| Settings::parse(&contents));
        if !settings.force_hdr {
            logging::write("Exposicao HDR do DXVK desativada pela configuracao.");
            return;
        }
        let name: Vec<u16> = "DXVK_HDR\0".encode_utf16().collect();
        let value: Vec<u16> = "1\0".encode_utf16().collect();
        let result =
            unsafe { SetEnvironmentVariableW(PCWSTR(name.as_ptr()), PCWSTR(value.as_ptr())) };
        if result.is_err() {
            logging::write("Falha ao solicitar a exposicao HDR ao DXVK.");
            return;
        }
        logging::write("DXVK_HDR=1 definido antes da inicializacao do DXGI.");
    });
}

fn real_dxgi() -> Option<HMODULE> {
    let raw = *REAL_DXGI.get_or_init(load_real_dxgi);
    if raw == 0 {
        return None;
    }
    Some(HMODULE(raw as *mut c_void))
}

fn load_real_dxgi() -> usize {
    let mut buffer = [0_u16; 512];
    let length = unsafe { GetSystemDirectoryW(Some(&mut buffer)) } as usize;
    if length == 0 || length + 10 >= buffer.len() {
        return 0;
    }
    let suffix: Vec<u16> = "\\dxgi.dll\0".encode_utf16().collect();
    buffer[length..length + suffix.len()].copy_from_slice(&suffix);
    let Ok(module) = (unsafe { LoadLibraryW(PCWSTR(buffer.as_ptr())) }) else {
        return 0;
    };
    module.0 as usize
}

unsafe fn resolve<T>(name: &'static [u8]) -> Option<T> {
    let module = real_dxgi()?;
    let address = unsafe { GetProcAddress(module, PCSTR(name.as_ptr())) }?;
    Some(unsafe { std::mem::transmute_copy(&address) })
}

unsafe fn forward_factory(
    name: &'static [u8],
    interface_id: *const GUID,
    output: *mut *mut c_void,
) -> HRESULT {
    prepare_dxgi();
    hooks::start();
    let Some(function) = (unsafe { resolve::<CreateFactory>(name) }) else {
        return HRESULT(0x8000_4005_u32 as i32);
    };
    unsafe { function(interface_id, output) }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn CreateDXGIFactory(
    interface_id: *const GUID,
    output: *mut *mut c_void,
) -> HRESULT {
    unsafe { forward_factory(b"CreateDXGIFactory\0", interface_id, output) }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn CreateDXGIFactory1(
    interface_id: *const GUID,
    output: *mut *mut c_void,
) -> HRESULT {
    unsafe { forward_factory(b"CreateDXGIFactory1\0", interface_id, output) }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn CreateDXGIFactory2(
    flags: u32,
    interface_id: *const GUID,
    output: *mut *mut c_void,
) -> HRESULT {
    prepare_dxgi();
    hooks::start();
    let Some(function) = (unsafe { resolve::<CreateFactory2>(b"CreateDXGIFactory2\0") }) else {
        return HRESULT(0x8000_4005_u32 as i32);
    };
    unsafe { function(flags, interface_id, output) }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn DllMain(instance: HINSTANCE, reason: u32, _: *mut c_void) -> BOOL {
    if reason != DLL_PROCESS_ATTACH {
        return TRUE;
    }
    logging::initialize(HMODULE(instance.0));
    let _ = unsafe { DisableThreadLibraryCalls(HMODULE(instance.0)) };
    TRUE
}
