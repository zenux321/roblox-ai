//! Furnizori, buget, mașina de stări și executorul pe fixture (S06, S07, S11/S12 parțial, S20).
#![allow(clippy::unwrap_used)]

mod common;

use std::fs;
use std::sync::Arc;

use common::{APPROVER, NOW, exec_config, exec_config_value, exec_plan};
use rab_core::approval::Approval;
use rab_core::config::{AgentsConfig, Mode};
use rab_core::executor::{AuditEvent, ExecError, FakeExecutor, TextPatch};
use rab_core::ids::ProviderId;
use rab_core::plan::{PlanStatus, sha256_hex};
use rab_core::policy::{BudgetLedger, Denial, PolicyContext, is_ollama_cloud_model};
use rab_core::provider::{FakeProvider, GenerateRequest, GuardedProvider, ProviderError};
use rab_core::state::{
    Resolution, Run, RunError, RunStatus, TaskState, plan_transition_allowed,
    run_transition_allowed,
};
use serde_json::json;

const ROOT: &str = "D:/Roblox/ObbyVulcanic";

fn gen_req(model: &str) -> GenerateRequest {
    GenerateRequest {
        model: model.into(),
        prompt: "Explică checkpointul".into(),
        max_output_tokens: 256,
    }
}

#[test]
fn s06_disabled_or_zero_budget_paid_provider_makes_zero_calls() {
    let config = exec_config(ROOT);
    let ledger = BudgetLedger::new(config.budget.max_run_usd);
    let paid =
        FakeProvider::new(ProviderId::AnthropicApi, ["nu trebuie".to_string()]).with_cost(0.02);
    let guarded = GuardedProvider {
        config: &config,
        ledger: &ledger,
        provider: &paid,
    };
    assert_eq!(
        guarded.generate(None, Some(0.02), &gen_req("model-api")),
        Err(ProviderError::Denied(Denial::ProviderDisabled(
            ProviderId::AnthropicApi
        )))
    );

    // Furnizor activ, dar apelurile plătite sunt oprite / buget 0.
    let mut v = exec_config_value(ROOT);
    v["providers"]["anthropic_api"]["enabled"] = json!(true);
    v["providers"]["anthropic_api"]["model"] = json!("model-api");
    let enabled = AgentsConfig::parse(&v.to_string()).unwrap();
    let guarded = GuardedProvider {
        config: &enabled,
        ledger: &ledger,
        provider: &paid,
    };
    assert_eq!(
        guarded.generate(None, Some(0.02), &gen_req("model-api")),
        Err(ProviderError::Denied(Denial::PaidCallsDisabled))
    );
    v["policy"]["allow_paid_calls"] = json!(true);
    let zero = AgentsConfig::parse(&v.to_string()).unwrap();
    let guarded = GuardedProvider {
        config: &zero,
        ledger: &ledger,
        provider: &paid,
    };
    assert_eq!(
        guarded.generate(None, Some(0.02), &gen_req("model-api")),
        Err(ProviderError::Denied(Denial::ZeroBudget))
    );
    assert_eq!(
        paid.calls(),
        0,
        "spy: niciun apel nu a ajuns la furnizorul plătit"
    );
}

#[test]
fn unknown_price_and_plan_budget_are_enforced() {
    let mut v = exec_config_value(ROOT);
    v["providers"]["anthropic_api"]["enabled"] = json!(true);
    v["providers"]["anthropic_api"]["model"] = json!("model-api");
    v["policy"]["allow_paid_calls"] = json!(true);
    v["budget"]["max_run_usd"] = json!(1.0);
    v["budget"]["max_month_usd"] = json!(5.0);
    let config = AgentsConfig::load(&v.to_string(), Mode::Execution).unwrap();
    let ledger = BudgetLedger::new(1.0);
    let paid = FakeProvider::new(ProviderId::AnthropicApi, ["ok".to_string()]).with_cost(0.02);
    let guarded = GuardedProvider {
        config: &config,
        ledger: &ledger,
        provider: &paid,
    };
    assert_eq!(
        guarded.generate(None, None, &gen_req("model-api")),
        Err(ProviderError::Denied(Denial::UnknownPrice))
    );
    let plan = exec_plan(ROOT); // max_usd = 0 în plan
    assert_eq!(
        guarded.generate(Some(&plan.budget), Some(0.02), &gen_req("model-api")),
        Err(ProviderError::Denied(Denial::ZeroBudget))
    );
    assert_eq!(paid.calls(), 0);
    let ok = guarded
        .generate(None, Some(0.02), &gen_req("model-api"))
        .unwrap();
    assert_eq!(ok.text, "ok");
    assert!((ledger.committed_usd() - 0.02).abs() < 1e-9);
}

#[test]
fn local_provider_is_free_but_cloud_models_are_blocked() {
    let config = exec_config(ROOT);
    let ledger = BudgetLedger::new(0.0);
    let local = FakeProvider::new(
        ProviderId::Ollama,
        ["Serverul e sursa de adevăr.".to_string()],
    );
    let guarded = GuardedProvider {
        config: &config,
        ledger: &ledger,
        provider: &local,
    };
    assert_eq!(
        guarded.generate(None, None, &gen_req("gpt-oss:120b-cloud")),
        Err(ProviderError::Denied(Denial::CloudModelBlocked(
            "gpt-oss:120b-cloud".into()
        )))
    );
    assert_eq!(local.calls(), 0);
    let response = guarded
        .generate(None, None, &gen_req("qwen2.5-coder:7b"))
        .unwrap();
    assert_eq!(response.cost_usd, 0.0);
    assert!(is_ollama_cloud_model("x:cloud"));
    assert!(!is_ollama_cloud_model("qwen2.5-coder:7b"));
}

#[test]
fn s07_concurrent_reservations_never_exceed_budget() {
    let ledger = Arc::new(BudgetLedger::new(1.0));
    let handles: Vec<_> = (0..16)
        .map(|_| {
            let ledger = Arc::clone(&ledger);
            std::thread::spawn(move || ledger.reserve(0.3).is_ok())
        })
        .collect();
    let granted = handles
        .into_iter()
        .map(|h| h.join().unwrap())
        .filter(|ok| *ok)
        .count();
    assert_eq!(granted, 3, "doar 3 × 0,30 $ încap în 1,00 $");
    assert!(ledger.reserved_usd() <= 1.0 + 1e-9);
}

#[test]
fn s07_uncertain_reservation_is_not_released_automatically() {
    let ledger = BudgetLedger::new(1.0);
    let id = ledger.reserve(0.8).unwrap();
    ledger.mark_uncertain(id);
    assert!(!ledger.release(id), "costul este „în curs de verificare”");
    assert_eq!(ledger.reserve(0.5), Err(Denial::BudgetExceeded));
    ledger.commit(id, 0.8);
    assert!((ledger.committed_usd() - 0.8).abs() < 1e-9);
    assert_eq!(
        BudgetLedger::new(0.0).reserve(0.01),
        Err(Denial::ZeroBudget)
    );
}

#[test]
fn transitions_follow_the_architecture() {
    assert!(plan_transition_allowed(
        PlanStatus::Draft,
        PlanStatus::AwaitingApproval
    ));
    assert!(plan_transition_allowed(
        PlanStatus::Approved,
        PlanStatus::AwaitingApproval
    ));
    assert!(!plan_transition_allowed(
        PlanStatus::Draft,
        PlanStatus::Approved
    ));
    assert!(!plan_transition_allowed(
        PlanStatus::Superseded,
        PlanStatus::Approved
    ));
    assert!(run_transition_allowed(
        RunStatus::Running,
        RunStatus::RecoveryRequired
    ));
    assert!(!run_transition_allowed(
        RunStatus::Succeeded,
        RunStatus::Running
    ));
    assert!(!run_transition_allowed(
        RunStatus::Cancelled,
        RunStatus::Running
    ));
    assert!(!run_transition_allowed(
        RunStatus::Queued,
        RunStatus::Succeeded
    ));
}

#[test]
fn succeeded_task_is_never_rerun_and_dependencies_are_respected() {
    let plan = exec_plan(ROOT);
    let mut run = Run::new(&plan);
    assert_eq!(run.ready_tasks(), vec!["generate".to_string()]);
    assert_eq!(
        run.start_task("verify"),
        Err(RunError::DependenciesPending("verify".into()))
    );
    assert_eq!(run.start_task("generate"), Ok(1));
    run.complete_task("generate", &"a".repeat(64)).unwrap();
    run.on_ui_reconnect();
    assert_eq!(
        run.start_task("generate"),
        Err(RunError::AlreadySucceeded("generate".into()))
    );
    assert_eq!(run.ready_tasks(), vec!["verify".to_string()]);
    run.start_task("verify").unwrap();
    run.complete_task("verify", &"b".repeat(64)).unwrap();
    assert_eq!(run.status, RunStatus::Succeeded);
    assert!(run.transition(RunStatus::Running).is_err());
}

#[test]
fn retries_are_bounded() {
    let plan = exec_plan(ROOT); // max_retries_per_task = 2
    let mut run = Run::new(&plan);
    for attempt in 1..=3 {
        assert_eq!(run.start_task("generate"), Ok(attempt));
        run.fail_task("generate").unwrap();
    }
    assert_eq!(run.status, RunStatus::Failed);
    assert!(run.start_task("generate").is_err());
}

#[test]
fn s11_s12_restart_marks_uncertain_effects_for_reconciliation() {
    let plan = exec_plan(ROOT);
    let mut run = Run::new(&plan);
    run.start_task("generate").unwrap();
    // Crash după efect, înainte de receipt.
    run.reconcile_after_restart();
    assert_eq!(run.status, RunStatus::RecoveryRequired);
    assert_eq!(
        run.task_state("generate"),
        Some(&TaskState::RecoveryRequired { attempt: 1 })
    );
    assert_eq!(
        run.start_task("generate"),
        Err(RunError::WrongRunStatus(RunStatus::RecoveryRequired))
    );
    // Reconcilierea găsește efectul: nu se repetă.
    run.resolve_recovery(
        "generate",
        Resolution::EffectConfirmed {
            receipt_sha256: "c".repeat(64),
        },
    )
    .unwrap();
    assert_eq!(run.status, RunStatus::Running);
    assert!(matches!(
        run.task_state("generate"),
        Some(TaskState::Succeeded { .. })
    ));
    assert_eq!(run.ready_tasks(), vec!["verify".to_string()]);
}

#[test]
fn executor_applies_patch_only_after_policy_gate() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_str().unwrap().to_string();
    let config = exec_config(&root);
    let plan = exec_plan(&root);
    let approval = Approval::grant(&plan, APPROVER, NOW).unwrap();
    let ctx = PolicyContext {
        config: &config,
        plan: &plan,
        approval: Some(&approval),
        cancelled: false,
    };
    let mut exec = FakeExecutor::new(ctx, dir.path()).unwrap();

    let content = "local Checkpoints = {}\nreturn Checkpoints\n";
    let receipt = exec
        .apply_validated_patch(
            "generate",
            &TextPatch {
                path: "src/server/Checkpoints.luau".into(),
                expected_base_sha256: None,
                new_content: content.into(),
            },
        )
        .unwrap();
    let written = fs::read_to_string(dir.path().join("src/server/Checkpoints.luau")).unwrap();
    assert_eq!(written, content);
    assert_eq!(receipt.sha256, sha256_hex(content.as_bytes()));

    // Baza s-a schimbat (utilizatorul a editat): conflict, fără suprascriere.
    let stale = exec.apply_validated_patch(
        "generate",
        &TextPatch {
            path: "src/server/Checkpoints.luau".into(),
            expected_base_sha256: Some("0".repeat(64)),
            new_content: "-- suprascris\n".into(),
        },
    );
    assert!(matches!(stale, Err(ExecError::BaseConflict)));
    assert_eq!(
        fs::read_to_string(dir.path().join("src/server/Checkpoints.luau")).unwrap(),
        content
    );

    // În afara scope-ului: refuz auditat, nimic scris.
    let outside = exec.apply_validated_patch(
        "generate",
        &TextPatch {
            path: "../evadare.txt".into(),
            expected_base_sha256: None,
            new_content: "x".into(),
        },
    );
    assert!(matches!(outside, Err(ExecError::Denied(_))));
    assert!(!dir.path().parent().unwrap().join("evadare.txt").exists());
    assert!(
        exec.audit()
            .iter()
            .any(|e| matches!(e, AuditEvent::Refused { .. }))
    );
    assert!(
        exec.audit()
            .iter()
            .any(|e| matches!(e, AuditEvent::Conflict { .. }))
    );

    // S20: după anulare nu mai există efecte noi.
    exec.cancel();
    let after_cancel = exec.apply_validated_patch(
        "generate",
        &TextPatch {
            path: "src/after.luau".into(),
            expected_base_sha256: None,
            new_content: "x".into(),
        },
    );
    assert!(matches!(
        after_cancel,
        Err(ExecError::Denied(Denial::RunCancelled))
    ));
    assert!(!dir.path().join("src/after.luau").exists());
}

#[cfg(unix)]
#[test]
fn executor_refuses_symlinks_inside_project() {
    let project = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    std::os::unix::fs::symlink(outside.path(), project.path().join("src")).unwrap();
    let root = project.path().to_str().unwrap().to_string();
    let config = exec_config(&root);
    let plan = exec_plan(&root);
    let approval = Approval::grant(&plan, APPROVER, NOW).unwrap();
    let ctx = PolicyContext {
        config: &config,
        plan: &plan,
        approval: Some(&approval),
        cancelled: false,
    };
    let mut exec = FakeExecutor::new(ctx, project.path()).unwrap();
    let result = exec.apply_validated_patch(
        "generate",
        &TextPatch {
            path: "src/escape.luau".into(),
            expected_base_sha256: None,
            new_content: "x".into(),
        },
    );
    assert!(matches!(result, Err(ExecError::SymlinkInPath)));
    assert!(!outside.path().join("escape.luau").exists());
}

#[test]
fn executor_enforces_output_limit() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_str().unwrap().to_string();
    let mut v = exec_config_value(&root);
    v["runtime"]["max_tool_output_bytes"] = json!(16);
    let config = AgentsConfig::load(&v.to_string(), Mode::Execution).unwrap();
    let plan = exec_plan(&root);
    let approval = Approval::grant(&plan, APPROVER, NOW).unwrap();
    let ctx = PolicyContext {
        config: &config,
        plan: &plan,
        approval: Some(&approval),
        cancelled: false,
    };
    let mut exec = FakeExecutor::new(ctx, dir.path()).unwrap();
    let result = exec.apply_validated_patch(
        "generate",
        &TextPatch {
            path: "src/big.luau".into(),
            expected_base_sha256: None,
            new_content: "x".repeat(17),
        },
    );
    assert!(matches!(result, Err(ExecError::TooLarge(16))));
}

#[test]
fn executor_root_must_match_plan_root() {
    let planned = tempfile::tempdir().unwrap();
    let other = tempfile::tempdir().unwrap();
    let root = planned.path().to_str().unwrap().to_string();
    let config = exec_config(&root);
    let plan = exec_plan(&root);
    let approval = Approval::grant(&plan, APPROVER, NOW).unwrap();
    let ctx = PolicyContext {
        config: &config,
        plan: &plan,
        approval: Some(&approval),
        cancelled: false,
    };
    assert!(matches!(
        FakeExecutor::new(ctx, other.path()),
        Err(ExecError::RootMismatch)
    ));
}
