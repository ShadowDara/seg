use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

pub type Config = Vec<ConfVal>;

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConfVal {
    String(String),
    CustomConf(CustomConf),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomConf {
    pub name: String,
    #[serde(rename = "File")]
    pub file: String,
}

pub fn load_config<P: AsRef<Path>>(path: P) -> Result<Config, String> {
    let path = path.as_ref();

    let content = fs::read_to_string(path)
        .map_err(|e| format!("Config-Datei '{}' konnte nicht gelesen werden: {}", path.display(), e))?;

    serde_json::from_str(&content)
        .map_err(|e| format!("Config-Datei '{}' enthält ungültiges JSON: {}", path.display(), e))
}
