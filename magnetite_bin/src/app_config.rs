use std::error::Error;
use std::path::Path;
use std::{collections::HashMap, fs, path::PathBuf};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

use magnetite_core::state::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    logger_level: String,
    config_path: PathBuf,
    proxy: Option<String>,
    server: Server,
    cache: Cache,
    #[serde(serialize_with = "toml::ser::tables_last")]
    env: HashMap<String, String>,
}

pub fn config_path() -> Result<PathBuf, Box<dyn Error>> {
    if let Some(proj_dirs) = ProjectDirs::from("com", "oiatz", "magnetite") {
        let config_dir = proj_dirs.config_dir();
        if !config_dir.exists() {
            fs::create_dir_all(config_dir)?;
        }
        Ok(proj_dirs.config_dir().join("config.toml"))
    } else {
        Ok(Path::new("./config.toml").into())
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        let app_config = AppConfig {
            server: Server {
                listen: "127.0.0.1".to_string(),
                port: 8080,
            },
            cache: Cache {
                expire: 5 * 60,
                r#type: CacheType::Memory,
            },
            logger_level: "INFO".to_string(),
            proxy: None,
            env: Default::default(),
            config_path: config_path().expect("can not find config file"),
        };

        app_config.save();
        app_config
    }
}

impl AppConfig {
    pub fn into_state(self) -> AppState {
        let redis = match &self.cache.r#type {
            CacheType::Redis { url } => Some(url.clone()),
            CacheType::Memory => None,
        };

        AppState {
            redis,
            cache_expire: self.cache.expire,
            env: self.env,
        }
    }

    pub fn from<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let path = path.as_ref();
        if path.exists() {
            let mut c = config::Config::default();
            c.merge(config::File::from(path.as_ref()))?;
            Ok(c.try_into()?)
        } else {
            Ok(AppConfig::default())
        }
    }

    pub fn address(&self) -> String {
        format!("{}:{}", self.server.listen, self.server.port)
    }

    fn save(&self) {
        eprintln!("self = {:#?}", self);
        let config_str = toml::to_string(&self).expect("serialize config");
        fs::write(&self.config_path, config_str).expect("save config to file")
    }
}

#[derive(Debug, Serialize, Deserialize)]
enum CacheType {
    Redis { url: String },
    Memory,
}

#[derive(Debug, Serialize, Deserialize)]
struct Cache {
    expire: usize,
    r#type: CacheType,
}

#[derive(Debug, Serialize, Deserialize)]
struct Server {
    listen: String,
    port: u16,
}

#[derive(Debug, StructOpt)]
pub struct Opt {
    #[structopt(short, long)]
    pub config: Option<PathBuf>,
    #[structopt(short, long)]
    debug: bool,
}
