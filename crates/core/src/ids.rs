//! Vocabularul intern: instrumente, furnizori, roluri și contracte de ieșire.
//!
//! Numele instrumentelor sunt contractul propriu al aplicației, nu numele
//! instrumentelor Roblox MCP; adaptoarele fac maparea după discovery.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Instrumentele cunoscute de policy engine. Un nume necunoscut este respins la parsare.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
pub enum ToolId {
    #[serde(rename = "memory.search")]
    MemorySearch,
    #[serde(rename = "memory.propose_note")]
    MemoryProposeNote,
    #[serde(rename = "project.read")]
    ProjectRead,
    #[serde(rename = "project.propose_patch")]
    ProjectProposePatch,
    #[serde(rename = "project.apply_validated_patch")]
    ProjectApplyValidatedPatch,
    #[serde(rename = "research.fetch")]
    ResearchFetch,
    #[serde(rename = "tests.read_results")]
    TestsReadResults,
    #[serde(rename = "tests.static_luau")]
    TestsStaticLuau,
    #[serde(rename = "reports.write")]
    ReportsWrite,
    #[serde(rename = "git.push")]
    GitPush,
    #[serde(rename = "github.create_pr")]
    GithubCreatePr,
    #[serde(rename = "roblox.upload_asset")]
    RobloxUploadAsset,
    #[serde(rename = "roblox.publish")]
    RobloxPublish,
}

/// Efecte externe care cer flag global, scope de plan și destinație exactă.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExternalAction {
    GitPush,
    PullRequest,
    Upload,
    Publish,
}

/// Clasa de efect a unui instrument; decide ce verificări aplică policy engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolEffect {
    /// Citire locală în rădăcina proiectului sau în memorie.
    Read,
    /// Produce o propunere (JSON), fără efect pe disc.
    Propose,
    /// Scriere locală făcută numai de executor.
    LocalWrite,
    /// Acces de rețea read-only, numai prin plan.
    Network,
    /// Efect în afara PC-ului.
    External(ExternalAction),
}

impl ToolId {
    pub const ALL: [ToolId; 13] = [
        ToolId::MemorySearch,
        ToolId::MemoryProposeNote,
        ToolId::ProjectRead,
        ToolId::ProjectProposePatch,
        ToolId::ProjectApplyValidatedPatch,
        ToolId::ResearchFetch,
        ToolId::TestsReadResults,
        ToolId::TestsStaticLuau,
        ToolId::ReportsWrite,
        ToolId::GitPush,
        ToolId::GithubCreatePr,
        ToolId::RobloxUploadAsset,
        ToolId::RobloxPublish,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            ToolId::MemorySearch => "memory.search",
            ToolId::MemoryProposeNote => "memory.propose_note",
            ToolId::ProjectRead => "project.read",
            ToolId::ProjectProposePatch => "project.propose_patch",
            ToolId::ProjectApplyValidatedPatch => "project.apply_validated_patch",
            ToolId::ResearchFetch => "research.fetch",
            ToolId::TestsReadResults => "tests.read_results",
            ToolId::TestsStaticLuau => "tests.static_luau",
            ToolId::ReportsWrite => "reports.write",
            ToolId::GitPush => "git.push",
            ToolId::GithubCreatePr => "github.create_pr",
            ToolId::RobloxUploadAsset => "roblox.upload_asset",
            ToolId::RobloxPublish => "roblox.publish",
        }
    }

    pub fn effect(self) -> ToolEffect {
        match self {
            ToolId::MemorySearch
            | ToolId::ProjectRead
            | ToolId::TestsReadResults
            | ToolId::TestsStaticLuau => ToolEffect::Read,
            ToolId::MemoryProposeNote | ToolId::ProjectProposePatch => ToolEffect::Propose,
            ToolId::ProjectApplyValidatedPatch | ToolId::ReportsWrite => ToolEffect::LocalWrite,
            ToolId::ResearchFetch => ToolEffect::Network,
            ToolId::GitPush => ToolEffect::External(ExternalAction::GitPush),
            ToolId::GithubCreatePr => ToolEffect::External(ExternalAction::PullRequest),
            ToolId::RobloxUploadAsset => ToolEffect::External(ExternalAction::Upload),
            ToolId::RobloxPublish => ToolEffect::External(ExternalAction::Publish),
        }
    }

    /// Agenții doar propun: nu pot deține instrumente care scriu sau au efect extern.
    pub fn agent_may_hold(self) -> bool {
        matches!(
            self.effect(),
            ToolEffect::Read | ToolEffect::Propose | ToolEffect::Network
        )
    }
}

impl std::fmt::Display for ToolId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProviderId {
    Ollama,
    AnthropicApi,
    OpenaiImages,
}

impl ProviderId {
    /// Furnizorii API sunt plătiți; Ollama local este gratuit.
    pub fn is_paid(self) -> bool {
        !matches!(self, ProviderId::Ollama)
    }
}

impl std::fmt::Display for ProviderId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ProviderId::Ollama => "ollama",
            ProviderId::AnthropicApi => "anthropic_api",
            ProviderId::OpenaiImages => "openai_images",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AgentRole {
    Planner,
    Researcher,
    Builder,
    Reviewer,
    MemoryCurator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OutputContract {
    PlanProposal,
    SourcedNotes,
    PatchProposal,
    ReviewReport,
    NoteProposal,
}

impl AgentRole {
    pub fn expected_contract(self) -> OutputContract {
        match self {
            AgentRole::Planner => OutputContract::PlanProposal,
            AgentRole::Researcher => OutputContract::SourcedNotes,
            AgentRole::Builder => OutputContract::PatchProposal,
            AgentRole::Reviewer => OutputContract::ReviewReport,
            AgentRole::MemoryCurator => OutputContract::NoteProposal,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_names_round_trip() {
        for tool in ToolId::ALL {
            let json = serde_json::to_string(&tool).expect("serializare");
            assert_eq!(json, format!("\"{}\"", tool.as_str()));
            let back: ToolId = serde_json::from_str(&json).expect("deserializare");
            assert_eq!(back, tool);
        }
    }

    #[test]
    fn unknown_tool_is_rejected() {
        assert!(serde_json::from_str::<ToolId>("\"shell.exec\"").is_err());
    }

    #[test]
    fn agents_cannot_hold_effect_tools() {
        assert!(!ToolId::ProjectApplyValidatedPatch.agent_may_hold());
        assert!(!ToolId::RobloxPublish.agent_may_hold());
        assert!(!ToolId::GitPush.agent_may_hold());
        assert!(ToolId::ProjectProposePatch.agent_may_hold());
    }
}
