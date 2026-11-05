use devtools_extension_sdk::{Block, FormAction, FormField, TextStyle, View};

pub enum Field {
    Number {
        label: String,
        value: usize,
        maximum: usize,
    },
    Checkbox {
        label: String,
        checked: bool,
    },
    Choice {
        label: String,
        options: Vec<String>,
        selected: usize,
    },
}

impl Field {
    pub fn create_number(label: &str, value: usize, maximum: usize) -> Self {
        Self::Number {
            label: label.into(),
            value,
            maximum,
        }
    }

    pub fn create_checkbox(label: &str, checked: bool) -> Self {
        Self::Checkbox {
            label: label.into(),
            checked,
        }
    }

    pub fn create_choice(label: &str, options: &[&str]) -> Self {
        Self::Choice {
            label: label.into(),
            options: options.iter().map(|s| (*s).into()).collect(),
            selected: 0,
        }
    }

    pub fn get_value(&self) -> usize {
        match self {
            Self::Number { value, .. } => *value,
            Self::Choice { selected, .. } => *selected,
            _ => 0,
        }
    }

    pub fn is_checked(&self) -> bool {
        matches!(self, Self::Checkbox { checked: true, .. })
    }

    fn build_view(&self) -> FormField {
        match self {
            Self::Number { label, value, .. } => FormField::Number {
                label: label.clone(),
                value: *value,
            },
            Self::Checkbox { label, checked } => FormField::Checkbox {
                label: label.clone(),
                checked: *checked,
            },
            Self::Choice {
                label,
                options,
                selected,
            } => FormField::Choice {
                label: label.clone(),
                value: options[*selected].clone(),
            },
        }
    }
}

pub struct Form {
    title: String,
    pub fields: Vec<Field>,
    selected: usize,
}

impl Form {
    pub fn create(title: &str, fields: Vec<Field>) -> Self {
        Self {
            title: title.into(),
            fields,
            selected: 0,
        }
    }

    pub fn handle_action(&mut self, action: FormAction) {
        match action {
            FormAction::Up => {
                self.selected = (self.selected + self.fields.len() - 1) % self.fields.len()
            }
            FormAction::Down => self.selected = (self.selected + 1) % self.fields.len(),
            action => match &mut self.fields[self.selected] {
                Field::Checkbox { checked, .. } if matches!(action, FormAction::Toggle) => {
                    *checked ^= true
                }
                Field::Number { value, .. } if matches!(action, FormAction::Decrease) => {
                    *value = value.saturating_sub(1).max(1)
                }
                Field::Number { value, maximum, .. } if matches!(action, FormAction::Increase) => {
                    *value = (*value + 1).min(*maximum)
                }
                Field::Choice {
                    selected, options, ..
                } if matches!(action, FormAction::Decrease) => {
                    *selected = (*selected + options.len() - 1) % options.len()
                }
                Field::Choice {
                    selected, options, ..
                } if matches!(action, FormAction::Increase | FormAction::Toggle) => {
                    *selected = (*selected + 1) % options.len()
                }
                _ => {}
            },
        }
    }

    pub fn build_view(&self) -> View {
        View {
            blocks: vec![
                Block::Header(self.title.clone()),
                Block::Form {
                    fields: self.fields.iter().map(Field::build_view).collect(),
                    selected: self.selected,
                },
                Block::Text {
                    text: "↑↓/Tab move · Space toggle · ←→ change · Enter generate · Esc cancel"
                        .into(),
                    style: TextStyle::Muted,
                },
            ],
            ..Default::default()
        }
    }
}
