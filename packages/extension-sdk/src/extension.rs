use devtools_core::{ExtensionEvent, ExtensionResponse, Manifest};

pub trait Extension {
    fn get_manifest(&self) -> Manifest;

    fn handle_event(&mut self, event: ExtensionEvent) -> ExtensionResponse;
}
