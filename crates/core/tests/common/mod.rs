//! Construiește configurații și planuri executabile din exemplele din `config/`.
#![allow(dead_code, clippy::unwrap_used)]

use rab_core::config::{AgentsConfig, Mode};
use rab_core::examples::{AGENTS_EXAMPLE_JSON, PLAN_EXAMPLE_JSON};
use rab_core::plan::Plan;
use serde_json::{Value, json};

pub const APPROVER: &str = "DESKTOP-TEST\\andrei";
pub const NOW: &str = "2026-10-22T12:00:00Z";

pub fn agents_value() -> Value {
    serde_json::from_str(AGENTS_EXAMPLE_JSON).unwrap()
}

pub fn plan_value() -> Value {
    serde_json::from_str(PLAN_EXAMPLE_JSON).unwrap()
}

/// Configurație completată ca de wizard: fără placeholder-e, fără example_only.
pub fn exec_config_value(root: &str) -> Value {
    let mut v = agents_value();
    v["example_only"] = json!(false);
    v["project"]["root"] = json!(root);
    v["project"]["vault_root"] = json!(format!("{root}/vault"));
    v["providers"]["ollama"]["model"] = json!("qwen2.5-coder:7b");
    v
}

pub fn exec_config(root: &str) -> AgentsConfig {
    let text = exec_config_value(root).to_string();
    AgentsConfig::load(&text, Mode::Execution).unwrap()
}

pub fn exec_plan_value(root: &str) -> Value {
    let mut v = plan_value();
    v["example_only"] = json!(false);
    v["project_root"] = json!(root);
    v["model"] = json!("qwen2.5-coder:7b");
    v["status"] = json!("awaiting_approval");
    v
}

pub fn plan_from(value: Value) -> Plan {
    Plan::load(&value.to_string(), Mode::Execution).unwrap()
}

pub fn exec_plan(root: &str) -> Plan {
    plan_from(exec_plan_value(root))
}
