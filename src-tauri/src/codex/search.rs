use serde::{Deserialize, Serialize};

use super::error::CodexError;
use super::home::expand_tilde;
use super::threads::{list_threads, CodexThread};
use super::transcript::{read_transcript_messages, CodexTranscriptMessage};

const DEFAULT_SEARCH_RESULT_LIMIT: usize = 40;
const MAX_SEARCH_RESULT_LIMIT: usize = 100;
const SEARCH_MATCHES_PER_THREAD_LIMIT: usize = 4;
const SEARCH_QUERY_CHAR_LIMIT: usize = 200;
const SEARCH_SNIPPET_CONTEXT_CHARS: usize = 100;
const SEARCH_THREAD_SCAN_LIMIT: usize = 500;

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum CodexSearchScope {
    Active,
    Archived,
    All,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexSearchQuery {
    pub query: String,
    pub scope: CodexSearchScope,
    pub max_results: Option<usize>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexSearchResponse {
    pub query: String,
    pub scope: CodexSearchScope,
    pub scanned_thread_count: usize,
    pub matched_thread_count: usize,
    pub result_count: usize,
    pub truncated: bool,
    pub results: Vec<CodexSearchResult>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexSearchResult {
    pub thread_id: String,
    pub title: String,
    pub cwd: String,
    pub rollout_path: String,
    pub archived: bool,
    pub updated_at_ms: i64,
    pub transcript_truncated: bool,
    pub matches: Vec<CodexSearchMatch>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexSearchMatch {
    pub line_number: u64,
    pub timestamp: Option<String>,
    pub role: String,
    pub snippet: String,
}

pub fn search_history(query: CodexSearchQuery) -> Result<CodexSearchResponse, CodexError> {
    let trimmed_query = query.query.trim();
    let query_char_count = trimmed_query.chars().count();

    if query_char_count == 0 {
        return Err(CodexError::SearchOperation(
            "Search query cannot be empty.".to_string(),
        ));
    }

    if query_char_count > SEARCH_QUERY_CHAR_LIMIT {
        return Err(CodexError::SearchOperation(format!(
            "Search query is too long. Keep it under {SEARCH_QUERY_CHAR_LIMIT} characters."
        )));
    }

    let max_results = query
        .max_results
        .unwrap_or(DEFAULT_SEARCH_RESULT_LIMIT)
        .clamp(1, MAX_SEARCH_RESULT_LIMIT);
    let normalized_query = trimmed_query.to_lowercase();
    let mut scanned_thread_count = 0;
    let mut matched_thread_count = 0;
    let mut truncated = false;
    let mut results = Vec::new();

    for thread in list_threads()?.into_iter().filter(|thread| query.scope.includes(thread)) {
        if results.len() >= max_results || scanned_thread_count >= SEARCH_THREAD_SCAN_LIMIT {
            truncated = true;
            break;
        }

        let rollout_path = thread.rollout_path.trim();
        if rollout_path.is_empty() {
            continue;
        }

        scanned_thread_count += 1;
        let transcript_path = expand_tilde(rollout_path);
        let transcript = read_transcript_messages(&transcript_path)?;

        if transcript.truncated {
            truncated = true;
        }

        if !transcript.exists {
            continue;
        }

        let matches = transcript
            .messages
            .iter()
            .filter_map(|message| {
                message_search_match(message, &normalized_query, query_char_count)
            })
            .take(SEARCH_MATCHES_PER_THREAD_LIMIT)
            .collect::<Vec<_>>();

        if matches.is_empty() {
            continue;
        }

        matched_thread_count += 1;
        results.push(CodexSearchResult {
            thread_id: thread.id,
            title: display_title(thread.title),
            cwd: thread.cwd,
            rollout_path: thread.rollout_path,
            archived: thread.archived,
            updated_at_ms: thread.updated_at_ms,
            transcript_truncated: transcript.truncated,
            matches,
        });
    }

    Ok(CodexSearchResponse {
        query: trimmed_query.to_string(),
        scope: query.scope,
        scanned_thread_count,
        matched_thread_count,
        result_count: results.len(),
        truncated,
        results,
    })
}

impl CodexSearchScope {
    fn includes(self, thread: &CodexThread) -> bool {
        match self {
            Self::Active => !thread.archived,
            Self::Archived => thread.archived,
            Self::All => true,
        }
    }
}

fn message_search_match(
    message: &CodexTranscriptMessage,
    normalized_query: &str,
    query_char_count: usize,
) -> Option<CodexSearchMatch> {
    let normalized_text = message.text.to_lowercase();
    let byte_index = normalized_text.find(normalized_query)?;
    let match_start = normalized_text[..byte_index].chars().count();

    Some(CodexSearchMatch {
        line_number: message.line_number,
        timestamp: message.timestamp.clone(),
        role: message.role.clone(),
        snippet: snippet_around_match(&message.text, match_start, query_char_count),
    })
}

fn snippet_around_match(text: &str, match_start: usize, match_len: usize) -> String {
    let chars = text.chars().collect::<Vec<_>>();
    let start = match_start.saturating_sub(SEARCH_SNIPPET_CONTEXT_CHARS);
    let end = (match_start + match_len + SEARCH_SNIPPET_CONTEXT_CHARS).min(chars.len());
    let snippet = chars[start..end].iter().collect::<String>();
    let mut normalized_snippet = snippet.split_whitespace().collect::<Vec<_>>().join(" ");

    if start > 0 {
        normalized_snippet = format!("...{normalized_snippet}");
    }

    if end < chars.len() {
        normalized_snippet.push_str("...");
    }

    normalized_snippet
}

fn display_title(title: String) -> String {
    let trimmed = title.trim();
    if trimmed.is_empty() {
        "Untitled session".to_string()
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snippet_collapses_context_around_match() {
        let snippet = snippet_around_match("alpha\nBeta gamma delta", 6, 4);

        assert_eq!(snippet, "alpha Beta gamma delta");
    }
}
