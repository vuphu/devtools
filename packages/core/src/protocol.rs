use crate::View;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum ExtensionEvent {
    Init { args: String },
    Input { text: String },
    Tick,
    Form { action: FormAction },
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExtensionResponse {
    pub view: Option<View>,
    pub message: Option<String>,
    pub exit: bool,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FormAction {
    Up,
    Down,
    Toggle,
    Decrease,
    Increase,
    Submit,
    Cancel,
}
