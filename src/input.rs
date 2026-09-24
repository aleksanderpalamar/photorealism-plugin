use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_CONTROL};

const KEY_P: i32 = b'P' as i32;
const PRESSED: u16 = 0x8000;

#[derive(Default)]
pub struct Shortcut {
    held: bool,
}

impl Shortcut {
    pub fn triggered(&mut self) -> bool {
        let down = is_down(VK_CONTROL.0 as i32) && is_down(KEY_P);
        let edge = down && !self.held;
        self.held = down;
        edge
    }
}

fn is_down(key: i32) -> bool {
    unsafe { GetAsyncKeyState(key) as u16 & PRESSED != 0 }
}
