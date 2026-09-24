mod file;

use std::time::{Duration, Instant};

use crate::settings::Settings;

#[cfg(windows)]
pub use file::ConfigFile;

#[cfg(windows)]
pub const CONFIG_FILE_NAME: &str = "photorealism-plugin.cfg";

pub trait ConfigSource {
    fn read(&self) -> Option<String>;
}

pub trait ConfigSink {
    fn write(&self, contents: &str) -> bool;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ConfigUpdate {
    Unchanged(Settings),
    Reloaded(Settings),
}

impl ConfigUpdate {
    pub fn settings(self) -> Settings {
        match self {
            Self::Unchanged(settings) | Self::Reloaded(settings) => settings,
        }
    }
}

pub struct ConfigWatcher<S: ConfigSource> {
    source: S,
    interval: Duration,
    settings: Settings,
    last_poll: Option<Instant>,
}

impl<S: ConfigSource> ConfigWatcher<S> {
    pub fn new(source: S, interval: Duration) -> Self {
        Self {
            source,
            interval,
            settings: Settings::default(),
            last_poll: None,
        }
    }

    pub fn poll(&mut self, now: Instant) -> ConfigUpdate {
        if !self.is_due(now) {
            return ConfigUpdate::Unchanged(self.settings);
        }
        self.last_poll = Some(now);
        let Some(contents) = self.source.read() else {
            return ConfigUpdate::Unchanged(self.settings);
        };
        let parsed = Settings::parse(&contents);
        if parsed == self.settings {
            return ConfigUpdate::Unchanged(self.settings);
        }
        self.settings = parsed;
        ConfigUpdate::Reloaded(parsed)
    }

    fn is_due(&self, now: Instant) -> bool {
        let Some(last) = self.last_poll else {
            return true;
        };
        now.saturating_duration_since(last) >= self.interval
    }
}

#[cfg(test)]
mod tests {
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;
    use std::time::{Duration, Instant};

    use super::{ConfigSource, ConfigUpdate, ConfigWatcher};

    const INTERVAL: Duration = Duration::from_secs(1);

    struct FakeSource {
        contents: RefCell<Option<String>>,
        reads: Cell<usize>,
    }

    impl FakeSource {
        fn new(contents: &str) -> Rc<Self> {
            Rc::new(Self {
                contents: RefCell::new(Some(contents.to_owned())),
                reads: Cell::new(0),
            })
        }

        fn replace(&self, contents: Option<&str>) {
            *self.contents.borrow_mut() = contents.map(str::to_owned);
        }

        fn load(&self) -> Option<String> {
            self.reads.set(self.reads.get() + 1);
            self.contents.borrow().clone()
        }
    }

    impl ConfigSource for Rc<FakeSource> {
        fn read(&self) -> Option<String> {
            self.load()
        }
    }

    fn watcher(source: &Rc<FakeSource>) -> ConfigWatcher<Rc<FakeSource>> {
        ConfigWatcher::new(Rc::clone(source), INTERVAL)
    }

    #[test]
    fn reports_a_reload_on_the_first_poll() {
        let source = FakeSource::new("exposure=1.25");
        let mut watcher = watcher(&source);

        let update = watcher.poll(Instant::now());

        assert_eq!(update.settings().exposure, 1.25);
        assert!(matches!(update, ConfigUpdate::Reloaded(_)));
    }

    #[test]
    fn keeps_settings_until_the_interval_elapses() {
        let source = FakeSource::new("exposure=1.25");
        let mut watcher = watcher(&source);
        let start = Instant::now();
        watcher.poll(start);

        source.replace(Some("exposure=2.0"));
        let update = watcher.poll(start + INTERVAL / 2);

        assert_eq!(update.settings().exposure, 1.25);
        assert!(matches!(update, ConfigUpdate::Unchanged(_)));
        assert_eq!(source.reads.get(), 1);
    }

    #[test]
    fn reloads_after_the_interval_elapses() {
        let source = FakeSource::new("exposure=1.25");
        let mut watcher = watcher(&source);
        let start = Instant::now();
        watcher.poll(start);

        source.replace(Some("exposure=2.0"));
        let update = watcher.poll(start + INTERVAL);

        assert_eq!(update.settings().exposure, 2.0);
        assert!(matches!(update, ConfigUpdate::Reloaded(_)));
    }

    #[test]
    fn keeps_last_settings_when_the_source_fails() {
        let source = FakeSource::new("exposure=1.25");
        let mut watcher = watcher(&source);
        let start = Instant::now();
        watcher.poll(start);

        source.replace(None);
        let update = watcher.poll(start + INTERVAL);

        assert_eq!(update.settings().exposure, 1.25);
        assert!(matches!(update, ConfigUpdate::Unchanged(_)));
    }

    #[test]
    fn unchanged_contents_do_not_report_a_reload() {
        let source = FakeSource::new("exposure=1.25");
        let mut watcher = watcher(&source);
        let start = Instant::now();
        watcher.poll(start);

        source.replace(Some("exposure=1.25\n# comentario"));
        let update = watcher.poll(start + INTERVAL);

        assert!(matches!(update, ConfigUpdate::Unchanged(_)));
    }
}
