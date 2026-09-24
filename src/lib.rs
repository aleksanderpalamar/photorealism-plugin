#[cfg(any(windows, test))]
mod color;
#[cfg(any(windows, test))]
mod config;
#[cfg(any(windows, test))]
mod menu;
#[cfg(any(windows, test))]
mod pipeline;
#[cfg(any(windows, test))]
mod settings;

#[cfg(windows)]
mod dinput;
#[cfg(windows)]
mod graphics;
#[cfg(windows)]
mod hooks;
#[cfg(windows)]
mod input;
#[cfg(windows)]
mod logging;
#[cfg(windows)]
mod probe;
#[cfg(windows)]
mod proxy;
#[cfg(windows)]
mod runtime;
#[cfg(windows)]
mod vtable;
