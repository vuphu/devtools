use crate::form::{Field, Form};
use crate::generators::{Generator, build_result_view, fill_random_bytes};
use anyhow::{Result, bail};
use devtools_extension_sdk::*;

pub struct Password {
    form: Form,
}

impl Default for Password {
    fn default() -> Self {
        Self {
            form: Form::create(
                "Password options",
                vec![
                    Field::create_number("Count", 1, 100),
                    Field::create_number("Password length", 32, 1024),
                    Field::create_checkbox("Lowercase", true),
                    Field::create_checkbox("Uppercase", true),
                    Field::create_checkbox("Digits", true),
                    Field::create_checkbox("Symbols", true),
                ],
            ),
        }
    }
}

impl Generator for Password {
    fn get_form(&mut self) -> &mut Form {
        &mut self.form
    }

    fn generate(&self) -> Result<View> {
        let names = ["lowercase", "uppercase", "digits", "symbols"];
        let groups: Vec<_> = names
            .iter()
            .enumerate()
            .filter_map(|(i, name)| self.form.fields[i + 2].is_checked().then_some(*name))
            .collect();
        if groups.is_empty() {
            bail!("Select at least one character class.");
        }
        let charset = groups.join("+");
        let count = self.form.fields[0].get_value();
        let length = self.form.fields[1].get_value();
        let values = (0..count)
            .map(|_| generate_one(length, &charset))
            .collect::<Result<Vec<_>>>()?;
        Ok(build_result_view("Random passwords", values))
    }
}

fn generate_one(size: usize, charset: &str) -> Result<String> {
    let usage = "Select valid character classes.";
    let args = ["/password", "", charset];
    let custom: Vec<&[u8]> = args
        .get(2)
        .copied()
        .unwrap_or("")
        .split('+')
        .filter_map(|name| match name {
            "lowercase" => Some(b"abcdefghijklmnopqrstuvwxyz".as_slice()),
            "uppercase" => Some(b"ABCDEFGHIJKLMNOPQRSTUVWXYZ".as_slice()),
            "digits" => Some(b"0123456789".as_slice()),
            "symbols" => Some(b"!@#$%^&*()-_=+[]{}:,.?".as_slice()),
            _ => None,
        })
        .collect();
    let groups: &[&[u8]] = match args.get(2).copied().unwrap_or("all") {
        "all" => &[
            b"abcdefghijklmnopqrstuvwxyz",
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZ",
            b"0123456789",
            b"!@#$%^&*()-_=+[]{}:,.?",
        ],
        "alphanumeric" => &[
            b"abcdefghijklmnopqrstuvwxyz",
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZ",
            b"0123456789",
        ],
        "letters" => &[b"abcdefghijklmnopqrstuvwxyz", b"ABCDEFGHIJKLMNOPQRSTUVWXYZ"],
        "digits" => &[b"0123456789"],
        name if !custom.is_empty() && custom.len() == name.split('+').count() => &custom,
        _ => bail!("Usage: {usage}"),
    };
    let alphabet: Vec<u8> = groups.concat();
    // Rejection sampling avoids modulo bias; require each selected character class.
    let password = loop {
        let mut password = Vec::with_capacity(size);
        while password.len() < size {
            let index = generate_random_index(alphabet.len())?;
            password.push(alphabet[index]);
        }
        if size < groups.len()
            || groups
                .iter()
                .all(|group| password.iter().any(|c| group.contains(c)))
        {
            break String::from_utf8(password)?;
        }
    };
    Ok(password)
}

fn generate_random_index(length: usize) -> Result<usize> {
    let limit = 256 - 256 % length;
    loop {
        let mut byte = [0];
        fill_random_bytes(&mut byte)?;
        if (byte[0] as usize) < limit {
            return Ok(byte[0] as usize % length);
        }
    }
}
