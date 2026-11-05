use crate::ExtensionInstance;
use anyhow::{Context, Result, bail};
use devtools_core::get_extensions_dir;
use std::{fs, path::PathBuf};

#[derive(Clone)]
pub struct ExtensionRegistry {
    dir: PathBuf,
}

impl Default for ExtensionRegistry {
    fn default() -> Self {
        Self::create(get_extensions_dir())
    }
}

impl ExtensionRegistry {
    pub fn create(dir: PathBuf) -> Self {
        Self { dir }
    }

    pub fn get_extensions(&self) -> Vec<String> {
        let mut results = Vec::new();
        let Ok(entries) = fs::read_dir(&self.dir) else {
            return results;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("wasm") {
                continue;
            }
            if let Some(name) = path.file_stem().and_then(|name| name.to_str()) {
                results.push(name.to_string());
            }
        }
        results.sort();
        results
    }

    pub fn load_extension(&self, name: &str) -> Result<ExtensionInstance> {
        let path = self.dir.join(format!("{}.wasm", name));
        if !path.exists() {
            bail!("Extension '{}' not found in ~/.devtools/extensions", name);
        }
        let bytes = fs::read(&path)
            .with_context(|| format!("Failed to read extension file: {}", path.display()))?;
        ExtensionInstance::load_from_bytes(&bytes)
    }
}
