use crate::ExtensionInstance;
use anyhow::{Context, Result, bail};
use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use wasmtime::component::Component;
use wasmtime::{Config, Engine};

struct CachedComponent {
    bytes: Vec<u8>,
    component: Component,
}

#[derive(Clone)]
pub struct ExtensionRegistry {
    dir: PathBuf,
    engine: Engine,
    components: Arc<Mutex<HashMap<String, CachedComponent>>>,
}

impl ExtensionRegistry {
    pub fn create(dir: PathBuf) -> Result<Self> {
        let mut config = Config::new();
        config
            .wasm_component_model_async(true)
            .wasm_component_model_async_stackful(true)
            .concurrency_support(true);
        let engine = Engine::new(&config)
            .map_err(anyhow::Error::from)
            .context("Failed to create extension engine")?;
        Ok(Self {
            dir,
            engine,
            components: Default::default(),
        })
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
        let component = {
            let mut components = self
                .components
                .lock()
                .map_err(|_| anyhow::anyhow!("Extension component cache lock poisoned"))?;
            if let Some(cached) = components.get(name).filter(|cached| cached.bytes == bytes) {
                cached.component.clone()
            } else {
                let component = Component::new(&self.engine, &bytes)
                    .map_err(anyhow::Error::from)
                    .context("Invalid WASI component")?;
                components.insert(
                    name.to_string(),
                    CachedComponent {
                        bytes,
                        component: component.clone(),
                    },
                );
                component
            }
        };
        ExtensionInstance::load_from_component(&self.engine, &component)
    }
}
