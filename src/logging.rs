use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use windows::Win32::Foundation::HMODULE;
use windows::Win32::System::LibraryLoader::GetModuleFileNameW;

static MODULE_DIRECTORY: OnceLock<PathBuf> = OnceLock::new();

pub fn initialize(module: HMODULE) {
    let mut buffer = [0_u16; 1024];
    let length = unsafe { GetModuleFileNameW(Some(module), &mut buffer) } as usize;
    if length == 0 {
        return;
    }
    let path = PathBuf::from(String::from_utf16_lossy(&buffer[..length]));
    let directory = path.parent().unwrap_or(Path::new(".")).to_path_buf();
    let _ = MODULE_DIRECTORY.set(directory);
}

pub fn plugin_path(name: &str) -> PathBuf {
    let base = MODULE_DIRECTORY
        .get()
        .cloned()
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("photorealism-plugin").join(name)
}

pub fn write(message: &str) {
    let path = plugin_path("photorealism-plugin.log");
    let Some(parent) = path.parent() else {
        return;
    };
    let _ = std::fs::create_dir_all(parent);
    let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) else {
        return;
    };
    let _ = writeln!(file, "{message}");
}
