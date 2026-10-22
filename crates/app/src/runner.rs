//! Execuția unui plan aprobat (M1: în proces, pe un fir separat).
//!
//! Fiecare pas este real: propunerea furnizorului fals trece prin policy gate și
//! executor, fișierele se scriu în copia proiectului, verificările statice
//! citesc fișierele scrise. Doar ritmul (pauzele dintre pași) este ales pentru
//! demo. Oprirea, pauza și revocarea aprobării sunt verificate înaintea
//! fiecărui efect.

use std::collections::HashMap;
use std::sync::atomic::Ordering;

use rab_core::approval::{Approval, ApprovalCheck};
use rab_core::executor::{ExecError, FakeExecutor, TextPatch};
use rab_core::ids::ProviderId;
use rab_core::plan::{Plan, sha256_hex};
use rab_core::policy::{BudgetLedger, PolicyContext};
use rab_core::provider::{FakeProvider, GenerateRequest, GuardedProvider};
use rab_core::state::{Resolution, Run, RunStatus};
use serde_json::json;

use crate::checks::{self, CheckOutcome};
use crate::demo;
use crate::diff::file_diff;
use crate::dto::{RunResults, TestResult};
use crate::service::{AppError, AppResult, Inner, RunControl, now_iso};

pub const REPORT_PATH: &str = "reports/static-checks.md";

enum TaskError {
    /// Oprire cerută (utilizator sau aprobare revocată); efectele deja produse rămân.
    Stopped(String),
    Failed(String),
}

#[derive(serde::Deserialize)]
struct Proposal {
    contract: String,
    files: Vec<ProposedFile>,
}

#[derive(serde::Deserialize)]
struct ProposedFile {
    path: String,
    content: String,
}

fn save(inner: &Inner, run_id: &str, run: &Run) {
    let _ = inner.store.save_run(run_id, run, &now_iso());
}

fn task_title(plan: &Plan, id: &str) -> String {
    plan.task(id)
        .and_then(|t| t.title.clone())
        .unwrap_or_else(|| id.to_string())
}

fn approval_valid(inner: &Inner, approval_id: &str, plan: &Plan) -> Option<Approval> {
    match inner.store.approval_by_id(approval_id) {
        Ok(Some(a)) if a.check(plan) == ApprovalCheck::Valid => Some(a),
        _ => None,
    }
}

pub(crate) fn execute(inner: &Inner, run_id: &str, control: &RunControl) {
    let Ok(Some(row)) = inner.store.run(run_id) else {
        return;
    };
    let Ok(Some(stored)) = inner.store.plan(&row.plan_id, row.plan_version) else {
        return;
    };
    let plan = stored.plan;
    let mut run = row.run;
    if matches!(run.status, RunStatus::Queued | RunStatus::Paused) {
        let resumed = run.status == RunStatus::Paused;
        if run.transition(RunStatus::Running).is_err() {
            return;
        }
        save(inner, run_id, &run);
        let message = if resumed {
            "Rulare reluată"
        } else {
            "Rulare pornită"
        };
        inner.event(run_id, "running", "info", message, json!({}));
    }

    loop {
        if control.cancel.load(Ordering::SeqCst) {
            stop(
                inner,
                run_id,
                &mut run,
                "Rulare oprită de utilizator. Niciun pas nou nu mai pornește.",
            );
            break;
        }
        if wait_if_paused(inner, run_id, &mut run, control) {
            continue;
        }
        let Some(approval) = approval_valid(inner, &row.approval_id, &plan) else {
            stop(
                inner,
                run_id,
                &mut run,
                "Aprobarea a fost revocată sau planul s-a schimbat; nu se mai produc efecte.",
            );
            break;
        };
        let Some(task_id) = run.ready_tasks().into_iter().next() else {
            break;
        };
        let attempt = match run.start_task(&task_id) {
            Ok(a) => a,
            Err(e) => {
                inner.event(run_id, "error", "error", &e.to_string(), json!({}));
                break;
            }
        };
        save(inner, run_id, &run);
        let _ = inner
            .store
            .start_attempt(run_id, &task_id, attempt, &now_iso());
        let title = task_title(&plan, &task_id);
        inner.event(
            run_id,
            "task_started",
            "info",
            &format!("Pornește: {title} (încercarea {attempt})"),
            json!({ "task": task_id }),
        );

        let outcome = match task_id.as_str() {
            "generate" => generate(inner, run_id, &plan, &approval, control),
            "verify" => verify(inner, run_id, &plan, &approval, control),
            other => Err(TaskError::Failed(format!("sarcină necunoscută: {other}"))),
        };
        match outcome {
            Ok(receipt) => {
                let _ = run.complete_task(&task_id, &receipt);
                let _ = inner.store.finish_attempt(
                    run_id,
                    &task_id,
                    attempt,
                    "succeeded",
                    Some(&receipt),
                    &now_iso(),
                );
                inner.event(
                    run_id,
                    "task_succeeded",
                    "ok",
                    &format!("Gata: {title}"),
                    json!({ "task": task_id }),
                );
            }
            Err(TaskError::Stopped(reason)) => {
                let _ = inner.store.finish_attempt(
                    run_id,
                    &task_id,
                    attempt,
                    "stopped",
                    None,
                    &now_iso(),
                );
                stop(inner, run_id, &mut run, &reason);
                break;
            }
            Err(TaskError::Failed(message)) => {
                let _ = run.fail_task(&task_id);
                let _ = inner.store.finish_attempt(
                    run_id,
                    &task_id,
                    attempt,
                    "failed",
                    None,
                    &now_iso(),
                );
                inner.event(
                    run_id,
                    "task_failed",
                    "error",
                    &message,
                    json!({ "task": task_id }),
                );
            }
        }
        save(inner, run_id, &run);
        if run.status.is_terminal() {
            break;
        }
    }
    if run.status.is_terminal() {
        let _ = finalize(inner, run_id, &run);
    }
}

fn stop(inner: &Inner, run_id: &str, run: &mut Run, reason: &str) {
    if run.transition(RunStatus::Cancelled).is_ok() {
        save(inner, run_id, run);
    }
    inner.event(run_id, "cancelled", "warn", reason, json!({}));
}

/// Întoarce `true` dacă rularea a stat în pauză (bucla reia verificările).
fn wait_if_paused(inner: &Inner, run_id: &str, run: &mut Run, control: &RunControl) -> bool {
    let mut paused = control
        .paused
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if !*paused {
        return false;
    }
    if run.transition(RunStatus::Paused).is_ok() {
        save(inner, run_id, run);
        inner.event(
            run_id,
            "paused",
            "info",
            "În pauză. Niciun pas nou nu pornește.",
            json!({}),
        );
    }
    while *paused && !control.cancel.load(Ordering::SeqCst) {
        paused = control
            .cv
            .wait(paused)
            .unwrap_or_else(std::sync::PoisonError::into_inner);
    }
    drop(paused);
    if !control.cancel.load(Ordering::SeqCst) && run.transition(RunStatus::Running).is_ok() {
        save(inner, run_id, run);
        inner.event(run_id, "running", "info", "Rulare reluată", json!({}));
    }
    true
}

fn interrupted(
    inner: &Inner,
    run_id: &str,
    plan: &Plan,
    control: &RunControl,
) -> Option<TaskError> {
    if control.sleep(inner.pacing.step_ms) {
        return Some(TaskError::Stopped(
            "Rulare oprită de utilizator în timpul pasului.".into(),
        ));
    }
    let row = inner.store.run(run_id).ok().flatten()?;
    if approval_valid(inner, &row.approval_id, plan).is_none() {
        return Some(TaskError::Stopped(
            "Aprobarea a fost revocată în timpul pasului; efectele deja produse rămân, altele nu mai pornesc.".into(),
        ));
    }
    None
}

fn flush_audit(inner: &Inner, run_id: &str, exec: &FakeExecutor<'_>) {
    for event in exec.audit() {
        let kind = match event {
            rab_core::executor::AuditEvent::Refused { .. } => "refused",
            rab_core::executor::AuditEvent::Intent { .. } => "intent",
            rab_core::executor::AuditEvent::Effect { .. } => "effect",
            rab_core::executor::AuditEvent::Conflict { .. } => "conflict",
        };
        let detail = serde_json::to_string(event).unwrap_or_default();
        let _ = inner
            .store
            .append_audit(Some(run_id), kind, &detail, &now_iso());
    }
}

fn generate(
    inner: &Inner,
    run_id: &str,
    plan: &Plan,
    approval: &Approval,
    control: &RunControl,
) -> Result<String, TaskError> {
    let config = &inner.config;
    let limit = plan.budget.max_usd.min(config.budget.max_run_usd);
    let ledger = BudgetLedger::new(limit);
    let fake = FakeProvider::new(ProviderId::Ollama, [demo::proposal_json()]);
    let guarded = GuardedProvider {
        config,
        ledger: &ledger,
        provider: &fake,
    };
    let response = guarded
        .generate(
            Some(&plan.budget),
            Some(0.0),
            &GenerateRequest {
                model: plan.model.clone(),
                prompt: plan.objective.clone(),
                max_output_tokens: config.providers.ollama.max_output_tokens,
            },
        )
        .map_err(|e| TaskError::Failed(format!("Furnizorul a fost refuzat: {e}")))?;
    let reservation = inner
        .store
        .reserve_budget(Some(run_id), response.cost_usd, limit, &now_iso())
        .map_err(|e| TaskError::Failed(e.to_string()))?;
    let _ = inner.store.commit_budget(reservation, response.cost_usd);

    let proposal: Proposal = serde_json::from_str(&response.text)
        .map_err(|e| TaskError::Failed(format!("Propunere invalidă: {e}")))?;
    if proposal.contract != "patch_proposal"
        || proposal.files.is_empty()
        || proposal.files.len() > 20
    {
        return Err(TaskError::Failed(
            "Propunerea nu respectă contractul patch_proposal".into(),
        ));
    }
    inner.event(
        run_id,
        "proposal",
        "info",
        &format!(
            "Constructorul a propus {} fișiere (furnizor demonstrativ, cost 0,00 $)",
            proposal.files.len()
        ),
        json!({}),
    );

    let ctx = PolicyContext {
        config,
        plan,
        approval: Some(approval),
        cancelled: false,
    };
    let mut exec = FakeExecutor::new(ctx, &inner.project_root)
        .map_err(|e| TaskError::Failed(e.to_string()))?;
    let mut hashes = Vec::new();
    let mut result = Ok(());
    for file in &proposal.files {
        if let Some(stop) = interrupted(inner, run_id, plan, control) {
            exec.cancel();
            result = Err(stop);
            break;
        }
        let target = sha256_hex(file.content.as_bytes());
        if std::fs::read(inner.project_root.join(&file.path))
            .ok()
            .is_some_and(|b| sha256_hex(&b) == target)
        {
            inner.event(
                run_id,
                "file_skipped",
                "info",
                &format!("Deja aplicat: {}", file.path),
                json!({}),
            );
            hashes.push(target);
            continue;
        }
        let patch = TextPatch {
            path: file.path.clone(),
            expected_base_sha256: None,
            new_content: file.content.clone(),
        };
        match exec.apply_validated_patch("generate", &patch) {
            Ok(receipt) => {
                let _ = inner.store.record_file_change(
                    run_id,
                    &receipt.path,
                    receipt.previous_content.as_deref(),
                    &file.content,
                );
                inner.event(
                    run_id,
                    "file_written",
                    "ok",
                    &format!(
                        "Scris {} · {} linii",
                        receipt.path,
                        file.content.lines().count()
                    ),
                    json!({ "path": receipt.path }),
                );
                hashes.push(target);
            }
            Err(ExecError::BaseConflict) => {
                result = Err(TaskError::Failed(format!(
                    "{} există deja cu alt conținut; nu îl suprascriu fără o decizie a ta.",
                    file.path
                )));
                break;
            }
            Err(e) => {
                result = Err(TaskError::Failed(format!("{}: {e}", file.path)));
                break;
            }
        }
    }
    flush_audit(inner, run_id, &exec);
    result.map(|()| sha256_hex(hashes.join("").as_bytes()))
}

fn verify(
    inner: &Inner,
    run_id: &str,
    plan: &Plan,
    approval: &Approval,
    control: &RunControl,
) -> Result<String, TaskError> {
    if let Some(stop) = interrupted(inner, run_id, plan, control) {
        return Err(stop);
    }
    let root = &inner.project_root;
    let changed: Vec<String> = inner
        .store
        .file_changes(run_id)
        .map_err(|e| TaskError::Failed(e.to_string()))?
        .into_iter()
        .map(|c| c.path)
        .collect();
    let results: Vec<(&str, CheckOutcome)> = vec![
        (demo::TEST_ROJO, checks::rojo_structure(root)),
        (demo::TEST_REQUIRE, checks::no_external_require(root)),
        (
            demo::TEST_SCOPE,
            checks::changes_in_scope(&changed, &plan.write_scopes()),
        ),
    ];
    for (title, outcome) in &results {
        inner.event(
            run_id,
            "check",
            if outcome.passed { "ok" } else { "error" },
            &format!(
                "Verificare „{title}”: {} · {}",
                if outcome.passed {
                    "trecută"
                } else {
                    "eșuată"
                },
                outcome.note
            ),
            json!({ "title": title, "passed": outcome.passed, "note": outcome.note }),
        );
    }
    if let Some(stop) = interrupted(inner, run_id, plan, control) {
        return Err(stop);
    }

    let report = report_markdown(plan, &results);
    let ctx = PolicyContext {
        config: &inner.config,
        plan,
        approval: Some(approval),
        cancelled: false,
    };
    let mut exec = FakeExecutor::new(ctx, root).map_err(|e| TaskError::Failed(e.to_string()))?;
    let current = std::fs::read(root.join(REPORT_PATH))
        .ok()
        .map(|b| sha256_hex(&b));
    let written = exec.write_report(
        "verify",
        &TextPatch {
            path: REPORT_PATH.into(),
            expected_base_sha256: current,
            new_content: report.clone(),
        },
    );
    flush_audit(inner, run_id, &exec);
    let receipt =
        written.map_err(|e| TaskError::Failed(format!("Raportul nu s-a putut scrie: {e}")))?;
    let _ = inner.store.record_file_change(
        run_id,
        REPORT_PATH,
        receipt.previous_content.as_deref(),
        &report,
    );
    inner.event(
        run_id,
        "file_written",
        "ok",
        &format!("Scris {REPORT_PATH}"),
        json!({ "path": REPORT_PATH }),
    );
    Ok(receipt.sha256)
}

fn report_markdown(plan: &Plan, results: &[(&str, CheckOutcome)]) -> String {
    let mut out = format!(
        "# Raport verificări statice\n\nPlan: {} v{} · sha256 `{}`\n\n| Test | Rezultat | Detalii |\n|---|---|---|\n",
        plan.title.clone().unwrap_or_else(|| plan.id.clone()),
        plan.version,
        plan.sha256()
    );
    for (title, outcome) in results {
        out.push_str(&format!(
            "| {title} | {} | {} |\n",
            if outcome.passed {
                "✓ trecut"
            } else {
                "✕ eșuat"
            },
            outcome.note
        ));
    }
    for title in plan
        .tasks
        .iter()
        .flat_map(|t| &t.acceptance)
        .filter(|t| !demo::is_static_test(t))
    {
        out.push_str(&format!(
            "| {title} | – neexecutat | Necesită Roblox Studio (etapa M4) |\n"
        ));
    }
    out.push_str("\nCodul a fost propus de furnizorul demonstrativ (răspuns prestabilit), nu de un model real.\n");
    out
}

/// Construiește și salvează rezultatele finale ale rulării.
pub(crate) fn finalize(inner: &Inner, run_id: &str, run: &Run) -> AppResult<()> {
    let store = &inner.store;
    let row = store
        .run(run_id)?
        .ok_or_else(|| AppError::NotFound(run_id.into()))?;
    let plan = store
        .plan(&row.plan_id, row.plan_version)?
        .ok_or_else(|| AppError::NotFound(row.plan_id.clone()))?
        .plan;
    let mut checks: HashMap<String, (bool, String)> = HashMap::new();
    for event in store.events_after(run_id, 0, 10_000)? {
        if event.kind == "check" {
            let payload: serde_json::Value =
                serde_json::from_str(&event.payload_json).unwrap_or_default();
            if let Some(title) = payload["title"].as_str() {
                checks.insert(
                    title.to_string(),
                    (
                        payload["passed"].as_bool().unwrap_or(false),
                        payload["note"].as_str().unwrap_or_default().to_string(),
                    ),
                );
            }
        }
    }
    let tests: Vec<TestResult> = plan
        .tasks
        .iter()
        .flat_map(|t| &t.acceptance)
        .enumerate()
        .map(|(i, title)| {
            let (status, note) = if demo::is_static_test(title) {
                match checks.get(title) {
                    Some((true, note)) => ("passed", note.clone()),
                    Some((false, note)) => ("failed", note.clone()),
                    None => (
                        "not_run",
                        "Verificarea nu a rulat: rularea s-a oprit înainte.".into(),
                    ),
                }
            } else {
                (
                    "not_run",
                    "Necesită Roblox Studio; playtestul vine în etapa M4.".into(),
                )
            };
            TestResult {
                id: format!("t{}", i + 1),
                title: title.clone(),
                status: status.into(),
                note,
            }
        })
        .collect();
    let any_failed = tests.iter().any(|t| t.status == "failed");
    let all_passed = tests.iter().all(|t| t.status == "passed");
    let verdict = match run.status {
        RunStatus::Cancelled => "cancelled",
        RunStatus::Failed => "failed",
        _ if any_failed => "failed",
        _ if all_passed => "passed",
        _ => "needs_review",
    };
    let changes: Vec<_> = store
        .file_changes(run_id)?
        .iter()
        .map(|c| file_diff(&c.path, c.before_text.as_deref(), &c.after_text))
        .collect();
    let duration = parse_iso(&row.created_at)
        .map(|start| {
            (time::OffsetDateTime::now_utc() - start)
                .whole_seconds()
                .max(0)
        })
        .unwrap_or(0);
    let results = RunResults {
        verdict: verdict.into(),
        tests,
        limitations: vec![
            "Playtestul în Roblox Studio nu a fost executat: integrarea Studio vine în etapa M4.".into(),
            "Codul a fost propus de un furnizor demonstrativ (răspuns prestabilit), nu de un model real.".into(),
            "Aprobarea folosește numele utilizatorului de sistem, fără autentificare; legarea de contul Windows vine în M2."
                .into(),
        ],
        duration_seconds: duration,
        cost_usd: 0.0,
        report_path: changes.iter().any(|c| c.path == REPORT_PATH).then(|| REPORT_PATH.to_string()),
        changes,
    };
    store.save_results(run_id, &serde_json::to_string(&results)?)?;
    let label = match verdict {
        "passed" => "Finalizat: toate testele au trecut",
        "needs_review" => "Necesită verificare: unele teste nu au fost executate",
        "failed" => "Eșuat: cel puțin un test sau pas a eșuat",
        _ => "Rulare oprită; rezultatele parțiale sunt salvate",
    };
    inner.event(
        run_id,
        "finished",
        if verdict == "passed" { "ok" } else { "warn" },
        label,
        json!({ "verdict": verdict }),
    );
    Ok(())
}

fn parse_iso(value: &str) -> Option<time::OffsetDateTime> {
    time::OffsetDateTime::parse(value, &time::format_description::well_known::Rfc3339).ok()
}

/// Reconcilierea unui pas incert: caută efectul pe disc în loc să-l repete orb.
pub(crate) fn inspect_effect(inner: &Inner, task_id: &str) -> Resolution {
    if task_id != "generate" {
        // Verificările și raportul sunt idempotente: se pot relua.
        return Resolution::EffectAbsent;
    }
    let mut hashes = Vec::new();
    for (path, content) in demo::PROPOSAL {
        let target = sha256_hex(content.as_bytes());
        match std::fs::read(inner.project_root.join(path)) {
            Ok(bytes) if sha256_hex(&bytes) == target => hashes.push(target),
            _ => return Resolution::EffectAbsent,
        }
    }
    Resolution::EffectConfirmed {
        receipt_sha256: sha256_hex(hashes.join("").as_bytes()),
    }
}
