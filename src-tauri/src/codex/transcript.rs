use serde::Serialize;
use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use super::error::CodexError;
use super::home::expand_tilde;

pub(crate) const TRANSCRIPT_LINE_LIMIT: u64 = 2_000;
pub(crate) const TRANSCRIPT_MESSAGE_CHAR_LIMIT: usize = 12_000;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexTranscript {
    pub path: String,
    pub exists: bool,
    pub line_count: u64,
    pub invalid_line_count: u64,
    pub truncated: bool,
    pub messages: Vec<CodexTranscriptMessage>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexTranscriptMessage {
    pub line_number: u64,
    pub timestamp: Option<String>,
    pub role: String,
    pub text: String,
}

#[derive(Debug)]
pub(crate) struct TranscriptMessagesRead {
    pub exists: bool,
    pub line_count: u64,
    pub invalid_line_count: u64,
    pub truncated: bool,
    pub messages: Vec<CodexTranscriptMessage>,
}

pub fn read_transcript(path: String) -> Result<CodexTranscript, CodexError> {
    let trimmed_path = path.trim();
    if trimmed_path.is_empty() {
        return Err(CodexError::TranscriptPathUnavailable);
    }

    let path_buf = expand_tilde(trimmed_path);
    let display_path = path_buf.to_string_lossy().into_owned();

    let transcript = read_transcript_messages(&path_buf)?;

    Ok(CodexTranscript {
        path: display_path,
        exists: transcript.exists,
        line_count: transcript.line_count,
        invalid_line_count: transcript.invalid_line_count,
        truncated: transcript.truncated,
        messages: transcript.messages,
    })
}

pub(crate) fn read_transcript_messages(
    path: &Path,
) -> Result<TranscriptMessagesRead, CodexError> {
    if !path.exists() {
        return Ok(TranscriptMessagesRead {
            exists: false,
            line_count: 0,
            invalid_line_count: 0,
            truncated: false,
            messages: Vec::new(),
        });
    }

    let file = File::open(path).map_err(|source| CodexError::TranscriptRead {
        path: path.to_path_buf(),
        source,
    })?;
    let reader = BufReader::new(file);
    let mut line_count = 0;
    let mut invalid_line_count = 0;
    let mut truncated = false;
    let mut messages = Vec::new();

    for line_result in reader.lines() {
        if line_count >= TRANSCRIPT_LINE_LIMIT {
            truncated = true;
            break;
        }

        let line = line_result.map_err(|source| CodexError::TranscriptRead {
            path: path.to_path_buf(),
            source,
        })?;
        line_count += 1;

        if line.trim().is_empty() {
            continue;
        }

        let value = match serde_json::from_str::<Value>(&line) {
            Ok(value) => value,
            Err(_) => {
                invalid_line_count += 1;
                continue;
            }
        };

        if let Some(message) = extract_message(line_count, &value) {
            messages.push(message);
        }
    }

    Ok(TranscriptMessagesRead {
        exists: true,
        line_count,
        invalid_line_count,
        truncated,
        messages,
    })
}

fn extract_message(line_number: u64, value: &Value) -> Option<CodexTranscriptMessage> {
    let timestamp = value
        .get("timestamp")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    let root_type = value.get("type").and_then(Value::as_str);

    for candidate in message_candidates(value) {
        let candidate_type = candidate.get("type").and_then(Value::as_str).or(root_type);
        let role = candidate
            .get("role")
            .and_then(Value::as_str)
            .map(normalize_role)
            .unwrap_or_else(|| infer_role(candidate_type));

        if let Some(text) =
            extract_candidate_text(candidate).or_else(|| function_call_text(candidate))
        {
            let text = text.trim();
            if !text.is_empty() {
                return Some(CodexTranscriptMessage {
                    line_number,
                    timestamp,
                    role,
                    text: truncate_message(text),
                });
            }
        }
    }

    None
}

fn message_candidates(value: &Value) -> Vec<&Value> {
    let mut candidates = vec![value];

    if let Some(payload) = value.get("payload") {
        candidates.push(payload);

        if let Some(item) = payload.get("item") {
            candidates.push(item);
        }
    }

    if let Some(item) = value.get("item") {
        candidates.push(item);
    }

    candidates
}

fn extract_candidate_text(value: &Value) -> Option<String> {
    value
        .get("content")
        .and_then(text_from_value)
        .or_else(|| value.get("message").and_then(text_from_value))
        .or_else(|| value.get("text").and_then(text_from_value))
}

fn text_from_value(value: &Value) -> Option<String> {
    if let Some(text) = value.as_str() {
        return Some(text.to_string());
    }

    if let Some(items) = value.as_array() {
        let parts = items
            .iter()
            .filter_map(text_from_value)
            .filter(|part| !part.trim().is_empty())
            .collect::<Vec<_>>();

        if parts.is_empty() {
            return None;
        }

        return Some(parts.join("\n"));
    }

    if let Some(object) = value.as_object() {
        return object
            .get("text")
            .and_then(text_from_value)
            .or_else(|| object.get("content").and_then(text_from_value))
            .or_else(|| object.get("message").and_then(text_from_value));
    }

    None
}

fn function_call_text(value: &Value) -> Option<String> {
    let value_type = value.get("type").and_then(Value::as_str)?;
    if value_type != "function_call" {
        return None;
    }

    let name = value
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or("function_call");
    let arguments = value
        .get("arguments")
        .and_then(text_from_value)
        .unwrap_or_default();

    Some(format!("{name} {arguments}").trim().to_string())
}

fn normalize_role(role: &str) -> String {
    match role.to_lowercase().as_str() {
        "assistant" | "agent" => "assistant".to_string(),
        "user" => "user".to_string(),
        "system" => "system".to_string(),
        "tool" | "function" => "tool".to_string(),
        other => other.to_string(),
    }
}

fn infer_role(value_type: Option<&str>) -> String {
    let Some(value_type) = value_type else {
        return "event".to_string();
    };
    let normalized = value_type.to_lowercase();

    if normalized.contains("user") {
        return "user".to_string();
    }

    if normalized.contains("assistant") || normalized.contains("agent") {
        return "assistant".to_string();
    }

    if normalized.contains("system") {
        return "system".to_string();
    }

    if normalized.contains("tool") || normalized.contains("function") {
        return "tool".to_string();
    }

    normalized
}

fn truncate_message(text: &str) -> String {
    let mut chars = text.chars();
    let truncated = chars
        .by_ref()
        .take(TRANSCRIPT_MESSAGE_CHAR_LIMIT)
        .collect::<String>();

    if chars.next().is_some() {
        format!("{truncated}\n[message truncated]")
    } else {
        truncated
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_transcript_path() -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        env::temp_dir().join(format!("codex-history-manager-transcript-{suffix}.jsonl"))
    }

    #[test]
    fn read_transcript_extracts_messages_and_counts_invalid_lines() {
        let path = temp_transcript_path();
        let mut file = File::create(&path).expect("transcript fixture should be created");

        writeln!(
            file,
            r#"{{"timestamp":"2026-05-22T00:00:00Z","type":"response_item","payload":{{"type":"message","role":"user","content":[{{"type":"input_text","text":"hello"}}]}}}}"#
        )
        .expect("user message should be written");
        writeln!(
            file,
            r#"{{"timestamp":"2026-05-22T00:00:01Z","type":"event_msg","payload":{{"type":"agent_message","message":"hi there"}}}}"#
        )
        .expect("assistant message should be written");
        writeln!(file, "not-json").expect("invalid line should be written");

        let transcript = read_transcript(path.to_string_lossy().into_owned())
            .expect("transcript should be parsed");

        assert!(transcript.exists);
        assert_eq!(transcript.line_count, 3);
        assert_eq!(transcript.invalid_line_count, 1);
        assert_eq!(transcript.messages.len(), 2);
        assert_eq!(transcript.messages[0].role, "user");
        assert_eq!(transcript.messages[0].text, "hello");
        assert_eq!(transcript.messages[1].role, "assistant");
        assert_eq!(transcript.messages[1].text, "hi there");

        fs::remove_file(path).expect("transcript fixture should be removed");
    }
}
