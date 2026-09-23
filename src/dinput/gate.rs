use std::ffi::c_void;
use std::ptr::null_mut;
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};

use windows::Win32::Devices::HumanInterfaceDevice::{DIDEVICEOBJECTDATA, IDirectInputDevice8W};
use windows::core::{HRESULT, Interface};

use super::devices::{self, Kind};
use super::{is_capturing, push_motion, set_button};
use crate::logging;
use crate::vtable::{entry, patch};

const GET_DEVICE_STATE_SLOT: usize = 9;
const GET_DEVICE_DATA_SLOT: usize = 10;
const AXIS_X_OFFSET: u32 = 0;
const AXIS_Y_OFFSET: u32 = 4;
const BUTTON_OFFSET: u32 = 12;
const PRESSED_BIT: u8 = 0x80;

type GetDeviceStateFn = unsafe extern "system" fn(*mut c_void, u32, *mut c_void) -> HRESULT;
type GetDeviceDataFn =
    unsafe extern "system" fn(*mut c_void, u32, *mut DIDEVICEOBJECTDATA, *mut u32, u32) -> HRESULT;

static ORIGINAL_STATE: AtomicPtr<c_void> = AtomicPtr::new(null_mut());
static ORIGINAL_DATA: AtomicPtr<c_void> = AtomicPtr::new(null_mut());
static REPORTED: AtomicBool = AtomicBool::new(false);

pub fn patch_device(device: &IDirectInputDevice8W) -> windows::core::Result<()> {
    let raw = device.as_raw();
    unsafe {
        patch(
            entry(raw, GET_DEVICE_STATE_SLOT),
            hooked_state as *mut c_void,
            &ORIGINAL_STATE,
        )?;
        patch(
            entry(raw, GET_DEVICE_DATA_SLOT),
            hooked_data as *mut c_void,
            &ORIGINAL_DATA,
        )?;
    }
    Ok(())
}

unsafe extern "system" fn hooked_state(
    device: *mut c_void,
    size: u32,
    data: *mut c_void,
) -> HRESULT {
    let original: GetDeviceStateFn =
        unsafe { std::mem::transmute(ORIGINAL_STATE.load(Ordering::Acquire)) };
    let result = unsafe { original(device, size, data) };
    if result.is_err() || data.is_null() || !is_capturing() {
        return result;
    }
    let kind = devices::kind_of(device);
    if !kind.is_gated() {
        return result;
    }
    if matches!(kind, Kind::Mouse) {
        unsafe { absorb_state(data) };
    }
    report("estado");
    unsafe { std::ptr::write_bytes(data.cast::<u8>(), 0, size as usize) };
    result
}

unsafe extern "system" fn hooked_data(
    device: *mut c_void,
    size: u32,
    data: *mut DIDEVICEOBJECTDATA,
    count: *mut u32,
    flags: u32,
) -> HRESULT {
    let original: GetDeviceDataFn =
        unsafe { std::mem::transmute(ORIGINAL_DATA.load(Ordering::Acquire)) };
    let result = unsafe { original(device, size, data, count, flags) };
    if result.is_err() || count.is_null() || !is_capturing() {
        return result;
    }
    let kind = devices::kind_of(device);
    if !kind.is_gated() {
        return result;
    }
    let entries = unsafe { *count };
    if matches!(kind, Kind::Mouse) && !data.is_null() {
        unsafe { absorb_data(data, entries) };
    }
    report("buffer");
    unsafe { *count = 0 };
    result
}

unsafe fn absorb_state(data: *mut c_void) {
    let bytes = data.cast::<u8>();
    let horizontal = unsafe { read_axis(bytes, 0) };
    let vertical = unsafe { read_axis(bytes, 4) };
    let pressed = unsafe { *bytes.add(BUTTON_OFFSET as usize) } & PRESSED_BIT != 0;
    push_motion(horizontal, vertical);
    set_button(pressed);
}

unsafe fn absorb_data(entries: *const DIDEVICEOBJECTDATA, count: u32) {
    for index in 0..count as usize {
        let item = unsafe { &*entries.add(index) };
        match item.dwOfs {
            AXIS_X_OFFSET => push_motion(item.dwData as i32, 0),
            AXIS_Y_OFFSET => push_motion(0, item.dwData as i32),
            BUTTON_OFFSET => set_button(item.dwData as u8 & PRESSED_BIT != 0),
            _ => {}
        }
    }
}

unsafe fn read_axis(bytes: *const u8, offset: usize) -> i32 {
    let mut value = [0_u8; 4];
    unsafe { std::ptr::copy_nonoverlapping(bytes.add(offset), value.as_mut_ptr(), 4) };
    i32::from_ne_bytes(value)
}

fn report(mode: &str) {
    if REPORTED.swap(true, Ordering::AcqRel) {
        return;
    }
    logging::write(&format!(
        "Entrada do jogo bloqueada pelo menu; o DirectInput usa {mode}."
    ));
}
