//! Configurațiile exemplu din `config/` și verdictul lor, afișat în UI.

use serde::Serialize;

use crate::config::{AgentsConfig, CollaborationConfig, ConfigError, Issue, IssueCode, Mode};
use crate::plan::Plan;

pub const AGENTS_EXAMPLE_JSON: &str = include_str!("../../../config/agents.example.json");
pub const COLLABORATION_EXAMPLE_JSON: &str =
    include_str!("../../../config/collaboration.example.json");
pub const PLAN_EXAMPLE_JSON: &str = include_str!("../../../config/plan.example.json");

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ExampleVerdict {
    pub name: &'static str,
    /// Parsare strictă reușită și invariante de siguranță respectate.
    pub contract_valid: bool,
    /// Poate fi folosită pentru execuție (exemplele nu pot).
    pub execution_ready: bool,
    pub execution_issue_codes: Vec<IssueCode>,
}

fn verdict(
    name: &'static str,
    parsed: Result<(Vec<Issue>, Vec<Issue>), ConfigError>,
) -> ExampleVerdict {
    match parsed {
        Ok((contract, execution)) => {
            let mut codes: Vec<IssueCode> = execution.iter().map(|i| i.code).collect();
            codes.dedup();
            ExampleVerdict {
                name,
                contract_valid: contract.is_empty(),
                execution_ready: execution.is_empty(),
                execution_issue_codes: codes,
            }
        }
        Err(_) => ExampleVerdict {
            name,
            contract_valid: false,
            execution_ready: false,
            execution_issue_codes: Vec::new(),
        },
    }
}

pub fn example_verdicts() -> Vec<ExampleVerdict> {
    vec![
        verdict(
            "agents.example.json",
            AgentsConfig::parse(AGENTS_EXAMPLE_JSON)
                .map(|c| (c.validate(Mode::Contract), c.validate(Mode::Execution))),
        ),
        verdict(
            "collaboration.example.json",
            CollaborationConfig::parse(COLLABORATION_EXAMPLE_JSON)
                .map(|c| (c.validate(Mode::Contract), c.validate(Mode::Execution))),
        ),
        verdict(
            "plan.example.json",
            Plan::parse(PLAN_EXAMPLE_JSON)
                .map(|p| (p.validate(Mode::Contract), p.validate(Mode::Execution))),
        ),
    ]
}
