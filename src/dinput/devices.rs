use std::ffi::c_void;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

use windows::Win32::Devices::HumanInterfaceDevice::{
    DI8DEVTYPE_KEYBOARD, DI8DEVTYPE_MOUSE, DIDEVCAPS, IDirectInputDevice8W,
};
use windows::core::Interface;

const SLOTS: usize = 8;
const TYPE_MASK: u32 = 0xFF;

static ADDRESSES: [AtomicUsize; SLOTS] = [const { AtomicUsize::new(0) }; SLOTS];
static KINDS: [AtomicU32; SLOTS] = [const { AtomicU32::new(0) }; SLOTS];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Other,
    Mouse,
    Keyboard,
}

impl Kind {
    fn code(self) -> u32 {
        match self {
            Self::Other => 0,
            Self::Mouse => 1,
            Self::Keyboard => 2,
        }
    }

    fn from_code(code: u32) -> Self {
        match code {
            1 => Self::Mouse,
            2 => Self::Keyboard,
            _ => Self::Other,
        }
    }

    pub fn is_gated(self) -> bool {
        !matches!(self, Self::Other)
    }
}

pub fn kind_of(device: *mut c_void) -> Kind {
    let address = device as usize;
    for slot in 0..SLOTS {
        let stored = ADDRESSES[slot].load(Ordering::Acquire);
        if stored == address {
            return Kind::from_code(KINDS[slot].load(Ordering::Acquire));
        }
        if stored != 0 {
            continue;
        }
        let kind = query(device);
        KINDS[slot].store(kind.code(), Ordering::Release);
        ADDRESSES[slot].store(address, Ordering::Release);
        return kind;
    }
    query(device)
}

fn query(device: *mut c_void) -> Kind {
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
