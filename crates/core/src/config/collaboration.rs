//! Contractul `collaboration` v0.2 (vezi `config/collaboration.example.json`).
//!
//! Host/join, execuția remote și Blender nu sunt implementate încă: în modul
//! `Execution` activarea lor este respinsă cu `FeatureNotAvailable`.

use std::net::IpAddr;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{ConfigError, Issue, IssueCode, Issues, Mode, collect_placeholders};
use crate::ids::ToolId;

pub const SCHEMA_VERSION: &str = "0.2";
/// Codul de asociere expiră în cel mult 5 minute (COLLABORATION.md).
pub const MAX_PAIRING_TTL_SECONDS: u32 = 300;
pub const MAX_FILE_BYTES: u64 = 200 * 1024 * 1024;
pub const MAX_SHARED_PROJECT_BYTES: u64 = 2 * 1024 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CollaborationConfig {
    pub schema_version: String,
    pub example_only: bool,
    pub collaboration: CollaborationSection,
    pub device_worker: DeviceWorkerSection,
    pub chat: ChatSection,
    pub attachments: AttachmentsSection,
    pub blender_mcp: BlenderMcpSection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CollaborationMode {
    Solo,
    Host,
    Participant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CollaborationTransport {
    HttpsAndWebsocketTls,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ConflictPolicy {
    PreserveBothRequireResolution,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OfflineMutations {
    LocalProposalsOnly,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CollaborationSection {
    pub enabled: bool,
    pub mode: CollaborationMode,
    pub transport: CollaborationTransport,
    pub host_bind_ip: Option<String>,
    pub host_port: Option<u16>,
    pub allow_wildcard_bind: bool,
    pub require_host_fingerprint_verification: bool,
    pub require_host_acceptance: bool,
    pub pairing_code_ttl_seconds: u32,
    pub pairing_max_attempts: u32,
    pub shared_project_roots: Vec<String>,
    pub shared_vault_paths: Vec<String>,
    pub auto_host_failover: bool,
    pub conflict_policy: ConflictPolicy,
    pub offline_mutations: OfflineMutations,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DeviceWorkerSection {
    pub accept_remote_tasks: bool,
    pub allowed_project_ids: Vec<String>,
    pub allowed_tools: Vec<ToolId>,
    pub max_concurrent_tasks: u32,
    pub require_local_consent: bool,
    pub require_plan_hash: bool,
    pub require_lease_and_fencing_token: bool,
    pub max_api_run_usd: f64,
    pub expose_local_mcp_or_ollama: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Visibility {
    Private,
    Shared,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum UnsupportedAnalysisAction {
    ExplainNoPaidFallback,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ChatSection {
    pub default_visibility: Visibility,
    pub general_conversations: bool,
    pub project_conversations: bool,
    pub require_explicit_share: bool,
    pub sanitize_markdown: bool,
    pub unsupported_analysis_action: UnsupportedAnalysisAction,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AttachmentsSection {
    pub max_file_bytes: u64,
    pub max_shared_project_bytes: u64,
    pub min_free_disk_bytes: u64,
    pub chunk_bytes: u64,
    pub resume_transfers: bool,
    pub verify_content_hash: bool,
    pub execute_attachments: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BlenderMcpSection {
    pub enabled: bool,
    pub server_source: Option<String>,
    pub pinned_version: Option<String>,
    pub transport: Option<String>,
    pub device_id: Option<String>,
    pub instance_id: Option<String>,
    pub require_adapter_security_review: bool,
    pub allow_arbitrary_python: bool,
    pub require_scene_copy: bool,
    pub require_export_validation: bool,
}

impl CollaborationConfig {
    pub fn parse(json: &str) -> Result<Self, ConfigError> {
        serde_json::from_str(json).map_err(|e| ConfigError::Parse(e.to_string()))
    }

    pub fn load(json: &str, mode: Mode) -> Result<Self, ConfigError> {
        let config = Self::parse(json)?;
        let issues = config.validate(mode);
        if issues.is_empty() {
            Ok(config)
        } else {
            Err(ConfigError::Invalid(issues))
        }
    }

    pub fn validate(&self, mode: Mode) -> Vec<Issue> {
        let mut is = Issues::default();
        is.require(
            self.schema_version == SCHEMA_VERSION,
            "$.schema_version",
            IssueCode::UnsupportedSchemaVersion,
            "versiune de schemă nesuportată (așteptat 0.2)",
        );
        self.validate_collaboration(&mut is, mode);
        self.validate_device_worker(&mut is, mode);
        self.validate_chat(&mut is);
        self.validate_attachments(&mut is);
        self.validate_blender(&mut is, mode);
        if mode == Mode::Execution {
            is.require(
                !self.example_only,
                "$.example_only",
                IssueCode::ExampleOnly,
                "configurația exemplu nu poate fi folosită la execuție",
            );
            collect_placeholders(self, &mut is);
        }
        is.0
    }

    fn validate_collaboration(&self, is: &mut Issues, mode: Mode) {
        let c = &self.collaboration;
        let safe = [
            (
                !c.allow_wildcard_bind,
                "allow_wildcard_bind",
                "bind pe toate interfețele este interzis",
            ),
            (
                c.require_host_fingerprint_verification,
                "require_host_fingerprint_verification",
                "fingerprintul gazdei se verifică obligatoriu",
            ),
            (
                c.require_host_acceptance,
                "require_host_acceptance",
                "gazda acceptă explicit fiecare cerere",
            ),
            (
                !c.auto_host_failover,
                "auto_host_failover",
                "nu există failover automat de gazdă",
            ),
        ];
        for (ok, name, message) in safe {
            is.require(
                ok,
                &format!("$.collaboration.{name}"),
                IssueCode::UnsafeFlag,
                message,
            );
        }
        is.require(
            (30..=MAX_PAIRING_TTL_SECONDS).contains(&c.pairing_code_ttl_seconds),
            "$.collaboration.pairing_code_ttl_seconds",
            IssueCode::InvalidValue,
            "codul de asociere expiră între 30 și 300 de secunde",
        );
        is.require(
            (1..=10).contains(&c.pairing_max_attempts),
            "$.collaboration.pairing_max_attempts",
            IssueCode::InvalidValue,
            "între 1 și 10 încercări de asociere",
        );
        if let Some(ip) = &c.host_bind_ip {
            match ip.parse::<IpAddr>() {
                Ok(addr) if addr.is_unspecified() => is.push(
                    "$.collaboration.host_bind_ip",
                    IssueCode::UnsafeFlag,
                    "bind la 0.0.0.0 sau :: este interzis; alege interfața VPN/LAN",
                ),
                Ok(addr) if addr.is_multicast() || addr.is_loopback() => is.push(
                    "$.collaboration.host_bind_ip",
                    IssueCode::InvalidValue,
                    "adresa trebuie să fie a adaptorului VPN/LAN ales",
                ),
                Ok(_) => {}
                Err(_) => is.push(
                    "$.collaboration.host_bind_ip",
                    IssueCode::InvalidValue,
                    "adresă IP invalidă (nu se acceptă hostname)",
                ),
            }
        }
        if let Some(port) = c.host_port {
            is.require(
                port >= 1024,
                "$.collaboration.host_port",
                IssueCode::InvalidValue,
                "portul trebuie să fie ≥ 1024",
            );
        }
        is.require(
            !(c.enabled && c.mode == CollaborationMode::Solo),
            "$.collaboration.mode",
            IssueCode::Inconsistent,
            "colaborarea activă cere modul host sau participant",
        );
        if c.enabled && c.mode == CollaborationMode::Host {
            is.require(
                c.host_bind_ip.is_some() && c.host_port.is_some(),
                "$.collaboration.host_bind_ip",
                IssueCode::MissingValue,
                "gazda alege explicit IP-ul adaptorului și portul",
            );
        }
        for (i, root) in c.shared_project_roots.iter().enumerate() {
            is.relative_path(root, &format!("$.collaboration.shared_project_roots[{i}]"));
        }
        for (i, path) in c.shared_vault_paths.iter().enumerate() {
            is.relative_path(path, &format!("$.collaboration.shared_vault_paths[{i}]"));
        }
        if mode == Mode::Execution && c.enabled {
            is.push(
                "$.collaboration.enabled",
                IssueCode::FeatureNotAvailable,
                "host/join se implementează în etapa C1",
            );
        }
    }

    fn validate_device_worker(&self, is: &mut Issues, mode: Mode) {
        let d = &self.device_worker;
        let safe = [
            (d.require_local_consent, "require_local_consent"),
            (d.require_plan_hash, "require_plan_hash"),
            (
                d.require_lease_and_fencing_token,
                "require_lease_and_fencing_token",
            ),
            (!d.expose_local_mcp_or_ollama, "expose_local_mcp_or_ollama"),
        ];
        for (ok, name) in safe {
            is.require(
                ok,
                &format!("$.device_worker.{name}"),
                IssueCode::UnsafeFlag,
                "această protecție a dispozitivului nu poate fi dezactivată",
            );
        }
        is.require(
            d.max_concurrent_tasks == 1,
            "$.device_worker.max_concurrent_tasks",
            IssueCode::InvalidValue,
            "o singură sarcină simultană per PC",
        );
        is.finite_non_negative(d.max_api_run_usd, "$.device_worker.max_api_run_usd");
        for (i, id) in d.allowed_project_ids.iter().enumerate() {
            is.require(
                super::agents::is_identifier(id),
                &format!("$.device_worker.allowed_project_ids[{i}]"),
                IssueCode::InvalidValue,
                "id de proiect invalid",
            );
        }
        for tool in &d.allowed_tools {
            if matches!(tool.effect(), crate::ids::ToolEffect::External(_)) {
                is.push(
                    "$.device_worker.allowed_tools",
                    IssueCode::UnsafeFlag,
                    format!("`{tool}` nu poate rula ca sarcină remote"),
                );
            }
        }
        if mode == Mode::Execution && d.accept_remote_tasks {
            is.push(
                "$.device_worker.accept_remote_tasks",
                IssueCode::FeatureNotAvailable,
                "sarcinile pe alt PC se implementează în etapa C3",
            );
        }
    }

    fn validate_chat(&self, is: &mut Issues) {
        let c = &self.chat;
        is.require(
            c.default_visibility == Visibility::Private,
            "$.chat.default_visibility",
            IssueCode::UnsafeFlag,
            "conversațiile sunt private implicit",
        );
        is.require(
            c.require_explicit_share,
            "$.chat.require_explicit_share",
            IssueCode::UnsafeFlag,
            "partajarea cere alegere explicită",
        );
        is.require(
            c.sanitize_markdown,
            "$.chat.sanitize_markdown",
            IssueCode::UnsafeFlag,
            "Markdown-ul este sanitizat",
        );
        is.require(
            c.general_conversations || c.project_conversations,
            "$.chat",
            IssueCode::Inconsistent,
            "cel puțin un tip de conversație trebuie permis",
        );
    }

    fn validate_attachments(&self, is: &mut Issues) {
        let a = &self.attachments;
        is.require(
            a.max_file_bytes > 0 && a.max_file_bytes <= MAX_FILE_BYTES,
            "$.attachments.max_file_bytes",
            IssueCode::InvalidValue,
            "limita pe fișier este între 1 B și 200 MiB",
        );
        is.require(
            a.max_shared_project_bytes >= a.max_file_bytes
                && a.max_shared_project_bytes <= MAX_SHARED_PROJECT_BYTES,
            "$.attachments.max_shared_project_bytes",
            IssueCode::InvalidValue,
            "limita pe proiect este ≥ limita pe fișier și ≤ 2 GiB",
        );
        is.require(
            (64 * 1024..=16 * 1024 * 1024).contains(&a.chunk_bytes)
                && a.chunk_bytes <= a.max_file_bytes.max(64 * 1024),
            "$.attachments.chunk_bytes",
            IssueCode::InvalidValue,
            "chunk între 64 KiB și 16 MiB",
        );
        is.require(
            a.verify_content_hash,
            "$.attachments.verify_content_hash",
            IssueCode::UnsafeFlag,
            "hash-ul conținutului se verifică obligatoriu",
        );
        is.require(
            !a.execute_attachments,
            "$.attachments.execute_attachments",
            IssueCode::UnsafeFlag,
            "atașamentele nu se execută niciodată",
        );
    }

    fn validate_blender(&self, is: &mut Issues, mode: Mode) {
        let b = &self.blender_mcp;
        let safe = [
            (
                b.require_adapter_security_review,
                "require_adapter_security_review",
            ),
            (!b.allow_arbitrary_python, "allow_arbitrary_python"),
            (b.require_scene_copy, "require_scene_copy"),
            (b.require_export_validation, "require_export_validation"),
        ];
        for (ok, name) in safe {
            is.require(
                ok,
                &format!("$.blender_mcp.{name}"),
                IssueCode::UnsafeFlag,
                "această protecție Blender nu poate fi dezactivată",
            );
        }
        if mode == Mode::Execution && b.enabled {
            is.push(
                "$.blender_mcp.enabled",
                IssueCode::FeatureNotAvailable,
                "adaptorul Blender rămâne neconfigurat până la validarea din etapa B1",
            );
        }
    }
}
