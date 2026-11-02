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
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct View {
    pub title: String,
    pub blocks: Vec<Block>,
    pub input_prompt: Option<String>,
}
