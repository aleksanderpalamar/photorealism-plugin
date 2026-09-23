use std::ffi::c_void;
use std::sync::atomic::{AtomicPtr, Ordering};

use windows::Win32::System::Memory::{
    PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS, VirtualProtect,
};

pub unsafe fn entry(object: *mut c_void, slot: usize) -> *mut *mut c_void {
    let table = unsafe { *(object as *mut *mut *mut c_void) };
    unsafe { table.add(slot) }
}

pub unsafe fn patch(
    entry: *mut *mut c_void,
    replacement: *mut c_void,
    original: &AtomicPtr<c_void>,
) -> windows::core::Result<()> {
    let mut previous = PAGE_PROTECTION_FLAGS::default();
    unsafe {
        VirtualProtect(
            entry.cast(),
            size_of::<*mut c_void>(),
            PAGE_EXECUTE_READWRITE,
            &mut previous,
        )?
    };
    let current = unsafe { *entry };
    if current == replacement {
        return unsafe { restore_protection(entry, previous) };
    }
    original.store(current, Ordering::Release);
    unsafe { entry.write(replacement) };
    unsafe { restore_protection(entry, previous) }
}

unsafe fn restore_protection(
    entry: *mut *mut c_void,
    previous: PAGE_PROTECTION_FLAGS,
) -> windows::core::Result<()> {
    let mut ignored = PAGE_PROTECTION_FLAGS::default();
    unsafe {
        VirtualProtect(
            entry.cast(),
            size_of::<*mut c_void>(),
            previous,
            &mut ignored,
        )?
    };
    Ok(())
}
