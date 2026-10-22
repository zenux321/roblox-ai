//! Obiectele trimise interfeței (JSON camelCase), oglindă a `src/domain/types.ts`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationSummary {
    pub id: String,
    pub title: String,
    pub visibility: String,
    pub scope: String,
    pub last_activity: String,
    pub is_new: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub meta: String,
    pub analysis: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alternative: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preview: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeSnippet {
    pub language: String,
    pub file: String,
    pub code: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum AssistantExtra {
    PreparePlan,
    PlanCard {
        title: String,
        summary: String,
        approver: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        plan_id: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        plan_version: Option<u32>,
    },
    ImageGenerationUnconfigured,
    AssociateProject,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum Message {
    Person {
        id: String,
        author_id: String,
        time: String,
        text: String,
        attachments: Vec<Attachment>,
    },
    Agent {
        id: String,
        agent: String,
        title: String,
        computer: String,
        details: Vec<String>,
    },
    Assistant {
        id: String,
        time: String,
        model: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tag: Option<String>,
        paragraphs: Vec<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        list: Option<Vec<String>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        code: Option<CodeSnippet>,
        extras: Vec<AssistantExtra>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        reply_to: Option<String>,
        attempt: u32,
        stopped: bool,
    },
}

impl Message {
    pub fn id(&self) -> &str {
        match self {
            Message::Person { id, .. }
            | Message::Agent { id, .. }
            | Message::Assistant { id, .. } => id,
        }
    }

    pub fn kind(&self) -> &'static str {
        match self {
            Message::Person { .. } => "person",
            Message::Agent { .. } => "agent",
            Message::Assistant { .. } => "assistant",
        }
    }

    /// Textul folosit de căutare.
    pub fn search_text(&self) -> String {
        match self {
            Message::Person { text, .. } => text.clone(),
            Message::Agent { agent, title, .. } => format!("{agent} {title}"),
            Message::Assistant {
                paragraphs, list, ..
            } => {
                let mut parts = paragraphs.clone();
                parts.extend(list.clone().unwrap_or_default());
                parts.join(" ")
            }
        }
    }
}

/// Evenimentele trimise pe canalul unui răspuns în curs.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ReplyEvent {
    Thinking,
    Streaming { text: String },
    Done { message: Box<Message> },
    Error { message: String },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanTaskView {
    pub id: String,
    pub number: usize,
    pub title: String,
    pub description: String,
    pub agent: String,
    pub location: String,
    pub zone: String,
    pub after: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanTestView {
    pub id: String,
    pub title: String,
    /// `static` = verificare automată în această etapă; `studio` = cere Roblox Studio.
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionRow {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalView {
    pub approved_by: String,
    pub approved_at: String,
    pub identity_kind: String,
    pub plan_sha256: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revoked_at: Option<String>,
    pub valid: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanView {
    pub id: String,
    pub version: u32,
    pub status: String,
    pub title: String,
    pub objective: String,
    pub sha256: String,
    pub estimated_duration: String,
    pub cost_estimate_usd: f64,
    pub cost_limit_usd: f64,
    pub max_retries: u32,
    pub provider_label: String,
    pub tasks: Vec<PlanTaskView>,
    pub tests: Vec<PlanTestView>,
    pub permissions: Vec<PermissionRow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approval: Option<ApprovalView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_run_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunTaskView {
    pub id: String,
    pub title: String,
    pub state: String,
    pub attempt: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunView {
    pub id: String,
    pub plan_id: String,
    pub plan_version: u32,
    pub status: String,
    pub tasks: Vec<RunTaskView>,
    pub done: usize,
    pub total: usize,
    pub activity: String,
    pub started_at: String,
    pub updated_at: String,
    pub last_seq: i64,
    pub live: bool,
    pub has_results: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunEventView {
    pub seq: i64,
    pub at: String,
    pub kind: String,
    pub level: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestResult {
    pub id: String,
    pub title: String,
    /// `passed` | `failed` | `not_run`
    pub status: String,
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffLine {
    /// `add` | `del` | `ctx`
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub number: Option<usize>,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileDiff {
    pub path: String,
    pub added: usize,
    pub removed: usize,
    pub is_new: bool,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunResults {
    /// `passed` | `needs_review` | `failed` | `cancelled`
    pub verdict: String,
    pub tests: Vec<TestResult>,
    pub limitations: Vec<String>,
    pub duration_seconds: i64,
    pub cost_usd: f64,
    pub report_path: Option<String>,
    pub changes: Vec<FileDiff>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceInfo {
    pub project_id: String,
    pub project_name: String,
    pub project_root: String,
    pub data_dir: String,
    pub identity: String,
}
