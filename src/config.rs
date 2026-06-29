use std::path::PathBuf;

pub struct Config {
    pub db_path: Option<PathBuf>,
}

impl Config {
    pub fn load() -> Self {
        let raw = dirs::config_dir()
            .map(|d| d.join("staze").join("config.toml"))
            .and_then(|p| std::fs::read_to_string(p).ok())
            .unwrap_or_default();

        let db_path = toml::from_str::<toml::Value>(&raw).ok()
            .and_then(|v| v.get("db_path")?.as_str().map(PathBuf::from));

        Config { db_path }
    }

    pub fn resolved_staze_path(&self) -> PathBuf {
        self.db_path.clone().unwrap_or_else(|| {
            dirs::data_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("staze")
        })
    }

    pub fn resolved_db_path(&self) -> PathBuf {
        self.db_path.clone().unwrap_or_else(|| {
            self.resolved_staze_path()
                .join("staze.db")
        })
    }    
    
    pub fn resolved_csv_path(&self) -> PathBuf {
        self.db_path.clone().unwrap_or_else(|| {
            self.resolved_staze_path()
                .join("sessions.csv")
        })
    }
}
