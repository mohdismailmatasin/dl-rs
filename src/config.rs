use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

const CONFIG_DIR_NAME: &str = "dl-rs";
const CONFIG_FILE_NAME: &str = "settings.conf";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub download_dir: PathBuf,
    pub connections: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            download_dir: dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("/tmp"))
                .join("Downloads"),
            connections: 16,
        }
    }
}

impl Config {
    pub fn path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| {
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("/tmp"))
                    .join(".config")
            })
            .join(CONFIG_DIR_NAME)
            .join(CONFIG_FILE_NAME)
    }

    pub fn load() -> Self {
        let path = Self::path();
        if !path.exists() {
            return Self::default();
        }

        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => return Self::default(),
        };

        let mut config = Self::default();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some(pos) = line.find('=') {
                let key = line[..pos].trim();
                let value = line[pos + 1..].trim();

                match key {
                    "download_dir" => {
                        config.download_dir = PathBuf::from(value);
                    }
                    "connections" => {
                        if let Ok(n) = value.parse::<u32>() {
                            config.connections = n;
                        }
                    }
                    _ => {}
                }
            }
        }

        config
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = format!(
            "# dl-rs configuration\n\
             # Download directory\n\
             download_dir = {}\n\
             # Number of connections per download\n\
             connections = {}\n",
            self.download_dir.display(),
            self.connections
        );

        let mut file = fs::File::create(&path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    pub fn save_if_not_exists() {
        let path = Self::path();
        if !path.exists() {
            let config = Self::default();
            let _ = config.save();
        }
    }

    pub fn merge_with_args(&self, connections: Option<u32>, dir: Option<PathBuf>) -> Self {
        Self {
            download_dir: dir.unwrap_or_else(|| self.download_dir.clone()),
            connections: connections.unwrap_or(self.connections),
        }
    }
}
