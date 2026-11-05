use crate::form::{Field, Form};
use crate::generators::{Generator, build_result_view};
use anyhow::{Context, Result};
use devtools_extension_sdk::*;

struct UuidVersionOption {
    label: &'static str,
    generate: fn() -> ::uuid::Uuid,
}

const UUID_VERSION_OPTIONS: &[UuidVersionOption] = &[
    UuidVersionOption {
        label: "v4",
        generate: ::uuid::Uuid::new_v4,
    },
    UuidVersionOption {
        label: "v7",
        generate: ::uuid::Uuid::now_v7,
    },
];

pub struct Uuid {
    form: Form,
}

impl Default for Uuid {
    fn default() -> Self {
        let versions: Vec<_> = UUID_VERSION_OPTIONS
            .iter()
            .map(|option| option.label)
            .collect();
        Self {
            form: Form::create(
                "UUID options",
                vec![
                    Field::create_number("Count", 1, 100),
                    Field::create_choice("UUID version", &versions),
                ],
            ),
        }
    }
}

impl Generator for Uuid {
    fn get_form(&mut self) -> &mut Form {
        &mut self.form
    }

    fn generate(&self) -> Result<View> {
        let version = UUID_VERSION_OPTIONS
            .get(self.form.fields[1].get_value())
            .context("Select a valid UUID version.")?;
        let mut values = Vec::new();
        for _ in 0..self.form.fields[0].get_value() {
            let uuid = (version.generate)();
            values.push(uuid.to_string());
        }
        Ok(build_result_view(
            &format!("Random UUID {}", version.label),
            values,
        ))
    }
}
