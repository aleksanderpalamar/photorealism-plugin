use std::path::PathBuf;

use super::{ConfigSink, ConfigSource};

pub struct ConfigFile {
    path: PathBuf,
}

impl ConfigFile {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl ConfigSource for ConfigFile {
    fn read(&self) -> Option<String> {
        std::fs::read_to_string(&self.path).ok()
    }
}

impl ConfigSink for ConfigFile {
    fn write(&self, contents: &str) -> bool {
        let temporary = self.path.with_extension("tmp");
        if std::fs::write(&temporary, contents).is_err() {
            return false;
        }
        std::fs::rename(&temporary, &self.path).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::{ConfigFile, ConfigSink, ConfigSource};
    use std::path::PathBuf;

    fn temporary_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("photorealism-{}-{}.cfg", std::process::id(), name))
    }

    #[test]
    fn reads_the_configuration_file() {
        let path = temporary_path("read");
        std::fs::write(&path, "exposure=0.5").expect("arquivo temporario");
        let source = ConfigFile::new(path.clone());

        let contents = source.read();
        let _ = std::fs::remove_file(&path);

        assert_eq!(contents.as_deref(), Some("exposure=0.5"));
    }

    #[test]
    fn reports_a_missing_configuration_file() {
        let path = temporary_path("missing");
        let _ = std::fs::remove_file(&path);

        assert_eq!(ConfigFile::new(path).read(), None);
    }

    #[test]
    fn writes_and_reads_back_the_same_contents() {
        let path = temporary_path("write");
        let file = ConfigFile::new(path.clone());

        assert!(file.write("exposure=1.5\n"));
        let contents = file.read();
        let _ = std::fs::remove_file(&path);

        assert_eq!(contents.as_deref(), Some("exposure=1.5\n"));
    }

    #[test]
    fn writing_replaces_the_previous_contents() {
        let path = temporary_path("replace");
        let file = ConfigFile::new(path.clone());
        file.write("exposure=1.5\n");

        file.write("exposure=0.25\n");
        let contents = file.read();
        let _ = std::fs::remove_file(&path);

        assert_eq!(contents.as_deref(), Some("exposure=0.25\n"));
    }

    #[test]
    fn writing_into_a_missing_directory_fails_without_panicking() {
        let path = temporary_path("missing-directory").join("nested.cfg");

        assert!(!ConfigFile::new(path).write("exposure=1.0\n"));
    }
}
