//! Contractele de configurare: parsare strictă și validare semantică.
#![allow(clippy::unwrap_used)]

mod common;

use rab_core::config::{AgentsConfig, CollaborationConfig, ConfigError, IssueCode, Mode};
use rab_core::examples::{COLLABORATION_EXAMPLE_JSON, example_verdicts};
use serde_json::{Value, json};

fn agents_codes(v: &Value, mode: Mode) -> Vec<IssueCode> {
    AgentsConfig::parse(&v.to_string())
        .unwrap()
        .validate(mode)
        .into_iter()
        .map(|i| i.code)
        .collect()
}

fn collab_value() -> Value {
    serde_json::from_str(COLLABORATION_EXAMPLE_JSON).unwrap()
}

fn collab_codes(v: &Value, mode: Mode) -> Vec<IssueCode> {
    CollaborationConfig::parse(&v.to_string())
        .unwrap()
        .validate(mode)
        .into_iter()
        .map(|i| i.code)
        .collect()
}

#[test]
fn examples_are_valid_contracts_but_rejected_for_execution() {
    let verdicts = example_verdicts();
    assert_eq!(verdicts.len(), 3);
    for v in &verdicts {
        assert!(v.contract_valid, "{} trebuie să fie contract valid", v.name);
        assert!(!v.execution_ready, "{} nu poate executa", v.name);
        assert!(
            v.execution_issue_codes.contains(&IssueCode::ExampleOnly),
            "{}",
            v.name
        );
    }
    let agents = &verdicts[0];
    assert!(
        agents
            .execution_issue_codes
            .contains(&IssueCode::Placeholder)
    );
}

#[test]
fn unknown_fields_are_rejected_everywhere() {
    let mut top = common::agents_value();
    top["auto_approve"] = json!(true);
    assert!(matches!(
        AgentsConfig::parse(&top.to_string()),
        Err(ConfigError::Parse(_))
    ));

    let mut nested = common::agents_value();
    nested["policy"]["allow_everything"] = json!(true);
    let err = AgentsConfig::parse(&nested.to_string()).unwrap_err();
    assert!(err.to_string().contains("allow_everything"), "{err}");

    let mut collab = collab_value();
    collab["collaboration"]["relay_url"] = json!("https://example.com");
    assert!(CollaborationConfig::parse(&collab.to_string()).is_err());
}

#[test]
fn unknown_enum_values_and_tools_are_rejected() {
    let mut v = common::agents_value();
    v["agents"][0]["tools"] = json!(["memory.search", "shell.exec"]);
    assert!(AgentsConfig::parse(&v.to_string()).is_err());

    let mut v = common::agents_value();
    v["policy"]["network_default"] = json!("allow");
    assert!(AgentsConfig::parse(&v.to_string()).is_err());
}

#[test]
fn completed_config_is_execution_ready() {
    let config = common::exec_config("D:\\Roblox\\ObbyVulcanic");
    assert!(!config.example_only);
    assert!(config.validate(Mode::Execution).is_empty());
}

#[test]
fn placeholders_are_rejected_at_execution() {
    let mut v = common::exec_config_value("/proiect");
    v["providers"]["ollama"]["model"] = json!("<SELECT_INSTALLED_LOCAL_MODEL>");
    let issues = AgentsConfig::parse(&v.to_string())
        .unwrap()
        .validate(Mode::Execution);
    assert!(
        issues
            .iter()
            .any(|i| i.code == IssueCode::Placeholder && i.path == "$.providers.ollama.model")
    );
}

#[test]
fn unsafe_flags_are_rejected_even_as_contract() {
    let cases = [
        ("/budget/paid_fallback", json!(true)),
        ("/policy/allow_arbitrary_shell", json!(true)),
        ("/policy/allow_arbitrary_host_code", json!(true)),
        ("/policy/secrets_in_model_context", json!(true)),
        ("/policy/require_approved_plan", json!(false)),
        (
            "/integrations/claude_code/embedded_provider_enabled",
            json!(true),
        ),
        ("/project/personal_notes_read_only", json!(false)),
    ];
    for (pointer, value) in cases {
        let mut v = common::agents_value();
        *v.pointer_mut(pointer).unwrap() = value;
        assert!(
            agents_codes(&v, Mode::Contract).contains(&IssueCode::UnsafeFlag),
            "{pointer} trebuie respins"
        );
    }
}

#[test]
fn agent_on_disabled_provider_is_rejected() {
    let mut v = common::agents_value();
    v["agents"][0]["provider"] = json!("anthropic_api");
    assert!(agents_codes(&v, Mode::Contract).contains(&IssueCode::ProviderDisabled));
}

#[test]
fn agents_cannot_hold_executor_tools_and_contracts_match_roles() {
    let mut v = common::agents_value();
    v["agents"][2]["tools"] = json!(["project.read", "project.apply_validated_patch"]);
    assert!(agents_codes(&v, Mode::Contract).contains(&IssueCode::AgentToolNotAllowed));

    let mut v = common::agents_value();
    v["agents"][0]["output_contract"] = json!("patch_proposal");
    assert!(agents_codes(&v, Mode::Contract).contains(&IssueCode::ContractMismatch));
}

#[test]
fn paid_provider_with_zero_budget_is_inconsistent() {
    let mut v = common::agents_value();
    v["providers"]["anthropic_api"]["enabled"] = json!(true);
    v["providers"]["anthropic_api"]["model"] = json!("model-ales-explicit");
    assert!(agents_codes(&v, Mode::Contract).contains(&IssueCode::Inconsistent));
}

#[test]
fn ollama_endpoint_must_be_local() {
    for endpoint in [
        "http://192.168.1.10:11434",
        "https://ollama.com",
        "http://0.0.0.0:11434",
    ] {
        let mut v = common::agents_value();
        v["providers"]["ollama"]["endpoint"] = json!(endpoint);
        assert!(
            agents_codes(&v, Mode::Contract).contains(&IssueCode::InvalidValue),
            "{endpoint}"
        );
    }
}

#[test]
fn research_hosts_must_be_public_hostnames() {
    for host in [
        "localhost",
        "169.254.169.254",
        "https://create.roblox.com",
        "intranet",
    ] {
        let mut v = common::agents_value();
        v["policy"]["research_allowed_hosts"] = json!([host]);
        assert!(
            agents_codes(&v, Mode::Contract).contains(&IssueCode::InvalidValue),
            "{host}"
        );
    }
}

#[test]
fn single_job_per_pc() {
    let mut v = common::agents_value();
    v["runtime"]["max_concurrent_tasks"] = json!(2);
    assert!(agents_codes(&v, Mode::Contract).contains(&IssueCode::InvalidValue));
}

#[test]
fn execution_roots_must_be_local_absolute_paths() {
    let v = common::exec_config_value("\\\\nas\\share\\proiect");
    assert!(agents_codes(&v, Mode::Execution).contains(&IssueCode::InvalidValue));
}

#[test]
fn collaboration_wildcard_bind_and_unsafe_flags_are_rejected() {
    for ip in ["0.0.0.0", "::"] {
        let mut v = collab_value();
        v["collaboration"]["host_bind_ip"] = json!(ip);
        assert!(
            collab_codes(&v, Mode::Contract).contains(&IssueCode::UnsafeFlag),
            "{ip}"
        );
    }
    let flags = [
        ("/collaboration/allow_wildcard_bind", json!(true)),
        ("/collaboration/auto_host_failover", json!(true)),
        (
            "/collaboration/require_host_fingerprint_verification",
            json!(false),
        ),
        ("/device_worker/require_local_consent", json!(false)),
        ("/device_worker/expose_local_mcp_or_ollama", json!(true)),
        ("/attachments/execute_attachments", json!(true)),
        ("/blender_mcp/allow_arbitrary_python", json!(true)),
        ("/chat/default_visibility", json!("shared")),
    ];
    for (pointer, value) in flags {
        let mut v = collab_value();
        *v.pointer_mut(pointer).unwrap() = value;
        assert!(
            collab_codes(&v, Mode::Contract).contains(&IssueCode::UnsafeFlag),
            "{pointer}"
        );
    }
}

#[test]
fn pairing_code_ttl_is_at_most_five_minutes() {
    let mut v = collab_value();
    v["collaboration"]["pairing_code_ttl_seconds"] = json!(600);
    assert!(collab_codes(&v, Mode::Contract).contains(&IssueCode::InvalidValue));
}

#[test]
fn unimplemented_features_cannot_be_enabled_for_execution() {
    let mut v = collab_value();
    v["example_only"] = json!(false);
    v["collaboration"]["enabled"] = json!(true);
    v["collaboration"]["mode"] = json!("host");
    v["collaboration"]["host_bind_ip"] = json!("26.14.82.7");
    v["collaboration"]["host_port"] = json!(47820);
    v["blender_mcp"]["enabled"] = json!(true);
    v["device_worker"]["accept_remote_tasks"] = json!(true);
    assert!(collab_codes(&v, Mode::Contract).is_empty());
    let exec = collab_codes(&v, Mode::Execution);
    assert_eq!(
        exec.iter()
            .filter(|c| **c == IssueCode::FeatureNotAvailable)
            .count(),
        3
    );
}
