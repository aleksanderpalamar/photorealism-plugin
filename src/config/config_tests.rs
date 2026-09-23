use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::{Duration, Instant};

use super::{ConfigSink, ConfigSource, ConfigUpdate, ConfigWatcher};

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

    fn store(&self, contents: &str) -> bool {
        self.replace(Some(contents));
        true
    }
}

impl ConfigSource for Rc<FakeSource> {
    fn read(&self) -> Option<String> {
        self.load()
    }
}

impl ConfigSink for Rc<FakeSource> {
    fn write(&self, contents: &str) -> bool {
        self.store(contents)
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

#[test]
fn storing_replaces_what_the_next_poll_reads() {
    let source = FakeSource::new("exposure=1.25");
    let mut watcher = watcher(&source);
    let start = Instant::now();
    watcher.poll(start);

    assert!(watcher.store("exposure=0.5"));
    let update = watcher.poll(start + INTERVAL);

    assert_eq!(update.settings().exposure, 0.5);
    assert!(matches!(update, ConfigUpdate::Reloaded(_)));
}
