use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Represents a parsed event from a JSONL transcript line.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum TranscriptEvent {
    ToolUse {
        agent_id: String,
        session_id: String,
        tool_name: String,
        tool_id: String,
        timestamp: String,
    },
    ToolResult {
        agent_id: String,
        session_id: String,
        tool_id: String,
        timestamp: String,
    },
    SubAgentSpawned {
        parent_agent_id: String,
        session_id: String,
        description: String,
        timestamp: String,
    },
    SessionActive {
        agent_id: String,
        session_id: String,
        timestamp: String,
    },
    /// Emitted by the watcher when a sub-agent JSONL file first appears.
    SubAgentStarted {
        agent_id: String,
        parent_agent_id: String,
        timestamp: String,
    },
}

/// Raw JSONL record — we only deserialize the fields we care about.
#[derive(Debug, Deserialize)]
struct RawRecord {
    #[serde(default)]
    r#type: String,
    #[serde(rename = "sessionId", default)]
    session_id: String,
    #[serde(rename = "agentId", default)]
    agent_id: Option<String>,
    #[serde(default)]
    message: Option<RawMessage>,
    #[serde(default)]
    timestamp: String,
}

#[derive(Debug, Deserialize)]
struct RawMessage {
    #[serde(default)]
    role: String,
    #[serde(default)]
    content: Value,
}

/// Classifies a tool as either reading or writing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolKind {
    Reading,
    Writing,
}

pub fn classify_tool(name: &str) -> ToolKind {
    match name {
        "Read" | "Grep" | "Glob" | "WebFetch" | "WebSearch" | "ToolSearch"
        | "TaskList" | "TaskGet" | "LSP" => ToolKind::Reading,
        _ => ToolKind::Writing,
    }
}

/// Parse a single JSONL line into zero or more transcript events.
pub fn parse_line(line: &str, file_agent_id: &str) -> Vec<TranscriptEvent> {
    let mut events = Vec::new();

    let record: RawRecord = match serde_json::from_str(line) {
        Ok(r) => r,
        Err(_) => return events,
    };

    let agent_id = record
        .agent_id
        .clone()
        .unwrap_or_else(|| file_agent_id.to_string());
    let session_id = record.session_id.clone();
    let timestamp = record.timestamp.clone();

    // Every record means the session is active
    if !session_id.is_empty() {
        events.push(TranscriptEvent::SessionActive {
            agent_id: agent_id.clone(),
            session_id: session_id.clone(),
            timestamp: timestamp.clone(),
        });
    }

    // Only look at assistant messages for tool use
    if record.r#type != "assistant" {
        return events;
    }

    let message = match &record.message {
        Some(m) if m.role == "assistant" => m,
        _ => return events,
    };

    // Content can be an array of content blocks
    let content_array = match &message.content {
        Value::Array(arr) => arr,
        _ => return events,
    };

    for block in content_array {
        let block_type = block.get("type").and_then(|v| v.as_str()).unwrap_or("");

        if block_type == "tool_use" {
            let tool_name = block
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let tool_id = block
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            // Check if this is an Agent tool call (sub-agent spawn)
            if tool_name == "Agent" {
                let description = block
                    .get("input")
                    .and_then(|v| v.get("description"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("sub-agent")
                    .to_string();

                events.push(TranscriptEvent::SubAgentSpawned {
                    parent_agent_id: agent_id.clone(),
                    session_id: session_id.clone(),
                    description,
                    timestamp: timestamp.clone(),
                });
            }

            events.push(TranscriptEvent::ToolUse {
                agent_id: agent_id.clone(),
                session_id: session_id.clone(),
                tool_name,
                tool_id,
                timestamp: timestamp.clone(),
            });
        }

        if block_type == "tool_result" {
            let tool_id = block
                .get("tool_use_id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            events.push(TranscriptEvent::ToolResult {
                agent_id: agent_id.clone(),
                session_id: session_id.clone(),
                tool_id,
                timestamp: timestamp.clone(),
            });
        }
    }

    events
}
