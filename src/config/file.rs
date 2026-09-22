use std::path::PathBuf;

use super::ConfigSource;

pub struct FileConfigSource {
    path: PathBuf,
}

impl FileConfigSource {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl ConfigSource for FileConfigSource {
    fn read(&self) -> Option<String> {
        std::fs::read_to_string(&self.path).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::{ConfigSource, FileConfigSource};
    use std::path::PathBuf;

    fn temporary_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("photorealism-{}-{}.cfg", std::process::id(), name))
    }

    #[test]
    fn reads_the_configuration_file() {
        let path = temporary_path("read");
        std::fs::write(&path, "exposure=0.5").expect("arquivo temporario");
        let source = FileConfigSource::new(path.clone());

        let contents = source.read();
        let _ = std::fs::remove_file(&path);

        assert_eq!(contents.as_deref(), Some("exposure=0.5"));
    }

    #[test]
    fn reports_a_missing_configuration_file() {
        let path = temporary_path("missing");
        let _ = std::fs::remove_file(&path);

        assert_eq!(FileConfigSource::new(path).read(), None);
    }
}
