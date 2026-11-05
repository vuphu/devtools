use crate::generators::{Generator, build_home_view};
use devtools_extension_sdk::*;

#[derive(Default)]

pub struct GeneratorExtension {
    active: Option<Box<dyn Generator>>,
}

impl Extension for GeneratorExtension {
    fn get_manifest(&self) -> Manifest {
        Manifest {
            name: "generator".into(),
            version: "0.1.0".into(),
            description: "Random password and UUID generator".into(),
            abi_version: CURRENT_ABI_VERSION,
        }
    }

    fn handle_event(&mut self, event: ExtensionEvent) -> ExtensionResponse {
        let text = match event {
            ExtensionEvent::Tick => return ExtensionResponse::default(),
            ExtensionEvent::Form { action } => {
                let Some(tool) = &mut self.active else {
                    return ExtensionResponse::default();
                };
                let view = match action {
                    FormAction::Cancel => {
                        self.active = None;
                        build_home_view()
                    }
                    FormAction::Submit => match tool.generate() {
                        Ok(view) => {
                            self.active = None;
                            view
                        }
                        Err(error) => {
                            let mut view = tool.get_form().build_view();
                            view.blocks.push(Block::Text {
                                text: error.to_string(),
                                style: TextStyle::Error,
                            });
                            view
                        }
                    },
                    action => {
                        tool.get_form().handle_action(action);
                        tool.get_form().build_view()
                    }
                };
                return ExtensionResponse {
                    view: Some(view),
                    ..Default::default()
                };
            }
            ExtensionEvent::Init { args } => args,
            ExtensionEvent::Input { text } => text,
        };
        if matches!(text.trim(), "/exit" | "exit" | "q" | "quit") {
            return ExtensionResponse {
                exit: true,
                ..Default::default()
            };
        }
        self.active = match text.trim() {
            "/password" => Some(Box::new(crate::generators::password::Password::default())),
            "/uuid" => Some(Box::new(crate::generators::uuid::Uuid::default())),
            _ => None,
        };
        let view = if let Some(tool) = &mut self.active {
            tool.get_form().build_view()
        } else {
            let mut view = build_home_view();
            if !text.trim().is_empty() {
                view.blocks.push(Block::Text {
                    text: "Unknown command. Use /password or /uuid.".into(),
                    style: TextStyle::Error,
                });
            }
            view
        };
        ExtensionResponse {
            view: Some(view),
            ..Default::default()
        }
    }
}
