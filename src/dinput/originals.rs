use std::ffi::c_void;
use std::ptr::null_mut;
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};

const SLOTS: usize = 4;

pub struct Originals {
    entries: [AtomicUsize; SLOTS],
    functions: [AtomicPtr<c_void>; SLOTS],
}

impl Originals {
    pub const fn new() -> Self {
        Self {
            entries: [const { AtomicUsize::new(0) }; SLOTS],
            functions: [const { AtomicPtr::new(null_mut()) }; SLOTS],
        }
    }

    pub fn remember(&self, entry: *mut *mut c_void, original: *mut c_void) {
        let key = entry as usize;
        for slot in 0..SLOTS {
            let stored = self.entries[slot].load(Ordering::Acquire);
            if stored == key {
                return;
            }
            if stored != 0 {
                continue;
            }
            self.functions[slot].store(original, Ordering::Release);
            self.entries[slot].store(key, Ordering::Release);
            return;
        }
    }

    pub fn lookup(&self, entry: *mut *mut c_void) -> *mut c_void {
        let key = entry as usize;
        for slot in 0..SLOTS {
            if self.entries[slot].load(Ordering::Acquire) == key {
                return self.functions[slot].load(Ordering::Acquire);
            }
        }
        null_mut()
    }
}
