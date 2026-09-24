use std::sync::{Mutex, OnceLock, TryLockError};
use std::time::{Duration, Instant};

use windows::Win32::Graphics::Dxgi::IDXGISwapChain;

use crate::config::{CONFIG_FILE_NAME, ConfigUpdate, ConfigWatcher, FileConfigSource};
use crate::dinput;
use crate::graphics::Renderer;
use crate::input::Shortcut;
use crate::logging;
use crate::menu::{Session, Viewport};
use crate::settings::Settings;

static RUNTIME: OnceLock<Mutex<Runtime>> = OnceLock::new();

const RELOAD_INTERVAL: Duration = Duration::from_secs(1);
const DEFAULT_VIEWPORT: Viewport = Viewport {
    width: 1920.0,
    height: 1080.0,
};

struct Runtime {
    renderer: Option<Renderer>,
    config: ConfigWatcher<FileConfigSource>,
    session: Session,
    shortcut: Shortcut,
    capturing: bool,
    active_force_hdr: bool,
    error_reported: bool,
}

impl Runtime {
    fn new() -> Self {
        let source = FileConfigSource::new(logging::plugin_path(CONFIG_FILE_NAME));
        let mut config = ConfigWatcher::new(source, RELOAD_INTERVAL);
        let settings = config.poll(Instant::now()).settings();
        logging::write(&format!("Configuracao inicial: {}", describe(settings)));
        Self {
            renderer: None,
            config,
            session: Session::default(),
            shortcut: Shortcut::default(),
            capturing: false,
            active_force_hdr: settings.force_hdr,
            error_reported: false,
        }
    }

    unsafe fn process(&mut self, swap_chain: &IDXGISwapChain) {
        let stored = self.refresh_settings();
        let viewport = self.viewport();
        self.poll_menu(viewport);
        let settings = self.session.settings(stored, viewport);
        if !settings.enabled && !self.session.is_visible() {
            self.set_capture(false);
            return;
        }
        if unsafe { self.ensure_renderer(swap_chain) }.is_err() {
            self.report_error("Falha ao inicializar o pipeline de cor e tonemap.");
            self.set_capture(false);
            return;
        }
        let session = self.session;
        let Some(renderer) = self.renderer.as_mut() else {
            self.set_capture(false);
            return;
        };
        if unsafe { renderer.render(swap_chain, settings, &session) }.is_err() {
            self.report_error("Falha ao aplicar o passe de cor e tonemap.");
            self.set_capture(false);
            return;
        }
        self.set_capture(self.session.is_visible() && self.renderable().is_some());
    }

    fn set_capture(&mut self, capturing: bool) {
        if self.capturing == capturing {
            return;
        }
        self.capturing = capturing;
        if !capturing {
            dinput::release();
        }
        dinput::set_capturing(capturing);
    }

    fn renderable(&self) -> Option<Viewport> {
        self.renderer.as_ref().and_then(Renderer::viewport)
    }

    fn viewport(&self) -> Viewport {
        self.renderable().unwrap_or(DEFAULT_VIEWPORT)
    }

    fn poll_menu(&mut self, viewport: Viewport) {
        if self.shortcut.triggered() {
            self.session.toggle(viewport);
            logging::write(&format!("Menu {}.", menu_state(self.session.is_visible())));
        }
        if !self.session.is_visible() {
            return;
        }
        let (horizontal, vertical) = dinput::take_motion();
        self.session.move_pointer(horizontal, vertical, viewport);
        self.session.set_button(dinput::button());
    }

    fn refresh_settings(&mut self) -> Settings {
        match self.config.poll(Instant::now()) {
            ConfigUpdate::Unchanged(settings) => settings,
            ConfigUpdate::Reloaded(settings) => {
                self.report_reload(settings);
                settings
            }
        }
    }

    fn report_reload(&self, settings: Settings) {
        logging::write(&format!("Configuracao recarregada: {}", describe(settings)));
        if settings.force_hdr != self.active_force_hdr {
            logging::write("force_hdr so e aplicado na inicializacao do jogo.");
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

fn menu_state(visible: bool) -> &'static str {
    if visible { "aberto" } else { "fechado" }
}

fn describe(settings: Settings) -> String {
    format!(
        "enabled={} exposure={:.2} contrast={:.2} saturation={:.2} temperature={:.0} \
         tint={:.2} highlight_rolloff={:.2} paper_white={:.0} peak={}",
        settings.enabled,
        settings.exposure,
        settings.contrast,
        settings.saturation,
        settings.temperature,
        settings.tint,
        settings.highlight_rolloff,
        settings.hdr_paper_white_nits,
        settings.hdr_peak_nits,
    )
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
