use std::cell::Cell;
use std::ffi::c_void;
use std::sync::Once;
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};
use std::time::Duration;

use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT;
use windows::Win32::Graphics::Dxgi::{
    DXGI_PRESENT, DXGI_PRESENT_PARAMETERS, IDXGISwapChain, IDXGISwapChain1,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::core::{HRESULT, Interface, PCWSTR};

use crate::vtable::{entry, patch};
use crate::{logging, probe::Probe, runtime};

type PresentFn = unsafe extern "system" fn(*mut c_void, u32, DXGI_PRESENT) -> HRESULT;
type ResizeFn = unsafe extern "system" fn(*mut c_void, u32, u32, u32, DXGI_FORMAT, u32) -> HRESULT;
type Present1Fn = unsafe extern "system" fn(
    *mut c_void,
    u32,
    DXGI_PRESENT,
    *const DXGI_PRESENT_PARAMETERS,
) -> HRESULT;

static START: Once = Once::new();
static ORIGINAL_PRESENT: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static ORIGINAL_RESIZE: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static ORIGINAL_PRESENT1: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static GATE_INSTALLED: AtomicBool = AtomicBool::new(false);
static GATE_REPORTED: AtomicBool = AtomicBool::new(false);

thread_local! {
    static PROCESSING: Cell<bool> = const { Cell::new(false) };
}

pub fn start() {
    START.call_once(|| {
        std::thread::spawn(install_when_ready);
    });
}

fn install_when_ready() {
    wait_for_steam_overlay();
    for _ in 0..120 {
        if install().is_ok() {
            logging::write("Hooks Present, Present1 e ResizeBuffers instalados.");
            install_input_gate();
            return;
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    logging::write("Falha ao instalar hooks D3D11 em 60 segundos.");
}

fn install_input_gate() {
    if GATE_INSTALLED.load(Ordering::Acquire) {
        return;
    }
    match crate::dinput::install() {
        Ok(()) => {
            GATE_INSTALLED.store(true, Ordering::Release);
            logging::write("Gate de entrada do DirectInput instalado.");
        }
        Err(error) => report_gate_failure(error),
    }
}

fn report_gate_failure(error: windows::core::Error) {
    if GATE_REPORTED.swap(true, Ordering::AcqRel) {
        return;
    }
    logging::write(&format!(
        "Falha ao instalar o gate de entrada do DirectInput: {:#x}.",
        error.code().0,
    ));
}

fn wait_for_steam_overlay() {
    let name: Vec<u16> = "gameoverlayrenderer64.dll\0".encode_utf16().collect();
    let mut stable_samples = 0;
    for _ in 0..30 {
        let loaded = unsafe { GetModuleHandleW(PCWSTR(name.as_ptr())) }.is_ok();
        stable_samples = if loaded { stable_samples + 1 } else { 0 };
        if stable_samples >= 4 {
            return;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn install() -> windows::core::Result<()> {
    let probe = Probe::create()?;
    let raw = probe.swap_chain.as_raw();
    unsafe {
        replace(
            entry(raw, 8),
            hooked_present as *mut c_void,
            &ORIGINAL_PRESENT,
        )?;
        replace(
            entry(raw, 13),
            hooked_resize as *mut c_void,
            &ORIGINAL_RESIZE,
        )?;
    }
    install_input_gate();
    patch_present1(&probe.swap_chain)
}

unsafe fn replace(
    slot: *mut *mut c_void,
    replacement: *mut c_void,
    original: &AtomicPtr<c_void>,
) -> windows::core::Result<()> {
    let Some(previous) = (unsafe { patch(slot, replacement)? }) else {
        return Ok(());
    };
    original.store(previous, Ordering::Release);
    Ok(())
}

fn patch_present1(swap_chain: &IDXGISwapChain) -> windows::core::Result<()> {
    let extended: IDXGISwapChain1 = swap_chain.cast()?;
    let raw = extended.as_raw();
    unsafe {
        replace(
            entry(raw, 22),
            hooked_present1 as *mut c_void,
            &ORIGINAL_PRESENT1,
        )
    }
}

unsafe extern "system" fn hooked_present(
    this: *mut c_void,
    sync_interval: u32,
    flags: DXGI_PRESENT,
) -> HRESULT {
    let owns_dispatch = enter_processing();
    if owns_dispatch {
        process_borrowed_swap_chain(this);
    }
    let original: PresentFn =
        unsafe { std::mem::transmute(ORIGINAL_PRESENT.load(Ordering::Acquire)) };
    let result = unsafe { original(this, sync_interval, flags) };
    leave_processing(owns_dispatch);
    result
}

unsafe extern "system" fn hooked_present1(
    this: *mut c_void,
    sync_interval: u32,
    flags: DXGI_PRESENT,
    parameters: *const DXGI_PRESENT_PARAMETERS,
) -> HRESULT {
    let owns_dispatch = enter_processing();
    if owns_dispatch {
        process_borrowed_swap_chain(this);
    }
    let original: Present1Fn =
        unsafe { std::mem::transmute(ORIGINAL_PRESENT1.load(Ordering::Acquire)) };
    let result = unsafe { original(this, sync_interval, flags, parameters) };
    leave_processing(owns_dispatch);
    result
}

fn process_borrowed_swap_chain(raw: *mut c_void) {
    if let Some(swap_chain) = unsafe { IDXGISwapChain::from_raw_borrowed(&raw) } {
        runtime::process(swap_chain);
    }
}

fn enter_processing() -> bool {
    PROCESSING.with(|processing| !processing.replace(true))
}

fn leave_processing(owns_dispatch: bool) {
    if !owns_dispatch {
        return;
    }
    PROCESSING.with(|processing| processing.set(false));
}

unsafe extern "system" fn hooked_resize(
    this: *mut c_void,
    count: u32,
    width: u32,
    height: u32,
    format: DXGI_FORMAT,
    flags: u32,
) -> HRESULT {
    runtime::reset();
    let original: ResizeFn =
        unsafe { std::mem::transmute(ORIGINAL_RESIZE.load(Ordering::Acquire)) };
    unsafe { original(this, count, width, height, format, flags) }
}
