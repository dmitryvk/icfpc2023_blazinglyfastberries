use config::{File, FileFormat};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize)]
pub struct Solver {
    pub problems: Directory,
    pub solutions: Directory,
    pub log: Log,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Directory {
    pub dir: PathBuf,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Log {
    pub level: log::LevelFilter,
    pub output: LogOutput,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogOutput {
    StdOut,
    File(String),
}

impl Solver {
    pub fn from_file(file_name: &str) -> anyhow::Result<Self> {
        let config_builder =
            config::Config::builder().add_source(File::new(file_name, FileFormat::Toml));
        let config = config_builder.build()?;
        let config: Solver = config.try_deserialize()?;
        Ok(config)
    }
}
