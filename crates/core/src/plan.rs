//! Planul de lucru: model, validare și hash canonic.
//!
//! Hash-ul este SHA-256 peste JSON canonicalizat (chei sortate, fără spații),
//! calculat din conținutul planului fără câmpurile derivate (`status`,
//! `approval`, `plan_sha256`). Orice altă schimbare produce alt hash și
//! invalidează aprobarea.

use std::collections::{HashMap, HashSet};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::config::{
    ConfigError, Issue, IssueCode, Mode, contains_placeholder, is_loopback_host, parse_simple_url,
};
use crate::config::{Issues, collect_placeholders, is_absolute_local_path};
use crate::ids::{ProviderId, ToolId};
use crate::paths::{RelPath, parse_relative};

pub const SCHEMA_VERSION: &str = "0.1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PlanStatus {
    Draft,
    AwaitingApproval,
    Approved,
    Superseded,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    pub schema_version: String,
    pub example_only: bool,
    pub id: String,
    pub version: u32,
    pub status: PlanStatus,
    pub project_id: String,
    pub project_root: String,
    pub base_snapshot_sha256: Option<String>,
    pub objective: String,
    pub provider: ProviderId,
    pub model: String,
    pub budget: PlanBudget,
    pub scope: PlanScope,
    pub tasks: Vec<PlanTask>,
    /// Generată de aplicație, niciodată de model sau de mână: un document cu
    /// valoare ne-nulă este respins.
    pub approval: Option<serde_json::Value>,
    pub plan_sha256: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PlanBudget {
    pub max_usd: f64,
    pub max_retries_per_task: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PlanScope {
    pub write_paths: Vec<String>,
    pub network_destinations: Vec<String>,
    pub external_data: String,
    pub studio_id: Option<String>,
    pub github_repository: Option<String>,
    pub allow_publish: bool,
    pub allow_upload: bool,
    pub allow_push: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PlanTask {
    pub id: String,
    pub depends_on: Vec<String>,
    pub allowed_tools: Vec<ToolId>,
    pub deliverables: Vec<String>,
    pub acceptance: Vec<String>,
}

impl Plan {
    pub fn parse(json: &str) -> Result<Self, ConfigError> {
        serde_json::from_str(json).map_err(|e| ConfigError::Parse(e.to_string()))
    }

    pub fn load(json: &str, mode: Mode) -> Result<Self, ConfigError> {
        let plan = Self::parse(json)?;
        let issues = plan.validate(mode);
        if issues.is_empty() {
            Ok(plan)
        } else {
            Err(ConfigError::Invalid(issues))
        }
    }

    pub fn task(&self, id: &str) -> Option<&PlanTask> {
        self.tasks.iter().find(|t| t.id == id)
    }

    /// Căile de scriere permise, deja validate.
    pub fn write_scopes(&self) -> Vec<RelPath> {
        self.scope
            .write_paths
            .iter()
            .filter_map(|p| parse_relative(p).ok())
            .collect()
    }

    pub fn sha256(&self) -> String {
        let mut value = serde_json::to_value(self).unwrap_or(serde_json::Value::Null);
        if let serde_json::Value::Object(map) = &mut value {
            map.remove("status");
            map.remove("approval");
            map.remove("plan_sha256");
        }
        let mut canonical = String::new();
        write_canonical(&value, &mut canonical);
        sha256_hex(canonical.as_bytes())
    }

    pub fn validate(&self, mode: Mode) -> Vec<Issue> {
        let mut is = Issues::default();
        is.require(
            self.schema_version == SCHEMA_VERSION,
            "$.schema_version",
            IssueCode::UnsupportedSchemaVersion,
            "versiune de schemă nesuportată (așteptat 0.1)",
        );
        is.require(
            crate::config::agents::is_identifier(&self.id),
            "$.id",
            IssueCode::InvalidValue,
            "id de plan invalid",
        );
        is.require(
            crate::config::agents::is_identifier(&self.project_id),
            "$.project_id",
            IssueCode::InvalidValue,
            "id de proiect invalid",
        );
        is.require(
            self.version >= 1,
            "$.version",
            IssueCode::InvalidValue,
            "versiunea începe de la 1",
        );
        is.require(
            !self.objective.trim().is_empty(),
            "$.objective",
            IssueCode::MissingValue,
            "planul are nevoie de un obiectiv",
        );
        is.require(
            self.approval.is_none()
                && self.status != PlanStatus::Approved
                && self.plan_sha256.is_none(),
            "$.approval",
            IssueCode::UnsafeFlag,
            "aprobarea și hash-ul sunt generate de aplicație, nu scrise în document",
        );
        if let Some(hash) = &self.base_snapshot_sha256 {
            is.require(
                is_sha256_hex(hash),
                "$.base_snapshot_sha256",
                IssueCode::InvalidValue,
                "hash SHA-256 hex invalid",
            );
        }
        is.finite_non_negative(self.budget.max_usd, "$.budget.max_usd");
        is.require(
            self.budget.max_retries_per_task <= 5,
            "$.budget.max_retries_per_task",
            IssueCode::InvalidValue,
            "maximum 5 reîncercări pe sarcină",
        );
        self.validate_scope(&mut is);
        self.validate_tasks(&mut is);
        if mode == Mode::Execution {
            is.require(
                !self.example_only,
                "$.example_only",
                IssueCode::ExampleOnly,
                "planul exemplu nu poate fi aprobat sau executat",
            );
            if !contains_placeholder(&self.project_root) {
                is.require(
                    is_absolute_local_path(&self.project_root),
                    "$.project_root",
                    IssueCode::InvalidValue,
                    "rădăcina proiectului trebuie să fie o cale locală absolută",
                );
            }
            collect_placeholders(self, &mut is);
        }
        is.0
    }

    fn validate_scope(&self, is: &mut Issues) {
        for (i, p) in self.scope.write_paths.iter().enumerate() {
            is.relative_path(p, &format!("$.scope.write_paths[{i}]"));
        }
        for (i, dest) in self.scope.network_destinations.iter().enumerate() {
            let ok = parse_simple_url(dest).is_some_and(|u| {
                u.scheme == "https" || (u.scheme == "http" && is_loopback_host(&u.host))
            });
            is.require(
                ok,
                &format!("$.scope.network_destinations[{i}]"),
                IssueCode::InvalidValue,
                "destinațiile sunt https sau endpointul local Ollama",
            );
        }
        if self.scope.allow_push {
            is.require(
                self.scope.github_repository.is_some(),
                "$.scope.github_repository",
                IssueCode::MissingValue,
                "push-ul cere repository-ul exact",
            );
        }
    }

    fn validate_tasks(&self, is: &mut Issues) {
        is.require(
            !self.tasks.is_empty(),
            "$.tasks",
            IssueCode::MissingValue,
            "planul are cel puțin o sarcină",
        );
        let scopes = self.write_scopes();
        let mut ids = HashSet::new();
        for (i, task) in self.tasks.iter().enumerate() {
            let path = format!("$.tasks[{i}]");
            is.require(
                crate::config::agents::is_identifier(&task.id),
                &format!("{path}.id"),
                IssueCode::InvalidValue,
                "id de sarcină invalid",
            );
            if !ids.insert(task.id.as_str()) {
                is.push(&path, IssueCode::DuplicateId, "sarcină duplicată");
            }
            is.require(
                !task.allowed_tools.is_empty(),
                &format!("{path}.allowed_tools"),
                IssueCode::MissingValue,
                "sarcina are cel puțin un instrument",
            );
            for (n, deliverable) in task.deliverables.iter().enumerate() {
                let dpath = format!("{path}.deliverables[{n}]");
                match parse_relative(deliverable) {
                    Ok(rel) => is.require(
                        scopes.iter().any(|s| rel.is_within(s)),
                        &dpath,
                        IssueCode::Inconsistent,
                        "livrabilul este în afara căilor de scriere ale planului",
                    ),
                    Err(err) => is.push(&dpath, IssueCode::InvalidValue, err.to_string()),
                }
            }
        }
        for (i, task) in self.tasks.iter().enumerate() {
            for dep in &task.depends_on {
                is.require(
                    ids.contains(dep.as_str()) && dep != &task.id,
                    &format!("$.tasks[{i}].depends_on"),
                    IssueCode::InvalidValue,
                    "dependență necunoscută sau către sine",
                );
            }
        }
        if has_cycle(&self.tasks) {
            is.push(
                "$.tasks",
                IssueCode::Inconsistent,
                "dependențele formează un ciclu",
            );
        }
    }
}

fn has_cycle(tasks: &[PlanTask]) -> bool {
    let index: HashMap<&str, usize> = tasks
        .iter()
        .enumerate()
        .map(|(i, t)| (t.id.as_str(), i))
        .collect();
    // 0 = nevizitat, 1 = în stivă, 2 = terminat
    let mut state = vec![0u8; tasks.len()];
    fn visit(i: usize, tasks: &[PlanTask], index: &HashMap<&str, usize>, state: &mut [u8]) -> bool {
        match state[i] {
            1 => return true,
            2 => return false,
            _ => {}
        }
        state[i] = 1;
        for dep in &tasks[i].depends_on {
            if let Some(&j) = index.get(dep.as_str())
                && visit(j, tasks, index, state)
            {
                return true;
            }
        }
        state[i] = 2;
        false
    }
    (0..tasks.len()).any(|i| visit(i, tasks, &index, &mut state))
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(64);
    for byte in digest.iter() {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

pub fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64
        && value
            .chars()
            .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c))
}

/// JSON canonic: chei sortate lexicografic, fără spații, zero normalizat.
fn write_canonical(value: &serde_json::Value, out: &mut String) {
    match value {
        serde_json::Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            out.push('{');
            for (i, key) in keys.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push_str(&serde_json::Value::String((*key).clone()).to_string());
                out.push(':');
                if let Some(v) = map.get(*key) {
                    write_canonical(v, out);
                }
            }
            out.push('}');
        }
        serde_json::Value::Array(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_canonical(item, out);
            }
            out.push(']');
        }
        serde_json::Value::Number(n) if n.as_f64() == Some(0.0) => out.push_str("0.0"),
        other => out.push_str(&other.to_string()),
    }
}
