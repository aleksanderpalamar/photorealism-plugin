use std::sync::{Mutex, OnceLock, TryLockError};

use windows::Win32::Graphics::Dxgi::IDXGISwapChain;

use crate::graphics::Renderer;
use crate::logging;
use crate::settings::Settings;

static RUNTIME: OnceLock<Mutex<Runtime>> = OnceLock::new();

struct Runtime {
    renderer: Option<Renderer>,
    settings: Settings,
    error_reported: bool,
}

impl Runtime {
    fn new() -> Self {
        let path = logging::plugin_path("photorealism-plugin.cfg");
        Self {
            renderer: None,
            settings: Settings::load(&path),
            error_reported: false,
        }
    }

    unsafe fn process(&mut self, swap_chain: &IDXGISwapChain) {
        if !self.settings.enabled {
            return;
        }
        if unsafe { self.ensure_renderer(swap_chain) }.is_err() {
            self.report_error("Falha ao inicializar o pipeline de cor e tonemap.");
            return;
        }
        let Some(renderer) = self.renderer.as_mut() else {
            return;
        };
        if unsafe { renderer.render(swap_chain, self.settings) }.is_err() {
            self.report_error("Falha ao aplicar o passe de cor e tonemap.");
        }
    }

    unsafe fn ensure_renderer(&mut self, swap_chain: &IDXGISwapChain) -> windows::core::Result<()> {
        let matches = self
            .renderer
            .as_ref()
            .is_some_and(|renderer| unsafe { renderer.matches(swap_chain) });
        if matches {
            return Ok(());
        }
        self.renderer = Some(unsafe { Renderer::new(swap_chain)? });
        self.error_reported = false;
        logging::write("Pipeline de cor e tonemap inicializado.");
        Ok(())
    }

    fn report_error(&mut self, message: &str) {
        if self.error_reported {
            return;
        }
        logging::write(message);
        self.error_reported = true;
    }
}

pub fn process(swap_chain: &IDXGISwapChain) {
    let mutex = RUNTIME.get_or_init(|| Mutex::new(Runtime::new()));
    let Ok(mut runtime) = mutex.try_lock() else {
        return;
    };
    unsafe { runtime.process(swap_chain) };
}

pub fn reset() {
    let Some(mutex) = RUNTIME.get() else {
        return;
    };
    match mutex.try_lock() {
        Ok(mut runtime) => runtime.renderer = None,
        Err(TryLockError::Poisoned(error)) => error.into_inner().renderer = None,
        Err(TryLockError::WouldBlock) => {}
    }
}
