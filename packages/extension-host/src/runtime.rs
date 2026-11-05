use anyhow::{Context, Result, ensure};
use devtools_core::{CURRENT_ABI_VERSION, ExtensionEvent, ExtensionResponse, Manifest};
use tokio::runtime::{Builder, Runtime};
use wasmtime::component::{Component, Linker, ResourceTable, TypedFunc};
use wasmtime::{Engine, Store};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};

struct State {
    wasi: WasiCtx,
    table: ResourceTable,
}

impl WasiView for State {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi,
            table: &mut self.table,
        }
    }
}

pub struct ExtensionInstance {
    runtime: Runtime,
    store: Store<State>,
    on_event: TypedFunc<(String,), (String,)>,
    manifest: Manifest,
}

impl ExtensionInstance {
    pub fn load_from_component(engine: &Engine, component: &Component) -> Result<Self> {
        let mut linker = Linker::new(engine);
        wasmtime_wasi::p3::add_to_linker(&mut linker)?;

        let mut store = Store::new(
            engine,
            State {
                wasi: WasiCtxBuilder::new().build(),
                table: ResourceTable::new(),
            },
        );

        let runtime = Builder::new_current_thread().enable_all().build()?;
        let instance = runtime.block_on(linker.instantiate_async(&mut store, component))?;
        let manifest_fn = instance.get_typed_func::<(), (String,)>(&mut store, "manifest")?;
        let (json,) = runtime.block_on(manifest_fn.call_async(&mut store, ()))?;
        let manifest: Manifest =
            serde_json::from_str(&json).context("Invalid extension manifest")?;
        let on_event = instance.get_typed_func(&mut store, "on-event")?;

        ensure!(
            manifest.abi_version == CURRENT_ABI_VERSION,
            "Incompatible extension ABI version {} (host supports {})",
            manifest.abi_version,
            CURRENT_ABI_VERSION
        );

        Ok(Self {
            runtime,
            store,
            on_event,
            manifest,
        })
    }

    pub fn get_manifest(&self) -> &Manifest {
        &self.manifest
    }

    pub fn handle_event(&mut self, event: &ExtensionEvent) -> Result<ExtensionResponse> {
        let json = serde_json::to_string(event)?;
        let (response,) = self
            .runtime
            .block_on(self.on_event.call_async(&mut self.store, (json,)))?;
        serde_json::from_str(&response).context("Invalid extension response")
    }
}
