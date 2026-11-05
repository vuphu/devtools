use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum TextStyle {
    #[default]
    Normal,
    Muted,
    Accent,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Block {
    Header(String),
    Text {
        text: String,
        style: TextStyle,
    },
    KeyValue {
        items: Vec<(String, String)>,
    },
    Table {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
    },
    Help(Vec<(String, String)>),
    Form {
        fields: Vec<FormField>,
        selected: usize,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct View {
    pub title: String,
    pub blocks: Vec<Block>,
    pub input_prompt: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FormField {
    Choice { label: String, value: String },
    Checkbox { label: String, checked: bool },
    Number { label: String, value: usize },
}
