use std::path::PathBuf;

use crate::appearance::{self, Appearance};

pub struct Config {
    pub db_path: Option<PathBuf>,
    pub appearance: Appearance,
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
        let flag = |key: &str, default: bool| table.get(key).and_then(|v| v.as_bool()).unwrap_or(default);
        let int = |key: &str| table.get(key).and_then(|v| v.as_integer());
        let float = |key: &str, default: f32| table.get(key).and_then(|v| v.as_float()).map(|f| f as f32).unwrap_or(default);

        let d = Appearance::default();
        let appearance = Appearance {
            theme: table.get("theme").and_then(|v| v.as_str()).and_then(appearance::theme_by_key).unwrap_or(d.theme),
            galaxy: flag("galaxy", d.galaxy),
            session_stars: flag("session_stars", d.session_stars),
            arms: int("galaxy_arms").map(|v| v.clamp(0, u8::MAX as i64) as u8).unwrap_or(d.arms),
            turn_secs: int("galaxy_turn_secs").map(|v| v.clamp(0, u32::MAX as i64) as u32).unwrap_or(d.turn_secs),
            tilt: float("galaxy_tilt", d.tilt),
            winding: float("galaxy_winding", d.winding),
            sparkle: flag("galaxy_sparkle", d.sparkle),
            field_stars: flag("galaxy_field_stars", d.field_stars),
        }.clamped();

        Config {
            db_path: table.get("db_path").and_then(|v| v.as_str()).map(PathBuf::from),
            appearance,
        }
    }

    /// Writes the appearance keys, keeping any other keys already in the file.
    pub fn save_appearance(a: &Appearance) -> std::io::Result<()> {
        let Some(path) = config_file() else { return Ok(()) };
        // round so hand-read values stay tidy (0.55, not 0.550000011920929)
        let round = |f: f32| (f as f64 * 100.0).round() / 100.0;
        let mut table = read_table();
        table.insert("theme".into(), appearance::THEMES[a.theme].key.into());
        table.insert("galaxy".into(), a.galaxy.into());
        table.insert("session_stars".into(), a.session_stars.into());
        table.insert("galaxy_arms".into(), (a.arms as i64).into());
        table.insert("galaxy_turn_secs".into(), (a.turn_secs as i64).into());
        table.insert("galaxy_tilt".into(), round(a.tilt).into());
        table.insert("galaxy_winding".into(), round(a.winding).into());
        table.insert("galaxy_sparkle".into(), a.sparkle.into());
        table.insert("galaxy_field_stars".into(), a.field_stars.into());
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
