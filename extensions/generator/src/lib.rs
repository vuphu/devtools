mod extension;
mod form;
mod generators;

pub use extension::GeneratorExtension;

devtools_extension_sdk::export_extension!(GeneratorExtension);
