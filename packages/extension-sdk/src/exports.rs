#[macro_export]
macro_rules! export_extension {
    ($ext_type:ident) => {
        #[cfg(target_os = "wasi")]
        mod component {
            use super::$ext_type;

            static INSTANCE: std::sync::Mutex<Option<$ext_type>> = std::sync::Mutex::new(None);

            impl $crate::bindings::Guest for $ext_type {
                fn manifest() -> String {
                    let mut instance = INSTANCE.lock().unwrap();
                    let extension = instance.get_or_insert_with(Default::default);
                    let manifest = $crate::Extension::get_manifest(extension);
                    $crate::serde_json::to_string(&manifest).unwrap()
                }

                fn on_event(event: String) -> String {
                    let response = match $crate::serde_json::from_str(&event) {
                        Ok(event) => {
                            let mut instance = INSTANCE.lock().unwrap();
                            let extension = instance.get_or_insert_with(Default::default);
                            $crate::Extension::handle_event(extension, event)
                        }
                        Err(error) => $crate::ExtensionResponse {
                            message: Some(format!("Invalid event payload: {error}")),
                            ..Default::default()
                        },
                    };
                    $crate::serde_json::to_string(&response).unwrap()
                }
            }

            $crate::bindings::export!($ext_type with_types_in $crate::bindings);
        }
    };
}
