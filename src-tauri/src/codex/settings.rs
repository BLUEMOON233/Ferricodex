use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use toml_edit::{value as toml_value, DocumentMut, Item, Table};

use super::error::CodexError;
use super::home::resolve_codex_home;

const CONFIG_FILE_NAME: &str = "config.toml";
const AUTH_FILE_NAME: &str = "auth.json";
const PROVIDERS_TABLE: &str = "model_providers";
const AUTH_API_KEY_NAME: &str = "OPENAI_API_KEY";
const AUTH_API_KEY_ALIASES: &[&str] = &["OPENAI_API_KEY", "api_key", "openai_api_key"];
const MAX_PROVIDER_COUNT: usize = 32;
const MAX_FIELD_LEN: usize = 512;
const MAX_API_KEY_LEN: usize = 4096;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexProviderSettings {
    pub path: String,
    pub exists: bool,
    pub revision: String,
    pub model: String,
    pub model_provider: String,
    pub providers: Vec<CodexProviderConfig>,
    pub has_secret_fields: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexAuthStatus {
    pub path: String,
    pub exists: bool,
    pub revision: String,
    pub has_api_key: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexProviderConfig {
    pub id: String,
    pub name: String,
    pub base_url: String,
    pub wire_api: String,
    pub env_key: String,
    pub requires_openai_auth: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexProviderSettingsUpdate {
    pub revision: String,
    pub model: String,
    pub model_provider: String,
    pub providers: Vec<CodexProviderConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexApiKeyUpdate {
    pub revision: String,
    pub api_key: String,
}

pub fn provider_settings() -> Result<CodexProviderSettings, CodexError> {
    let path = config_path()?;
    read_provider_settings_at(&path)
}

pub fn save_provider_settings(
    input: CodexProviderSettingsUpdate,
) -> Result<CodexProviderSettings, CodexError> {
    validate_update(&input)?;

    let path = config_path()?;
    let original = read_config_text(&path)?;
    let current_revision = revision_for(&original);

    if input.revision != current_revision {
        return Err(CodexError::SettingsOperation(
            "Codex config.toml changed outside Ferricodex. Reload it before saving.".to_string(),
        ));
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| {
            CodexError::SettingsOperation(format!(
                "Could not create Codex home at {}: {source}",
                parent.display()
            ))
        })?;
    }

    if path.exists() {
        backup_config(&path, &original)?;
    }

    let mut document = parse_config_document(&original)?;
    apply_provider_update(&mut document, input);

    fs::write(&path, document.to_string()).map_err(|source| {
        CodexError::SettingsOperation(format!(
            "Could not write Codex config at {}: {source}",
            path.display()
        ))
    })?;

    read_provider_settings_at(&path)
}

pub fn auth_status() -> Result<CodexAuthStatus, CodexError> {
    let path = auth_path()?;
    read_auth_status_at(&path)
}

pub fn update_api_key(input: CodexApiKeyUpdate) -> Result<CodexAuthStatus, CodexError> {
    let api_key = validate_api_key(&input.api_key)?;

    let path = auth_path()?;
    let original = read_auth_text(&path)?;
    let current_revision = revision_for(&original);

    if input.revision != current_revision {
        return Err(CodexError::SettingsOperation(
            "Codex auth.json changed outside Ferricodex. Reload it before saving.".to_string(),
        ));
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| {
            CodexError::SettingsOperation(format!(
                "Could not create Codex home at {}: {source}",
                parent.display()
            ))
        })?;
    }

    if path.exists() {
        backup_auth(&path, &original)?;
    }

    let next = upsert_api_key_json(&original, api_key)?;
    fs::write(&path, next).map_err(|source| {
        CodexError::SettingsOperation(format!(
            "Could not write Codex auth file at {}: {source}",
            path.display()
        ))
    })?;
    set_secret_file_permissions(&path)?;

    read_auth_status_at(&path)
}

fn config_path() -> Result<PathBuf, CodexError> {
    Ok(resolve_codex_home()?.path.join(CONFIG_FILE_NAME))
}

fn auth_path() -> Result<PathBuf, CodexError> {
    Ok(resolve_codex_home()?.path.join(AUTH_FILE_NAME))
}

fn read_provider_settings_at(path: &Path) -> Result<CodexProviderSettings, CodexError> {
    let contents = read_config_text(path)?;
    let exists = path.exists();
    let document = parse_config_document(&contents)?;

    Ok(CodexProviderSettings {
        path: path.to_string_lossy().into_owned(),
        exists,
        revision: revision_for(&contents),
        model: string_field(document.get("model")),
        model_provider: string_field(document.get("model_provider")),
        providers: provider_configs(&document),
        has_secret_fields: has_secret_fields(document.as_table()),
    })
}

fn read_config_text(path: &Path) -> Result<String, CodexError> {
    match fs::read_to_string(path) {
        Ok(contents) => Ok(contents),
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(source) => Err(CodexError::SettingsOperation(format!(
            "Could not read Codex config at {}: {source}",
            path.display()
        ))),
    }
}

fn read_auth_status_at(path: &Path) -> Result<CodexAuthStatus, CodexError> {
    let contents = read_auth_text(path)?;

    Ok(CodexAuthStatus {
        path: path.to_string_lossy().into_owned(),
        exists: path.exists(),
        revision: revision_for(&contents),
        has_api_key: auth_has_api_key(&contents)?,
    })
}

fn read_auth_text(path: &Path) -> Result<String, CodexError> {
    match fs::read_to_string(path) {
        Ok(contents) => Ok(contents),
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(source) => Err(CodexError::SettingsOperation(format!(
            "Could not read Codex auth file at {}: {source}",
            path.display()
        ))),
    }
}

fn parse_config_document(contents: &str) -> Result<DocumentMut, CodexError> {
    if contents.trim().is_empty() {
        return Ok(DocumentMut::new());
    }

    contents.parse::<DocumentMut>().map_err(|source| {
        CodexError::SettingsOperation(format!("Could not parse Codex config.toml: {source}"))
    })
}

fn revision_for(contents: &str) -> String {
    let mut hasher = DefaultHasher::new();
    contents.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn provider_configs(document: &DocumentMut) -> Vec<CodexProviderConfig> {
    let Some(table) = document.get(PROVIDERS_TABLE).and_then(Item::as_table) else {
        return Vec::new();
    };

    let mut providers = table
        .iter()
        .filter_map(|(id, item)| {
            let provider_table = item.as_table()?;
            Some(CodexProviderConfig {
                id: id.to_string(),
                name: string_field(provider_table.get("name")),
                base_url: string_field(provider_table.get("base_url")),
                wire_api: string_field(provider_table.get("wire_api")),
                env_key: string_field(provider_table.get("env_key")),
                requires_openai_auth: bool_field(provider_table.get("requires_openai_auth")),
            })
        })
        .collect::<Vec<_>>();

    providers.sort_by(|left, right| left.id.cmp(&right.id));
    providers
}

fn string_field(item: Option<&Item>) -> String {
    item.and_then(Item::as_str).unwrap_or_default().to_string()
}

fn bool_field(item: Option<&Item>) -> Option<bool> {
    item.and_then(Item::as_bool)
}

fn has_secret_fields(table: &Table) -> bool {
    table.iter().any(|(key, item)| {
        key_is_secret_like(key) || item.as_table().is_some_and(has_secret_fields)
    })
}

fn key_is_secret_like(key: &str) -> bool {
    let key = key.to_lowercase();

    key.contains("api_key")
        || key.contains("apikey")
        || key.contains("token")
        || key.contains("secret")
        || key.contains("password")
        || key.contains("credential")
        || key.contains("authorization")
}

fn validate_update(input: &CodexProviderSettingsUpdate) -> Result<(), CodexError> {
    validate_field("model", &input.model)?;
    validate_field("model_provider", &input.model_provider)?;

    if input.providers.len() > MAX_PROVIDER_COUNT {
        return Err(CodexError::SettingsOperation(format!(
            "Provider count is limited to {MAX_PROVIDER_COUNT}."
        )));
    }

    let mut ids = std::collections::BTreeSet::new();
    for provider in &input.providers {
        validate_provider_id(&provider.id)?;
        validate_field("provider.name", &provider.name)?;
        validate_field("provider.base_url", &provider.base_url)?;
        validate_field("provider.wire_api", &provider.wire_api)?;
        validate_field("provider.env_key", &provider.env_key)?;

        if !ids.insert(provider.id.trim().to_string()) {
            return Err(CodexError::SettingsOperation(format!(
                "Duplicate provider id: {}",
                provider.id
            )));
        }
    }

    Ok(())
}

fn validate_provider_id(id: &str) -> Result<(), CodexError> {
    let id = id.trim();
    if id.is_empty() {
        return Err(CodexError::SettingsOperation(
            "Provider id cannot be empty.".to_string(),
        ));
    }

    if id.len() > 80 {
        return Err(CodexError::SettingsOperation(
            "Provider id is too long.".to_string(),
        ));
    }

    if !id
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-'))
    {
        return Err(CodexError::SettingsOperation(
            "Provider id can only contain letters, numbers, underscores, and hyphens."
                .to_string(),
        ));
    }

    Ok(())
}

fn validate_field(name: &str, value: &str) -> Result<(), CodexError> {
    if value.len() > MAX_FIELD_LEN {
        return Err(CodexError::SettingsOperation(format!(
            "{name} is too long. Maximum length is {MAX_FIELD_LEN} characters."
        )));
    }

    Ok(())
}

fn backup_config(path: &Path, contents: &str) -> Result<(), CodexError> {
    backup_file(path, CONFIG_FILE_NAME, contents).map(|_| ())
}

fn backup_auth(path: &Path, contents: &str) -> Result<(), CodexError> {
    let backup_path = backup_file(path, AUTH_FILE_NAME, contents)?;
    set_secret_file_permissions(&backup_path)
}

fn backup_file(path: &Path, file_name: &str, contents: &str) -> Result<PathBuf, CodexError> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|source| {
            CodexError::SettingsOperation(format!("Could not create backup timestamp: {source}"))
        })?
        .as_secs();
    let backup_path = path.with_file_name(format!("{file_name}.ferricodex-{timestamp}.bak"));

    fs::write(&backup_path, contents).map_err(|source| {
        CodexError::SettingsOperation(format!(
            "Could not write Codex backup at {}: {source}",
            backup_path.display()
        ))
    })?;

    Ok(backup_path)
}

fn set_secret_file_permissions(path: &Path) -> Result<(), CodexError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(path, fs::Permissions::from_mode(0o600)).map_err(|source| {
            CodexError::SettingsOperation(format!(
                "Could not restrict permissions for {}: {source}",
                path.display()
            ))
        })?;
    }

    Ok(())
}

fn validate_api_key(value: &str) -> Result<&str, CodexError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(CodexError::SettingsOperation(
            "API key cannot be empty.".to_string(),
        ));
    }

    if trimmed.len() > MAX_API_KEY_LEN {
        return Err(CodexError::SettingsOperation(format!(
            "API key is too long. Maximum length is {MAX_API_KEY_LEN} characters."
        )));
    }

    if trimmed.chars().any(char::is_control) {
        return Err(CodexError::SettingsOperation(
            "API key cannot contain control characters.".to_string(),
        ));
    }

    Ok(trimmed)
}

fn auth_has_api_key(contents: &str) -> Result<bool, CodexError> {
    let value = parse_auth_json(contents)?;
    let Some(object) = value.as_object() else {
        return Err(CodexError::SettingsOperation(
            "Codex auth.json must be a JSON object.".to_string(),
        ));
    };

    Ok(object
        .iter()
        .any(|(key, value)| is_api_key_name(key) && json_string_is_present(value)))
}

fn upsert_api_key_json(contents: &str, api_key: &str) -> Result<String, CodexError> {
    let value = parse_auth_json(contents)?;
    let JsonValue::Object(mut object) = value else {
        return Err(CodexError::SettingsOperation(
            "Codex auth.json must be a JSON object.".to_string(),
        ));
    };

    let key = object
        .keys()
        .find(|key| is_api_key_name(key))
        .cloned()
        .unwrap_or_else(|| AUTH_API_KEY_NAME.to_string());

    object.insert(key, JsonValue::String(api_key.to_string()));
    json_to_pretty_text(JsonValue::Object(object))
}

fn parse_auth_json(contents: &str) -> Result<JsonValue, CodexError> {
    if contents.trim().is_empty() {
        return Ok(JsonValue::Object(JsonMap::new()));
    }

    serde_json::from_str::<JsonValue>(contents).map_err(|source| {
        CodexError::SettingsOperation(format!("Could not parse Codex auth.json: {source}"))
    })
}

fn json_to_pretty_text(value: JsonValue) -> Result<String, CodexError> {
    let mut output = serde_json::to_string_pretty(&value).map_err(|source| {
        CodexError::SettingsOperation(format!("Could not serialize Codex auth.json: {source}"))
    })?;
    output.push('\n');
    Ok(output)
}

fn json_string_is_present(value: &JsonValue) -> bool {
    value.as_str().is_some_and(|value| !value.trim().is_empty())
}

fn is_api_key_name(key: &str) -> bool {
    AUTH_API_KEY_ALIASES
        .iter()
        .any(|alias| key.eq_ignore_ascii_case(alias))
}

fn apply_provider_update(document: &mut DocumentMut, input: CodexProviderSettingsUpdate) {
    set_or_remove_string(document.as_table_mut(), "model", input.model.trim());
    set_or_remove_string(
        document.as_table_mut(),
        "model_provider",
        input.model_provider.trim(),
    );

    if !document.as_table().contains_key(PROVIDERS_TABLE) {
        document[PROVIDERS_TABLE] = Item::Table(Table::new());
    }

    for provider in input.providers {
        let provider_id = provider.id.trim();
        if document[PROVIDERS_TABLE][provider_id].is_none() {
            document[PROVIDERS_TABLE][provider_id] = Item::Table(Table::new());
        }

        if let Some(table) = document[PROVIDERS_TABLE][provider_id].as_table_mut() {
            set_or_remove_string(table, "name", provider.name.trim());
            set_or_remove_string(table, "base_url", provider.base_url.trim());
            set_or_remove_string(table, "wire_api", provider.wire_api.trim());
            set_or_remove_string(table, "env_key", provider.env_key.trim());

            match provider.requires_openai_auth {
                Some(value) => table["requires_openai_auth"] = toml_value(value),
                None => {
                    table.remove("requires_openai_auth");
                }
            }
        }
    }
}

fn set_or_remove_string(table: &mut Table, key: &str, field_value: &str) {
    if field_value.is_empty() {
        table.remove(key);
    } else {
        table[key] = toml_value(field_value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_provider_settings() {
        let document = r#"
model = "gpt-5.5"
model_provider = "bluemoon"

[model_providers.bluemoon]
base_url = "https://api.example.test/v1"
name = "Blue Moon"
requires_openai_auth = true
wire_api = "responses"
"#
        .parse::<DocumentMut>()
        .expect("valid toml");

        let providers = provider_configs(&document);

        assert_eq!(providers.len(), 1);
        assert_eq!(providers[0].id, "bluemoon");
        assert_eq!(providers[0].base_url, "https://api.example.test/v1");
        assert_eq!(providers[0].requires_openai_auth, Some(true));
    }

    #[test]
    fn updates_known_fields_and_keeps_unknown_fields() {
        let mut document = r#"
approval_policy = "on-request"
model = "old-model"

[model_providers.bluemoon]
base_url = "https://old.example.test/v1"
unknown_field = "keep me"
"#
        .parse::<DocumentMut>()
        .expect("valid toml");

        apply_provider_update(
            &mut document,
            CodexProviderSettingsUpdate {
                revision: String::new(),
                model: "new-model".to_string(),
                model_provider: "bluemoon".to_string(),
                providers: vec![CodexProviderConfig {
                    id: "bluemoon".to_string(),
                    name: "Blue Moon".to_string(),
                    base_url: "https://new.example.test/v1".to_string(),
                    wire_api: "responses".to_string(),
                    env_key: String::new(),
                    requires_openai_auth: Some(false),
                }],
            },
        );

        assert_eq!(document["approval_policy"].as_str(), Some("on-request"));
        assert_eq!(document["model"].as_str(), Some("new-model"));
        assert_eq!(document["model_provider"].as_str(), Some("bluemoon"));
        assert_eq!(
            document["model_providers"]["bluemoon"]["unknown_field"].as_str(),
            Some("keep me")
        );
        assert_eq!(
            document["model_providers"]["bluemoon"]["requires_openai_auth"].as_bool(),
            Some(false)
        );
    }

    #[test]
    fn detects_configured_api_key_without_exposing_value() {
        assert!(auth_has_api_key(r#"{"OPENAI_API_KEY":"sk-live"}"#).expect("valid json"));
        assert!(auth_has_api_key(r#"{"api_key":"sk-test"}"#).expect("valid json"));
        assert!(auth_has_api_key(r#"{"openai_api_key":"sk-test"}"#).expect("valid json"));
        assert!(!auth_has_api_key(r#"{"OPENAI_API_KEY":""}"#).expect("valid json"));
        assert!(!auth_has_api_key(r#"{"OPENAI_API_KEY":null}"#).expect("valid json"));
        assert!(!auth_has_api_key(r#"{"auth_mode":"apikey"}"#).expect("valid json"));
        assert!(auth_has_api_key("[]").is_err());
    }

    #[test]
    fn inserts_api_key_when_auth_file_is_empty() {
        let next = upsert_api_key_json("", "sk-test").expect("json updated");
        let value = serde_json::from_str::<JsonValue>(&next).expect("valid json");

        assert_eq!(value[AUTH_API_KEY_NAME].as_str(), Some("sk-test"));
    }

    #[test]
    fn updates_existing_api_key_and_preserves_other_lines() {
        let next = upsert_api_key_json(
            r#"{"auth_mode":"apikey","OPENAI_API_KEY":"old","other":"keep"}"#,
            "new\"key\\value",
        )
        .expect("json updated");
        let value = serde_json::from_str::<JsonValue>(&next).expect("valid json");

        assert_eq!(value["OPENAI_API_KEY"].as_str(), Some("new\"key\\value"));
        assert_eq!(value["auth_mode"].as_str(), Some("apikey"));
        assert_eq!(value["other"].as_str(), Some("keep"));
    }

    #[test]
    fn updates_existing_api_key_alias_without_adding_default_key() {
        let next = upsert_api_key_json(r#"{"api_key":"old","auth_mode":"apikey"}"#, "new")
            .expect("json updated");
        let value = serde_json::from_str::<JsonValue>(&next).expect("valid json");

        assert_eq!(value["api_key"].as_str(), Some("new"));
        assert!(value.get(AUTH_API_KEY_NAME).is_none());
    }

    #[test]
    fn validates_api_key_input() {
        assert_eq!(validate_api_key("  sk-test  ").expect("valid key"), "sk-test");
        assert!(validate_api_key("").is_err());
        assert!(validate_api_key("sk\nkey").is_err());
    }
}
