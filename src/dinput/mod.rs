mod devices;
mod gate;

use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

use windows::Win32::Devices::HumanInterfaceDevice::{
    DirectInput8Create, GUID_SysKeyboard, GUID_SysMouse, IDirectInput8W, IDirectInputDevice8W,
};
use windows::Win32::Foundation::HINSTANCE;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::core::{GUID, Interface};

const VERSION: u32 = 0x0800;

static CAPTURING: AtomicBool = AtomicBool::new(false);
static MOTION_X: AtomicI32 = AtomicI32::new(0);
static MOTION_Y: AtomicI32 = AtomicI32::new(0);
static BUTTON: AtomicBool = AtomicBool::new(false);

pub fn set_capturing(capturing: bool) {
    CAPTURING.store(capturing, Ordering::Release);
}

pub fn is_capturing() -> bool {
    CAPTURING.load(Ordering::Acquire)
}

pub fn button() -> bool {
    BUTTON.load(Ordering::Acquire)
}

pub fn take_motion() -> (f32, f32) {
    (
        MOTION_X.swap(0, Ordering::AcqRel) as f32,
        MOTION_Y.swap(0, Ordering::AcqRel) as f32,
    )
}

fn push_motion(horizontal: i32, vertical: i32) {
    MOTION_X.fetch_add(horizontal, Ordering::AcqRel);
    MOTION_Y.fetch_add(vertical, Ordering::AcqRel);
}

fn set_button(pressed: bool) {
    BUTTON.store(pressed, Ordering::Release);
}

pub fn install() -> windows::core::Result<()> {
    let interface = create_interface()?;
    gate::patch_device(&create_device(&interface, &GUID_SysMouse)?)?;
    gate::patch_device(&create_device(&interface, &GUID_SysKeyboard)?)?;
    Ok(())
}

fn create_interface() -> windows::core::Result<IDirectInput8W> {
    let module = unsafe { GetModuleHandleW(None)? };
    let mut raw = std::ptr::null_mut();
    unsafe {
        DirectInput8Create(
            HINSTANCE(module.0),
            VERSION,
            &IDirectInput8W::IID,
            &mut raw,
            None,
        )?
    };
    if raw.is_null() {
        return Err(windows::core::Error::from_win32());
    }
    Ok(unsafe { IDirectInput8W::from_raw(raw) })
}

fn create_device(
    interface: &IDirectInput8W,
    guid: &GUID,
) -> windows::core::Result<IDirectInputDevice8W> {
    let mut device = None;
    unsafe { interface.CreateDevice(guid, &mut device, None)? };
    device.ok_or_else(windows::core::Error::from_win32)
}
