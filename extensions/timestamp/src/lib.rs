use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use devtools_extension_sdk::*;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Default)]
pub struct TimestampExtension {
    live: bool,
}

impl Extension for TimestampExtension {
    fn get_manifest(&self) -> Manifest {
        Manifest {
            name: "timestamp".into(),
            version: "0.1.0".into(),
            description: "Unix epoch timestamp & human datetime converter".into(),
            abi_version: CURRENT_ABI_VERSION,
        }
    }

    fn handle_event(&mut self, event: ExtensionEvent) -> ExtensionResponse {
        let tick = matches!(event, ExtensionEvent::Tick);
        let text = match event {
            ExtensionEvent::Init { args } => args,
            ExtensionEvent::Input { text } => {
                if matches!(text.trim().to_ascii_lowercase().as_str(), "exit" | "quit")
                    || text.trim() == "q"
                {
                    return ExtensionResponse {
                        message: Some("Exited timestamp tool.".into()),
                        exit: true,
                        ..Default::default()
                    };
                }
                text
            }
            ExtensionEvent::Tick if self.live => String::new(),
            ExtensionEvent::Tick | ExtensionEvent::Form { .. } => {
                return ExtensionResponse::default();
            }
        };
        let query = text.trim();
        self.live = query.is_empty()
            || query.eq_ignore_ascii_case("now")
            || query.eq_ignore_ascii_case("refresh");
        let result = if self.live {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|error| error.to_string())
                .and_then(|duration| {
                    DateTime::from_timestamp_millis(duration.as_millis() as i64)
                        .ok_or_else(|| "Current time is out of range.".into())
                })
        } else {
            parse(query)
        };
        let message = if tick {
            None
        } else {
            Some(match &result {
                Err(error) => format!("Error: {error}"),
                Ok(_) if self.live => "Current time. Updates automatically.".into(),
                Ok(_) => format!("Parsed '{query}'"),
            })
        };
        ExtensionResponse {
            view: Some(build_view(result)),
            message,
            exit: false,
        }
    }
}

fn parse(query: &str) -> Result<DateTime<Utc>, String> {
    if let Ok(epoch) = query.parse::<i64>() {
        let date = if epoch.unsigned_abs() < 100_000_000_000 {
            DateTime::from_timestamp(epoch, 0)
        } else {
            DateTime::from_timestamp_millis(epoch)
        };
        return date.ok_or_else(|| "Timestamp is out of range.".into());
    }
    for format in ["%Y-%m-%d %H:%M:%S", "%Y-%m-%dT%H:%M:%S"] {
        if let Ok(date) = NaiveDateTime::parse_from_str(query, format) {
            return Ok(date.and_utc());
        }
    }
    if let Ok(date) = NaiveDate::parse_from_str(query, "%Y-%m-%d") {
        return Ok(date.and_hms_opt(0, 0, 0).unwrap().and_utc());
    }
    Err(format!(
        "Cannot parse '{query}'. Use an epoch, YYYY-MM-DD, or YYYY-MM-DD HH:MM:SS."
    ))
}

fn build_view(result: Result<DateTime<Utc>, String>) -> View {
    let content = match result {
        Ok(date) => Block::KeyValue {
            items: vec![
                ("Epoch (seconds)".into(), date.timestamp().to_string()),
                ("Epoch (millis)".into(), date.timestamp_millis().to_string()),
                (
                    "UTC".into(),
                    date.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                ),
                ("ISO 8601".into(), date.to_rfc3339()),
                ("Day".into(), date.format("%A").to_string()),
            ],
        },
        Err(text) => Block::Text {
            text,
            style: TextStyle::Error,
        },
    };
    View {
        title: "Timestamp".into(),
        blocks: vec![
            Block::Header("Timestamp".into()),
            content,
            Block::Help(vec![
                ("now / refresh".into(), "Current time".into()),
                ("<epoch>".into(), "Seconds or milliseconds to date".into()),
                ("<date>".into(), "YYYY-MM-DD [HH:MM:SS] to epoch".into()),
                ("exit / q".into(), "Back to shell".into()),
            ]),
        ],
        input_prompt: Some("timestamp> ".into()),
    }
}

export_extension!(TimestampExtension);
