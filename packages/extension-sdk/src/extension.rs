use devtools_core::{ExtensionEvent, ExtensionResponse, Manifest};

pub trait Extension {
    fn manifest(&self) -> Manifest;
    fn on_event(&mut self, event: ExtensionEvent) -> ExtensionResponse;
}
