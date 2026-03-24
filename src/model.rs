use serde::{Deserialize, Deserializer, Serialize};
use typed_builder::TypedBuilder;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    Assistant,
    User,
}

// ==================== Request Body ====================
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatRequest {
    #[serde(deserialize_with = "deserialize_model")]
    pub model: String,
    #[serde(deserialize_with = "deserialize_message")]
    pub messages: Vec<Message>,
    #[serde(
        rename = "reasoningEffort",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub reasoning_effort: Option<String>,
    #[serde(skip_serializing, default)]
    pub stream: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Default, TypedBuilder)]
pub struct Message {
    #[builder(default, setter(into))]
    pub role: Option<Role>,
    #[builder(default, setter(into))]
    pub content: Option<Content>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Content {
    Text(String),
    Vec(Vec<ContentItem>),
}

/// Unified content item supporting text and two image formats:
///
/// - `{"type":"text","text":"..."}`
/// - `{"type":"image","mimeType":"image/webp","image":"data:image/webp;base64,..."}`
/// - `{"type":"image_url","image_url":{"url":"data:image/jpeg;base64,..."}}`
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentItem {
    #[serde(rename = "text")]
    Text { text: String },

    #[serde(rename = "image")]
    Image {
        #[serde(rename = "mimeType")]
        mime_type: String,
        image: String,
    },

    #[serde(rename = "image_url")]
    ImageUrl { image_url: ImageUrlContent },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageUrlContent {
    pub url: String,
}

// ── deserializers ─────────────────────────────────────────────────────────────

fn deserialize_model<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let model = String::deserialize(deserializer)?;
    let model = match model.as_str() {
        "gpt-5-mini" => "gpt-5-mini",
        "gpt-4o-mini" => "gpt-4o-mini",
        "gpt-oss-120b" => "openai/gpt-oss-120b",
        "llama-4-scout" => "meta-llama/Llama-4-Scout-17B-16E-Instruct",
        "claude-haiku-4-5" => "claude-haiku-4-5",
        "mixtral-small-3" => "mistralai/Mistral-Small-24B-Instruct-2501",
        _ => model.as_str(),
    };
    Ok(model.to_owned())
}

fn deserialize_message<'de, D>(deserializer: D) -> Result<Vec<Message>, D::Error>
where
    D: Deserializer<'de>,
{
    let mut messages: Vec<Message> = Vec::deserialize(deserializer)?;
    for msg in &mut messages {
        if let Some(role) = msg.role.as_mut() {
            if matches!(role, Role::System) {
                *role = Role::User;
            }
        }
    }
    Ok(messages)
}

// ==================== Duck API Response Body ====================
#[derive(Deserialize)]
pub struct DuckChatCompletion {
    pub message: Option<String>,
    pub created: u64,
    #[serde(default = "default_id")]
    pub id: String,
    pub model: Option<String>,
}

fn default_id() -> String {
    "chatcmpl-123".to_owned()
}

// ==================== Response Body ====================
#[derive(Serialize, TypedBuilder)]
pub struct ChatCompletion<'a> {
    #[builder(default, setter(into))]
    #[serde(default = "default_id")]
    id: Option<String>,

    object: &'static str,

    #[builder(default, setter(into))]
    created: Option<u64>,

    model: &'a str,

    choices: Vec<Choice>,

    #[builder(default, setter(into))]
    usage: Option<Usage>,
}

#[derive(Serialize, TypedBuilder)]
pub struct Choice {
    index: usize,

    #[builder(default, setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<Message>,

    #[builder(default, setter(into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    delta: Option<Message>,

    #[builder(setter(into))]
    logprobs: Option<String>,

    #[builder(setter(into))]
    finish_reason: Option<&'static str>,
}

#[derive(Serialize, TypedBuilder)]
pub struct Usage {
    prompt_tokens: i32,
    completion_tokens: i32,
    total_tokens: i32,
}
