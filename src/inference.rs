use anyhow::Result;
use reqwest::Client;
use serde::{Serialize, Deserialize};
use std::env;
use crate::tooler::Tooler;

#[derive(Debug, Deserialize)]
pub struct AnthropicResponse {
    pub content: Vec<ContentItem>,
    pub id: String,
    pub model: String,
    pub role: String,
    #[serde(rename = "type")]
    pub message_type: String,
    pub stop_reason: String,
    pub stop_sequence: Option<String>,
    pub usage: Usage,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(tag = "type")]
pub enum ContentItem {
    #[serde(rename = "text")]
    Text {
        text: String,
    },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: String,
        content: String,
    },
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Message {
    pub role: Role,
    pub content: Vec<ContentItem>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
}

// TODO possibly delete
impl PartialEq<&str> for Role {
    fn eq(&self, other: &&str) -> bool {
        match self {
            Role::User => other.eq_ignore_ascii_case(&"user"),
            Role::Assistant => other.eq_ignore_ascii_case(&"assistant"),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct TextContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct ToolUseContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub id: String,
    pub name: String,
    pub input: serde_json::Value,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct ToolResultContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub tool_use_id: String,
    // TODO this will change eventually to be String | Content
    pub content: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Usage {
    pub input_tokens: i32,
    pub cache_creation_input_tokens: i32,
    pub cache_read_input_tokens: i32,
    pub output_tokens: i32,
}

#[derive(Serialize)]
struct AnthropicRequest<'a> {
    model: &'a str,
    messages: Vec<Message>,
    max_tokens: u32,
    tools: serde_json::Value,
    system: String,
}

pub struct Inference {
    client: Client,
    tooler: Tooler,
}

impl Inference {
    pub fn new() -> Self {
        Inference {
            client: Client::new(),
            tooler: Tooler::new(),
        }
    }

    pub async fn query_anthropic(&self, messages: Vec<Message>, system_message: Option<&str>) -> Result<AnthropicResponse, anyhow::Error> {
        let api_key = env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY environment variable not set");
        let system = system_message.unwrap_or("").to_string();

        let tools = self.tooler.get_tools_json()?;

        let request = AnthropicRequest {
            model: "claude-3-5-sonnet-20241022",
            messages,
            max_tokens: 8096,
            tools,
            system,
        };

        //use std::fs::File;
        //use std::io::prelude::*;

        //let mut file = File::create(".log").unwrap();
        //let _ = write!(file, "{:#?}", response_text);

        let res = self.client
            .post("https://api.anthropic.com/v1/messages")
            .header("Content-Type", "application/json")
            .header("X-API-Key", api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&request)
            .send()
            .await?
            .json()
            .await?;

        Ok(res)
    }
}

