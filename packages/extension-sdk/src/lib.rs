mod exports;
mod extension;

pub use devtools_core::*;
pub use extension::Extension;
pub use serde_json;

#[cfg(target_os = "wasi")]
pub mod bindings {
    wit_bindgen::generate!({
        path: "wit",
        world: "extension",
        pub_export_macro: true,
    });
}
