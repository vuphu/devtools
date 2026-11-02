use crate::View;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum ExtensionEvent {
    Init { args: String },
    Input { text: String },
    Tick,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExtensionResponse {
    pub view: Option<View>,
    pub message: Option<String>,
    pub exit: bool,
}
