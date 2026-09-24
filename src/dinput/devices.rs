use std::ffi::c_void;

use windows::Win32::Devices::HumanInterfaceDevice::{
    DI8DEVTYPE_KEYBOARD, DI8DEVTYPE_MOUSE, DIDEVCAPS, IDirectInputDevice8W,
};
use windows::core::Interface;

const TYPE_MASK: u32 = 0xFF;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Other,
    Mouse,
    Keyboard,
}

impl Kind {
    pub fn is_gated(self) -> bool {
        !matches!(self, Self::Other)
    }
}

pub fn kind_of(device: *mut c_void) -> Kind {
    let Some(interface) = (unsafe { IDirectInputDevice8W::from_raw_borrowed(&device) }) else {
        return Kind::Other;
    };
    let mut capabilities = DIDEVCAPS {
        dwSize: size_of::<DIDEVCAPS>() as u32,
        ..Default::default()
    };
    if unsafe { interface.GetCapabilities(&mut capabilities) }.is_err() {
        return Kind::Other;
    }
    match capabilities.dwDevType & TYPE_MASK {
        DI8DEVTYPE_MOUSE => Kind::Mouse,
        DI8DEVTYPE_KEYBOARD => Kind::Keyboard,
        _ => Kind::Other,
    }
}
