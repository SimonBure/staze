use std::path::PathBuf;

pub struct Config {
    pub db_path: Option<PathBuf>,
    pub theme: Option<String>,
    pub galaxy: bool,
    pub session_stars: bool,
}

fn config_file() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("staze").join("config.toml"))
}

fn read_table() -> toml::Table {
    config_file()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|raw| raw.parse::<toml::Table>().ok())
        .unwrap_or_default()
}

impl Config {
    pub fn load() -> Self {
        let table = read_table();
        let flag = |key: &str| table.get(key).and_then(|v| v.as_bool()).unwrap_or(true);

        Config {
            db_path: table.get("db_path").and_then(|v| v.as_str()).map(PathBuf::from),
            theme: table.get("theme").and_then(|v| v.as_str()).map(String::from),
            galaxy: flag("galaxy"),
            session_stars: flag("session_stars"),
        }
    }

    /// Writes the appearance keys, keeping any other keys already in the file.
    pub fn save_appearance(theme: &str, galaxy: bool, session_stars: bool) -> std::io::Result<()> {
        let Some(path) = config_file() else { return Ok(()) };
        let mut table = read_table();
        table.insert("theme".into(), theme.into());
        table.insert("galaxy".into(), galaxy.into());
        table.insert("session_stars".into(), session_stars.into());
        if let Some(dir) = path.parent() { std::fs::create_dir_all(dir)?; }
        std::fs::write(path, table.to_string())
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
