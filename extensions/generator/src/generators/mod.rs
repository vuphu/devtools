pub(crate) mod password;
pub(crate) mod uuid;

use crate::form::Form;
use anyhow::Result;
use devtools_extension_sdk::*;

pub trait Generator: Send {
    fn get_form(&mut self) -> &mut Form;

    fn generate(&self) -> Result<View>;
}

pub fn fill_random_bytes(bytes: &mut [u8]) -> Result<()> {
    getrandom::fill(bytes)
        .map_err(|error| anyhow::anyhow!("System randomness unavailable: {error}"))
}

pub fn get_hints() -> Vec<(String, String)> {
    vec![
        ("/password".into(), "Choose password options".into()),
        ("/uuid".into(), "Choose UUID options".into()),
        ("/exit".into(), "Back to shell".into()),
    ]
}

pub fn build_home_view() -> View {
    View {
        blocks: vec![Block::Header("Generator".into()), Block::Help(get_hints())],
        ..Default::default()
    }
}

pub fn build_result_view(title: &str, values: Vec<String>) -> View {
    View {
        blocks: vec![
            Block::Header(title.into()),
            Block::Text {
                text: values.into_iter().map(|value| value + "\n").collect(),
                style: TextStyle::Normal,
            },
            Block::Help(get_hints()),
        ],
        ..Default::default()
    }
}
