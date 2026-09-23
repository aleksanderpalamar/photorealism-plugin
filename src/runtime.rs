use std::sync::{Mutex, OnceLock, TryLockError};
use std::time::{Duration, Instant};

use windows::Win32::Graphics::Dxgi::IDXGISwapChain;

use crate::config::{CONFIG_FILE_NAME, ConfigUpdate, ConfigWatcher, FileConfigSource};
use crate::graphics::Renderer;
use crate::input::Shortcut;
use crate::logging;
use crate::menu::Visibility;
use crate::settings::Settings;

static RUNTIME: OnceLock<Mutex<Runtime>> = OnceLock::new();

const RELOAD_INTERVAL: Duration = Duration::from_secs(1);

struct Runtime {
    renderer: Option<Renderer>,
    config: ConfigWatcher<FileConfigSource>,
    menu: Visibility,
    shortcut: Shortcut,
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
            menu: Visibility::default(),
            shortcut: Shortcut::default(),
            active_force_hdr: settings.force_hdr,
            error_reported: false,
        }
    }

    unsafe fn process(&mut self, swap_chain: &IDXGISwapChain) {
        self.poll_menu();
        let settings = self.refresh_settings();
        if !settings.enabled && !self.menu.is_visible() {
            return;
        }
        if unsafe { self.ensure_renderer(swap_chain) }.is_err() {
            self.report_error("Falha ao inicializar o pipeline de cor e tonemap.");
            return;
        }
        let visibility = self.menu;
        let Some(renderer) = self.renderer.as_mut() else {
            return;
        };
        if unsafe { renderer.render(swap_chain, settings, visibility) }.is_err() {
            self.report_error("Falha ao aplicar o passe de cor e tonemap.");
        }
    }

    fn poll_menu(&mut self) {
        if !self.shortcut.triggered() {
            return;
        }
        self.menu = self.menu.toggled();
        logging::write(&format!("Menu {}.", menu_state(self.menu)));
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

fn menu_state(visibility: Visibility) -> &'static str {
    if visibility.is_visible() {
        "aberto"
    } else {
        "fechado"
    }
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
