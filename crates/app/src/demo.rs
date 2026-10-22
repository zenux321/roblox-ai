//! Proiectul demonstrativ al etapei M1: fixture, configurație, plan și
//! propunerea prestabilită a furnizorului fals.

use std::fs;
use std::path::Path;

use rab_core::config::{AgentsConfig, ConfigError, Mode};
use rab_core::examples::AGENTS_EXAMPLE_JSON;
use rab_core::ids::{AgentRole, ProviderId, ToolId};
use rab_core::plan::{Plan, PlanBudget, PlanScope, PlanStatus, PlanTask};
use serde_json::json;

pub const PROJECT_ID: &str = "obby-vulcanic";
pub const PROJECT_NAME: &str = "Obby Vulcanic";
pub const PLAN_ID: &str = "plan-obby-demo";
pub const DEMO_MODEL: &str = "demo-fake-provider";
pub const PROVIDER_LABEL: &str = "Furnizor demo (fals) · local, gratuit";

pub const TEST_ROJO: &str = "Structura Rojo este validă";
pub const TEST_REQUIRE: &str = "Fără module externe încărcate după ID";
pub const TEST_SCOPE: &str = "Fișierele modificate rămân în căile aprobate";
pub const TEST_STUDIO_PLAY: &str = "Checkpoint, respawn și finish testate în Studio";
pub const TEST_REMOTE: &str = "RemoteEvent falsificat respins de server (playtest)";

const FIXTURE: &[(&str, &str)] = &[
    (
        "default.project.json",
        include_str!("../../../tests/fixtures/obby-demo/default.project.json"),
    ),
    (
        "src/shared/Config.luau",
        include_str!("../../../tests/fixtures/obby-demo/src/shared/Config.luau"),
    ),
    (
        "README.md",
        include_str!("../../../tests/fixtures/obby-demo/README.md"),
    ),
];

/// Fișierele propuse de furnizorul demonstrativ (agentul „Constructor”).
pub const PROPOSAL: &[(&str, &str)] = &[
    (
        "src/server/Course.server.luau",
        include_str!("../demo/proposals/Course.server.luau"),
    ),
    (
        "src/server/CheckpointService.server.luau",
        include_str!("../demo/proposals/CheckpointService.server.luau"),
    ),
];

/// Copiază fixture-ul numai dacă proiectul nu există; nu suprascrie copia utilizatorului.
pub fn install_fixture(root: &Path) -> std::io::Result<bool> {
    if root.join("default.project.json").exists() {
        return Ok(false);
    }
    for (rel, content) in FIXTURE {
        let path = root.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, content)?;
    }
    Ok(true)
}

/// Configurația de execuție a demo-ului (nu se salvează ca setare a utilizatorului).
pub fn demo_config(root: &str, vault: &str) -> Result<AgentsConfig, ConfigError> {
    let mut value: serde_json::Value =
        serde_json::from_str(AGENTS_EXAMPLE_JSON).map_err(|e| ConfigError::Parse(e.to_string()))?;
    value["example_only"] = json!(false);
    value["project"]["id"] = json!(PROJECT_ID);
    value["project"]["root"] = json!(root);
    value["project"]["vault_root"] = json!(vault);
    value["providers"]["ollama"]["model"] = json!(DEMO_MODEL);
    AgentsConfig::load(&value.to_string(), Mode::Execution)
}

pub fn demo_plan(root: &str, version: u32) -> Plan {
    let task = |id: &str,
                title: &str,
                description: &str,
                agent: AgentRole,
                deps: &[&str],
                tools: &[ToolId],
                deliverables: &[&str],
                acceptance: &[&str]| PlanTask {
        id: id.into(),
        title: Some(title.into()),
        description: Some(description.into()),
        agent: Some(agent),
        depends_on: deps.iter().map(|d| d.to_string()).collect(),
        allowed_tools: tools.to_vec(),
        deliverables: deliverables.iter().map(|d| d.to_string()).collect(),
        acceptance: acceptance.iter().map(|a| a.to_string()).collect(),
    };
    Plan {
        schema_version: rab_core::plan::SCHEMA_VERSION.into(),
        example_only: false,
        id: PLAN_ID.into(),
        version,
        status: PlanStatus::AwaitingApproval,
        project_id: PROJECT_ID.into(),
        project_root: root.into(),
        base_snapshot_sha256: None,
        title: Some("Obby de test: checkpoint și finish".into()),
        objective:
            "Creează un obby de test cu cinci platforme, checkpoint și finish, fără publicare. \
                    Lava trimite jucătorul la ultimul checkpoint, iar serverul validează progresul."
                .into(),
        provider: ProviderId::Ollama,
        model: DEMO_MODEL.into(),
        budget: PlanBudget {
            max_usd: 0.0,
            max_retries_per_task: 2,
        },
        scope: PlanScope {
            write_paths: vec!["src".into(), "reports".into()],
            network_destinations: Vec::new(),
            external_data: "Niciuna: furnizorul demonstrativ rulează local".into(),
            studio_id: None,
            github_repository: None,
            allow_publish: false,
            allow_upload: false,
            allow_push: false,
        },
        tasks: vec![
            task(
                "generate",
                "Scrie traseul și checkpointul pe server",
                "Cinci platforme, lavă, finish; progresul se scrie numai pe server",
                AgentRole::Builder,
                &[],
                &[ToolId::ProjectRead, ToolId::ProjectApplyValidatedPatch],
                &["src/server"],
                &[TEST_ROJO, TEST_REQUIRE, TEST_SCOPE],
            ),
            task(
                "verify",
                "Verifică fișierele și scrie raportul",
                "Verificări statice; playtestul din Studio rămâne neexecutat până la M4",
                AgentRole::Reviewer,
                &["generate"],
                &[
                    ToolId::ProjectRead,
                    ToolId::TestsStaticLuau,
                    ToolId::ReportsWrite,
                ],
                &["reports/static-checks.md"],
                &[TEST_STUDIO_PLAY, TEST_REMOTE],
            ),
        ],
        approval: None,
        plan_sha256: None,
    }
}

/// Răspunsul prestabilit al furnizorului fals: o propunere de patch (JSON).
pub fn proposal_json() -> String {
    let files: Vec<serde_json::Value> = PROPOSAL
        .iter()
        .map(|(path, content)| json!({ "path": path, "content": content }))
        .collect();
    json!({ "contract": "patch_proposal", "files": files }).to_string()
}

/// Testele care au o verificare automată în această etapă.
pub fn is_static_test(title: &str) -> bool {
    matches!(title, TEST_ROJO | TEST_REQUIRE | TEST_SCOPE)
}
