use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use devtools_core::{ExtensionEvent, ExtensionResponse, View, ensure_devtools_dirs};
use devtools_extension_host::{ExtensionInstance, ExtensionRegistry};

pub enum ActiveMode {
    Shell,
    Loading {
        name: String,
        started: std::time::Instant,
        task: std::thread::JoinHandle<Result<(ExtensionInstance, ExtensionResponse)>>,
    },
    Extension {
        name: String,
        instance: ExtensionInstance,
        current_view: Option<View>,
    },
}

pub struct App {
    pub mode: ActiveMode,
    registry: ExtensionRegistry,
    installed: Vec<String>,
    history: std::collections::HashMap<String, Vec<String>>,
    history_index: Option<usize>,
    history_draft: String,
    pub input: String,
    pub input_cursor: usize,
    pub completion_index: usize,
    pub status_message: String,
    pub status_is_error: bool,
    pub should_quit: bool,
}

impl App {
    pub fn create() -> Result<Self> {
        ensure_devtools_dirs()?;
        let registry = ExtensionRegistry::create(devtools_core::get_extensions_dir())?;
        let installed = registry.get_extensions();
        Ok(Self {
            mode: ActiveMode::Shell,
            registry,
            installed,
            history: Default::default(),
            history_index: None,
            history_draft: String::new(),
            input: String::new(),
            input_cursor: 0,
            completion_index: 0,
            status_message: "Ready. Type /help or /e:<name> to start.".to_string(),
            status_is_error: false,
            should_quit: false,
        })
    }

    fn set_status(&mut self, msg: impl Into<String>, is_error: bool) {
        self.status_message = msg.into();
        self.status_is_error = is_error;
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }
        if key.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(key.code, KeyCode::Char('c' | 'd'))
        {
            self.should_quit = true;
            return;
        }
        if matches!(self.mode, ActiveMode::Loading { .. }) && key.code != KeyCode::Esc {
            return;
        }
        if let ActiveMode::Extension {
            instance,
            current_view: Some(view),
            ..
        } = &mut self.mode
            && view
                .blocks
                .iter()
                .any(|block| matches!(block, devtools_core::Block::Form { .. }))
        {
            use devtools_core::FormAction;
            let action = match key.code {
                KeyCode::Up | KeyCode::BackTab => FormAction::Up,
                KeyCode::Down | KeyCode::Tab => FormAction::Down,
                KeyCode::Char(' ') => FormAction::Toggle,
                KeyCode::Left => FormAction::Decrease,
                KeyCode::Right => FormAction::Increase,
                KeyCode::Enter => FormAction::Submit,
                KeyCode::Esc => FormAction::Cancel,
                _ => return,
            };
            match instance.handle_event(&ExtensionEvent::Form { action }) {
                Ok(response) => {
                    if let Some(updated) = response.view {
                        *view = updated;
                    }
                    if let Some(message) = response.message {
                        self.set_status(message, false);
                    }
                }
                Err(error) => self.set_status(format!("Extension error: {error:#}"), true),
            }
            return;
        }
        match key.code {
            KeyCode::Tab => return self.complete_input(),
            KeyCode::Up => return self.navigate_input(true),
            KeyCode::Down => return self.navigate_input(false),
            KeyCode::Esc => return self.handle_escape(),
            KeyCode::Enter if !self.get_completions().is_empty() => return self.complete_input(),
            KeyCode::Enter => return self.submit_input(),
            KeyCode::Char(c) => {
                self.history_index = None;
                self.input.insert(self.input_cursor, c);
                self.input_cursor += c.len_utf8();
            }
            KeyCode::Backspace if self.input_cursor > 0 => {
                self.history_index = None;
                self.move_cursor_left();
                self.input.remove(self.input_cursor);
            }
            KeyCode::Delete if self.input_cursor < self.input.len() => {
                self.history_index = None;
                self.input.remove(self.input_cursor);
            }
            KeyCode::Left => self.move_cursor_left(),
            KeyCode::Right if self.input_cursor < self.input.len() => {
                self.input_cursor += self.input[self.input_cursor..]
                    .chars()
                    .next()
                    .unwrap()
                    .len_utf8();
            }
            KeyCode::Home => self.input_cursor = 0,
            KeyCode::End => self.input_cursor = self.input.len(),
            _ => return,
        }
        self.completion_index = self.get_completions().len().saturating_sub(1);
    }

    fn move_cursor_left(&mut self) {
        self.input_cursor = self.input[..self.input_cursor]
            .char_indices()
            .next_back()
            .map_or(0, |(index, _)| index);
    }

    pub fn get_completions(&self) -> Vec<String> {
        if self.history_index.is_some()
            || self.input.is_empty()
            || self.input_cursor != self.input.len()
        {
            return Vec::new();
        }
        let mut candidates = match &self.mode {
            ActiveMode::Loading { .. } => return Vec::new(),
            ActiveMode::Shell => {
                let mut commands = vec!["/help".to_string(), "/exit".to_string()];
                for extension in &self.installed {
                    commands.push(format!("/e:{extension}"));
                }
                commands
            }
            ActiveMode::Extension { current_view, .. } => {
                let mut commands = vec!["/exit".to_string()];
                if let Some(view) = current_view {
                    for block in &view.blocks {
                        if let devtools_core::Block::Help(hints) = block {
                            for (command, _) in hints {
                                for word in command.split(" / ") {
                                    if !word.contains(['<', '>']) {
                                        commands.push(word.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
                commands
            }
        };
        candidates.retain(|candidate| {
            candidate.starts_with(&self.input)
                && candidate != &self.input
                && (!candidate.contains(' ') || self.input.contains(' '))
        });
        candidates.sort();
        candidates.dedup();
        candidates
    }

    fn select_completion(&mut self, previous: bool) {
        let count = self.get_completions().len();
        if count > 0 {
            self.completion_index = if previous {
                (self.completion_index + count - 1) % count
            } else {
                (self.completion_index + 1) % count
            };
        }
    }

    fn get_history_context(&self) -> String {
        match &self.mode {
            ActiveMode::Shell => String::new(),
            ActiveMode::Extension { name, .. } | ActiveMode::Loading { name, .. } => name.clone(),
        }
    }

    fn navigate_input(&mut self, previous: bool) {
        if !self.get_completions().is_empty() {
            self.select_completion(previous);
            return;
        }
        let context = self.get_history_context();
        let Some(history) = self
            .history
            .get(&context)
            .filter(|history| !history.is_empty())
        else {
            return;
        };
        self.history_index = match (previous, self.history_index) {
            (true, None) => {
                self.history_draft = self.input.clone();
                Some(history.len() - 1)
            }
            (true, Some(index)) => Some(index.saturating_sub(1)),
            (false, Some(index)) if index + 1 < history.len() => Some(index + 1),
            (false, Some(_)) => None,
            (false, None) => return,
        };
        self.input = self.history_index.map_or_else(
            || self.history_draft.clone(),
            |index| history[index].clone(),
        );
        self.input_cursor = self.input.len();
        self.completion_index = 0;
    }

    fn complete_input(&mut self) {
        let candidates = self.get_completions();
        if let Some(candidate) = candidates.get(self.completion_index) {
            self.input = candidate.clone();
            self.input_cursor = self.input.len();
            self.completion_index = self.get_completions().len().saturating_sub(1);
        }
    }

    fn submit_input(&mut self) {
        let input = std::mem::take(&mut self.input);
        self.input_cursor = 0;
        self.completion_index = 0;
        let text = input.trim();

        self.history_index = None;
        self.history_draft.clear();
        if !text.is_empty() {
            let context = self.get_history_context();
            let history = self.history.entry(context).or_default();
            if history.last().is_none_or(|previous| previous != text) {
                history.push(text.to_string());
            }
        }
        let ActiveMode::Extension { name, instance, .. } = &mut self.mode else {
            self.handle_shell_command(text);
            return;
        };
        if text.eq_ignore_ascii_case("/exit") || text.eq_ignore_ascii_case("exit") {
            self.handle_escape();
            return;
        }
        let exit_message = format!("Exited extension '{}'", name);
        let response = match instance.handle_event(&ExtensionEvent::Input {
            text: text.to_string(),
        }) {
            Ok(response) => response,
            Err(error) => {
                self.set_status(format!("Extension error: {error:#}"), true);
                return;
            }
        };
        if response.exit {
            self.mode = ActiveMode::Shell;
            self.set_status(response.message.unwrap_or(exit_message), false);
            return;
        }
        if let ActiveMode::Extension { current_view, .. } = &mut self.mode {
            if let Some(view) = response.view {
                *current_view = Some(view);
            }
        }
        if let Some(message) = response.message {
            self.set_status(message, false);
        }
    }

    fn handle_shell_command(&mut self, line: &str) {
        match line {
            "" => return,
            "/exit" => {
                self.should_quit = true;
                return;
            }
            "/help" => {
                self.set_status("/e:<name> [args]  |  /help  |  /exit", false);
                return;
            }
            _ => {}
        }

        if let Some(cmd) = line.strip_prefix("/e:") {
            let mut parts = cmd.splitn(2, ' ');
            let ext_name = parts.next().unwrap_or("").trim();
            let args = parts.next().unwrap_or("").trim();

            if ext_name.is_empty() {
                self.set_status("Usage: /e:<name> [args]", true);
                return;
            }

            self.launch_extension(ext_name, args);
            return;
        }

        self.set_status(
            format!("Unknown command: '{}'. Type /help for assistance.", line),
            true,
        );
    }

    fn launch_extension(&mut self, name: &str, args: &str) {
        self.set_status(format!("Loading {name}…"), false);
        let registry = self.registry.clone();
        let extension_name = name.to_string();
        let args = args.to_string();
        let task = std::thread::spawn(move || {
            let mut instance = registry.load_extension(&extension_name)?;
            let response = instance.handle_event(&ExtensionEvent::Init { args })?;
            Ok((instance, response))
        });
        self.mode = ActiveMode::Loading {
            name: name.to_string(),
            started: std::time::Instant::now(),
            task,
        };
    }

    pub fn poll_loading(&mut self) {
        if !matches!(&self.mode, ActiveMode::Loading { task, .. } if task.is_finished()) {
            return;
        }
        let ActiveMode::Loading { name, task, .. } =
            std::mem::replace(&mut self.mode, ActiveMode::Shell)
        else {
            return;
        };
        let result = task
            .join()
            .unwrap_or_else(|_| Err(anyhow::anyhow!("Extension loader stopped unexpectedly")));
        match result {
            Ok((instance, response)) => {
                let message = response.message.unwrap_or_else(|| format!("Loaded {name}"));
                self.set_status(message, false);
                if !response.exit {
                    self.mode = ActiveMode::Extension {
                        name,
                        instance,
                        current_view: response.view,
                    };
                }
            }
            Err(error) => self.set_status(format!("Failed to launch '{name}': {error:#}"), true),
        }
    }

    pub fn handle_tick(&mut self) {
        if let ActiveMode::Extension {
            instance,
            current_view,
            ..
        } = &mut self.mode
            && let Ok(response) = instance.handle_event(&ExtensionEvent::Tick)
            && let Some(view) = response.view
        {
            *current_view = Some(view);
        }
    }

    fn handle_escape(&mut self) {
        self.history_index = None;
        self.history_draft.clear();
        self.completion_index = 0;
        if let ActiveMode::Extension { name, .. } | ActiveMode::Loading { name, .. } = &self.mode {
            self.set_status(format!("Exited extension '{}'", name), false);
            self.mode = ActiveMode::Shell;
        } else {
            self.input.clear();
            self.input_cursor = 0;
        }
    }
}
