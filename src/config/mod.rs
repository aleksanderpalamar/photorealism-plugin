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

    pub fn store(&mut self, contents: &str) -> bool
    where
        S: ConfigSink,
    {
        self.source.write(contents)
    }

    fn is_due(&self, now: Instant) -> bool {
        let Some(last) = self.last_poll else {
            return true;
        };
        now.saturating_duration_since(last) >= self.interval
    }
}

#[cfg(test)]
#[path = "config_tests.rs"]
mod tests;
