//! Contractul `agents` v0.1 (vezi `config/agents.example.json`).

use std::collections::HashSet;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{
    ConfigError, Issue, IssueCode, Issues, Mode, collect_placeholders, is_absolute_local_path,
    is_loopback_host, is_public_hostname, parse_simple_url,
};
use crate::ids::{AgentRole, ProviderId, ToolId};

pub const SCHEMA_VERSION: &str = "0.1";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AgentsConfig {
    pub schema_version: String,
    pub example_only: bool,
    pub project: ProjectConfig,
    pub runtime: RuntimeConfig,
    pub budget: BudgetConfig,
    pub providers: ProvidersConfig,
    pub policy: PolicyConfig,
    pub agents: Vec<AgentConfig>,
    pub integrations: IntegrationsConfig,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectConfig {
    pub id: String,
    pub root: String,
    pub vault_root: String,
    pub vault_write_subdirectory: String,
    pub personal_notes_read_only: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IpcTransport {
    WindowsNamedPipe,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RuntimeConfig {
    pub max_concurrent_tasks: u32,
    pub max_agent_steps_per_task: u32,
    pub max_retries: u32,
    pub task_timeout_seconds: u64,
    pub max_tool_output_bytes: u64,
    pub background_worker: bool,
    pub start_at_logon: bool,
    pub ipc_transport: IpcTransport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum Currency {
    #[serde(rename = "USD")]
    Usd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum UnknownPriceAction {
    Deny,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BudgetConfig {
    pub currency: Currency,
    pub max_run_usd: f64,
    pub max_month_usd: f64,
    pub reserve_before_request: bool,
    pub unknown_price_action: UnknownPriceAction,
    pub paid_fallback: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProvidersConfig {
    pub ollama: OllamaConfig,
    pub anthropic_api: ApiProviderConfig,
    pub openai_images: OpenAiImagesConfig,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OllamaConfig {
    pub enabled: bool,
    pub endpoint: String,
    pub model: String,
    pub allow_cloud_models: bool,
    pub context_tokens: u32,
    pub max_output_tokens: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ApiProviderConfig {
    pub enabled: bool,
    pub model: Option<String>,
    pub credential_ref: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ImageQuality {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OpenAiImagesConfig {
    pub enabled: bool,
    pub model: Option<String>,
    pub credential_ref: String,
    pub max_images_per_run: u32,
    pub quality: ImageQuality,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum NetworkDefault {
    Deny,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PolicyConfig {
    pub require_approved_plan: bool,
    pub invalidate_approval_on_plan_change: bool,
    pub allow_arbitrary_shell: bool,
    pub allow_arbitrary_host_code: bool,
    pub allow_unrestricted_luau: bool,
    pub allow_paid_calls: bool,
    pub allow_upload: bool,
    pub allow_publish: bool,
    pub allow_git_push: bool,
    pub allow_github_pr: bool,
    pub network_default: NetworkDefault,
    pub research_allowed_hosts: Vec<String>,
    pub require_plan_for_network: bool,
    pub reject_external_private_network_targets: bool,
    pub secrets_in_model_context: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AgentConfig {
    pub id: AgentRole,
    pub provider: ProviderId,
    pub tools: Vec<ToolId>,
    pub output_contract: crate::ids::OutputContract,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct IntegrationsConfig {
    pub roblox_studio_mcp: StudioMcpConfig,
    pub rojo: RojoConfig,
    pub github: GithubConfig,
    pub claude_code: ClaudeCodeConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum McpTransport {
    Stdio,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum StudioConfigurationSource {
    StudioOfficialSetup,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StudioMcpConfig {
    pub enabled: bool,
    pub transport: McpTransport,
    pub configuration_source: StudioConfigurationSource,
    pub studio_id: Option<String>,
    pub require_capability_discovery: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RojoConfig {
    pub enabled: bool,
    pub project_file: String,
    pub filesystem_owned_paths: Vec<String>,
    pub require_backup_before_first_sync: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GithubAuth {
    GitCredentialManager,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GithubConfig {
    pub enabled: bool,
    pub repository: Option<String>,
    pub auth: GithubAuth,
    pub private_by_default: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ClaudeCodeMode {
    ExternalDevelopmentToolOnly,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ClaudeCodeConfig {
    pub mode: ClaudeCodeMode,
    pub embedded_provider_enabled: bool,
}

impl AgentsConfig {
    /// Parsare strictă; câmpurile necunoscute și valorile enum necunoscute sunt erori.
    pub fn parse(json: &str) -> Result<Self, ConfigError> {
        serde_json::from_str(json).map_err(|e| ConfigError::Parse(e.to_string()))
    }

    /// Parsare + validare; returnează configurația numai dacă nu are probleme.
    pub fn load(json: &str, mode: Mode) -> Result<Self, ConfigError> {
        let config = Self::parse(json)?;
        let issues = config.validate(mode);
        if issues.is_empty() {
            Ok(config)
        } else {
            Err(ConfigError::Invalid(issues))
        }
    }

    pub fn provider_enabled(&self, provider: ProviderId) -> bool {
        match provider {
            ProviderId::Ollama => self.providers.ollama.enabled,
            ProviderId::AnthropicApi => self.providers.anthropic_api.enabled,
            ProviderId::OpenaiImages => self.providers.openai_images.enabled,
        }
    }

    pub fn validate(&self, mode: Mode) -> Vec<Issue> {
        let mut is = Issues::default();
        is.require(
            self.schema_version == SCHEMA_VERSION,
            "$.schema_version",
            IssueCode::UnsupportedSchemaVersion,
            "versiune de schemă nesuportată (așteptat 0.1)",
        );
        self.validate_project(&mut is, mode);
        self.validate_runtime(&mut is);
        self.validate_budget(&mut is);
        self.validate_providers(&mut is, mode);
        self.validate_policy(&mut is);
        self.validate_agents(&mut is);
        self.validate_integrations(&mut is, mode);
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

    fn validate_project(&self, is: &mut Issues, mode: Mode) {
        let p = &self.project;
        is.require(
            is_identifier(&p.id),
            "$.project.id",
            IssueCode::InvalidValue,
            "id-ul proiectului poate conține doar a-z, 0-9, `-` și `_`",
        );
        is.relative_path(
            &p.vault_write_subdirectory,
            "$.project.vault_write_subdirectory",
        );
        is.require(
            p.personal_notes_read_only,
            "$.project.personal_notes_read_only",
            IssueCode::UnsafeFlag,
            "notele personale rămân read-only",
        );
        if mode == Mode::Execution {
            for (value, path) in [
                (&p.root, "$.project.root"),
                (&p.vault_root, "$.project.vault_root"),
            ] {
                if !super::contains_placeholder(value) {
                    is.require(
                        is_absolute_local_path(value),
                        path,
                        IssueCode::InvalidValue,
                        "rădăcina trebuie să fie o cale locală absolută (fără UNC)",
                    );
                }
            }
        }
    }

    fn validate_runtime(&self, is: &mut Issues) {
        let r = &self.runtime;
        is.require(
            r.max_concurrent_tasks == 1,
            "$.runtime.max_concurrent_tasks",
            IssueCode::InvalidValue,
            "inițial este permis un singur job simultan per PC",
        );
        is.require(
            (1..=50).contains(&r.max_agent_steps_per_task),
            "$.runtime.max_agent_steps_per_task",
            IssueCode::InvalidValue,
            "număr de pași între 1 și 50",
        );
        is.require(
            r.max_retries <= 5,
            "$.runtime.max_retries",
            IssueCode::InvalidValue,
            "maximum 5 reîncercări",
        );
        is.require(
            (1..=3600).contains(&r.task_timeout_seconds),
            "$.runtime.task_timeout_seconds",
            IssueCode::InvalidValue,
            "timeout între 1 și 3600 secunde",
        );
        is.require(
            (1..=16 * 1024 * 1024).contains(&r.max_tool_output_bytes),
            "$.runtime.max_tool_output_bytes",
            IssueCode::InvalidValue,
            "limită de output între 1 B și 16 MiB",
        );
        is.require(
            !r.start_at_logon || r.background_worker,
            "$.runtime.start_at_logon",
            IssueCode::Inconsistent,
            "pornirea la logon cere lucrul în fundal",
        );
    }

    fn validate_budget(&self, is: &mut Issues) {
        let b = &self.budget;
        is.finite_non_negative(b.max_run_usd, "$.budget.max_run_usd");
        is.finite_non_negative(b.max_month_usd, "$.budget.max_month_usd");
        is.require(
            b.max_run_usd <= b.max_month_usd,
            "$.budget.max_run_usd",
            IssueCode::Inconsistent,
            "limita pe rulare nu poate depăși limita lunară",
        );
        is.require(
            b.reserve_before_request,
            "$.budget.reserve_before_request",
            IssueCode::UnsafeFlag,
            "bugetul se rezervă înaintea fiecărei cereri",
        );
        is.require(
            !b.paid_fallback,
            "$.budget.paid_fallback",
            IssueCode::UnsafeFlag,
            "fallback automat către un furnizor plătit este interzis",
        );
    }

    fn validate_providers(&self, is: &mut Issues, mode: Mode) {
        let o = &self.providers.ollama;
        let local = parse_simple_url(&o.endpoint)
            .is_some_and(|u| u.scheme == "http" && is_loopback_host(&u.host));
        is.require(
            local,
            "$.providers.ollama.endpoint",
            IssueCode::InvalidValue,
            "Ollama folosește doar un endpoint local (127.0.0.1, localhost sau [::1])",
        );
        is.require(
            (512..=1_048_576).contains(&o.context_tokens),
            "$.providers.ollama.context_tokens",
            IssueCode::InvalidValue,
            "context între 512 și 1048576 tokeni",
        );
        is.require(
            o.max_output_tokens > 0 && o.max_output_tokens <= o.context_tokens,
            "$.providers.ollama.max_output_tokens",
            IssueCode::InvalidValue,
            "output-ul trebuie să fie > 0 și ≤ contextul",
        );
        if mode == Mode::Execution && o.enabled {
            is.require(
                !o.model.trim().is_empty(),
                "$.providers.ollama.model",
                IssueCode::MissingValue,
                "alege un model local instalat",
            );
        }

        let paid = [
            (
                "$.providers.anthropic_api",
                self.providers.anthropic_api.enabled,
                &self.providers.anthropic_api.model,
                &self.providers.anthropic_api.credential_ref,
            ),
            (
                "$.providers.openai_images",
                self.providers.openai_images.enabled,
                &self.providers.openai_images.model,
                &self.providers.openai_images.credential_ref,
            ),
        ];
        for (path, enabled, model, credential_ref) in paid {
            is.require(
                credential_ref.starts_with("credential-manager://"),
                &format!("{path}.credential_ref"),
                IssueCode::InvalidValue,
                "cheile sunt referințe către Credential Manager, nu secrete",
            );
            if enabled {
                is.require(
                    model.as_deref().is_some_and(|m| !m.trim().is_empty()),
                    &format!("{path}.model"),
                    IssueCode::MissingValue,
                    "un furnizor activ are nevoie de model explicit",
                );
                let budget_ok = self.policy.allow_paid_calls
                    && self.budget.max_run_usd > 0.0
                    && self.budget.max_month_usd > 0.0;
                is.require(
                    budget_ok,
                    &format!("{path}.enabled"),
                    IssueCode::Inconsistent,
                    "un furnizor plătit activ cere allow_paid_calls și buget > 0",
                );
            }
        }
        is.require(
            (1..=10).contains(&self.providers.openai_images.max_images_per_run),
            "$.providers.openai_images.max_images_per_run",
            IssueCode::InvalidValue,
            "între 1 și 10 imagini pe rulare",
        );
    }

    fn validate_policy(&self, is: &mut Issues) {
        let p = &self.policy;
        let must_be_false = [
            (p.allow_arbitrary_shell, "allow_arbitrary_shell"),
            (p.allow_arbitrary_host_code, "allow_arbitrary_host_code"),
            (p.allow_unrestricted_luau, "allow_unrestricted_luau"),
            (p.secrets_in_model_context, "secrets_in_model_context"),
        ];
        for (value, name) in must_be_false {
            is.require(
                !value,
                &format!("$.policy.{name}"),
                IssueCode::UnsafeFlag,
                "această permisiune nu poate fi activată",
            );
        }
        let must_be_true = [
            (p.require_approved_plan, "require_approved_plan"),
            (
                p.invalidate_approval_on_plan_change,
                "invalidate_approval_on_plan_change",
            ),
            (p.require_plan_for_network, "require_plan_for_network"),
            (
                p.reject_external_private_network_targets,
                "reject_external_private_network_targets",
            ),
        ];
        for (value, name) in must_be_true {
            is.require(
                value,
                &format!("$.policy.{name}"),
                IssueCode::UnsafeFlag,
                "această protecție nu poate fi dezactivată",
            );
        }
        if p.allow_paid_calls {
            is.require(
                self.budget.max_run_usd > 0.0,
                "$.policy.allow_paid_calls",
                IssueCode::Inconsistent,
                "apelurile plătite cer un buget pe rulare > 0",
            );
        }
        for (i, host) in p.research_allowed_hosts.iter().enumerate() {
            is.require(
                is_public_hostname(host),
                &format!("$.policy.research_allowed_hosts[{i}]"),
                IssueCode::InvalidValue,
                "doar hostname-uri publice, cu litere mici; fără IP, localhost sau schemă",
            );
        }
    }

    fn validate_agents(&self, is: &mut Issues) {
        is.require(
            !self.agents.is_empty(),
            "$.agents",
            IssueCode::MissingValue,
            "este nevoie de cel puțin un agent",
        );
        let mut seen = HashSet::new();
        for (i, agent) in self.agents.iter().enumerate() {
            let path = format!("$.agents[{i}]");
            if !seen.insert(agent.id) {
                is.push(&path, IssueCode::DuplicateId, "rol de agent duplicat");
            }
            is.require(
                agent.output_contract == agent.id.expected_contract(),
                &format!("{path}.output_contract"),
                IssueCode::ContractMismatch,
                "contractul de ieșire nu corespunde rolului",
            );
            is.require(
                self.provider_enabled(agent.provider),
                &format!("{path}.provider"),
                IssueCode::ProviderDisabled,
                "agentul folosește un furnizor dezactivat",
            );
            let mut tools = HashSet::new();
            for tool in &agent.tools {
                if !tools.insert(*tool) {
                    is.push(
                        &format!("{path}.tools"),
                        IssueCode::DuplicateId,
                        "instrument duplicat",
                    );
                }
                if !tool.agent_may_hold() {
                    is.push(
                        &format!("{path}.tools"),
                        IssueCode::AgentToolNotAllowed,
                        format!("`{tool}` aparține executorului, nu agenților"),
                    );
                }
            }
        }
    }

    fn validate_integrations(&self, is: &mut Issues, mode: Mode) {
        let i = &self.integrations;
        is.require(
            i.roblox_studio_mcp.require_capability_discovery,
            "$.integrations.roblox_studio_mcp.require_capability_discovery",
            IssueCode::UnsafeFlag,
            "capabilitățile Studio MCP se descoperă înainte de folosire",
        );
        is.require(
            i.rojo.require_backup_before_first_sync,
            "$.integrations.rojo.require_backup_before_first_sync",
            IssueCode::UnsafeFlag,
            "backup obligatoriu înaintea primei sincronizări",
        );
        is.relative_path(&i.rojo.project_file, "$.integrations.rojo.project_file");
        for (n, owned) in i.rojo.filesystem_owned_paths.iter().enumerate() {
            is.relative_path(
                owned,
                &format!("$.integrations.rojo.filesystem_owned_paths[{n}]"),
            );
        }
        is.require(
            i.github.private_by_default,
            "$.integrations.github.private_by_default",
            IssueCode::UnsafeFlag,
            "repository-urile sunt private implicit",
        );
        is.require(
            !i.claude_code.embedded_provider_enabled,
            "$.integrations.claude_code.embedded_provider_enabled",
            IssueCode::UnsafeFlag,
            "Claude Code este instrument de dezvoltare separat, nu furnizor",
        );
        if mode == Mode::Execution {
            if i.roblox_studio_mcp.enabled {
                is.require(
                    i.roblox_studio_mcp.studio_id.is_some(),
                    "$.integrations.roblox_studio_mcp.studio_id",
                    IssueCode::MissingValue,
                    "instanța Studio se alege explicit",
                );
            }
            if i.github.enabled {
                is.require(
                    i.github.repository.is_some(),
                    "$.integrations.github.repository",
                    IssueCode::MissingValue,
                    "repository-ul GitHub se alege explicit",
                );
            }
        }
    }
}

pub(crate) fn is_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
}
