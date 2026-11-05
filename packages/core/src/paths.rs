use anyhow::{Context, Result};
use std::{fs, path::PathBuf};

pub fn get_root_dir() -> PathBuf {
    if let Ok(val) = std::env::var("DEVTOOLS_HOME") {
        PathBuf::from(val)
    } else if let Some(home) = dirs::home_dir() {
        home.join(".devtools")
    } else {
        PathBuf::from(".devtools")
    }
}

pub fn get_extensions_dir() -> PathBuf {
    get_root_dir().join("extensions")
}

pub fn ensure_devtools_dirs() -> Result<()> {
    fs::create_dir_all(get_extensions_dir())
        .context("Failed to create ~/.devtools/extensions directory")
}
