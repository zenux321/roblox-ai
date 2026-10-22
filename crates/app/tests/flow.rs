//! Fluxul M1 cap-coadă, fără rețea și fără chei: plan → aprobare → rulare →
//! rezultate, plus oprire, revocare, pauză, repornire și conversații.
#![allow(clippy::unwrap_used)]

use std::path::Path;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use rab_app::dto::{Message, ReplyEvent};
use rab_app::{AppError, AppService, NoopSink, Pacing, REPORT_PATH};
use rab_core::state::Run;
use rab_storage::{RunRow, Store};

const WAIT: Duration = Duration::from_secs(10);
const IDENTITY: &str = "PC-TEST\\andrei";

fn open(dir: &Path, pacing: Pacing) -> AppService {
    AppService::open(dir, IDENTITY.into(), pacing, Arc::new(NoopSink)).unwrap()
}

fn slow() -> Pacing {
    Pacing {
        step_ms: 150,
        ..Pacing::INSTANT
    }
}

fn approve_new_plan(service: &AppService) -> (String, u32) {
    let (_, plan) = service.prepare_plan("checkpoint").unwrap();
    let approved = service
        .approve_plan(&plan.id, plan.version, &plan.sha256)
        .unwrap();
    assert_eq!(approved.status, "approved");
    assert!(approved.approval.unwrap().valid);
    (plan.id, plan.version)
}

fn wait_for_event(service: &AppService, run_id: &str, kind: &str) {
    let deadline = std::time::Instant::now() + WAIT;
    while std::time::Instant::now() < deadline {
        if service
            .events(run_id, 0, 1000)
            .unwrap()
            .iter()
            .any(|e| e.kind == kind)
        {
            return;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    panic!("evenimentul {kind} nu a apărut");
}

fn count_events(service: &AppService, run_id: &str, kind: &str) -> usize {
    service
        .events(run_id, 0, 1000)
        .unwrap()
        .iter()
        .filter(|e| e.kind == kind)
        .count()
}

#[test]
fn end_to_end_demo_produces_files_report_and_honest_verdict() {
    let dir = tempfile::tempdir().unwrap();
    let service = open(dir.path(), Pacing::INSTANT);
    assert_eq!(
        service.list_conversations().unwrap().len(),
        4,
        "conversațiile demo sunt inițializate"
    );

    let (message, plan) = service.prepare_plan("checkpoint").unwrap();
    assert!(matches!(message, Message::Assistant { .. }));
    assert_eq!(plan.status, "awaiting_approval");
    assert_eq!(plan.tests.iter().filter(|t| t.kind == "studio").count(), 2);
    assert!(
        service.start_run(&plan.id, plan.version).is_err(),
        "fără aprobare nu pornește"
    );

    service
        .approve_plan(&plan.id, plan.version, &plan.sha256)
        .unwrap();
    let run = service.start_run(&plan.id, plan.version).unwrap();
    assert!(service.wait_until_idle(WAIT));

    let run = service.get_run(&run.id).unwrap();
    assert_eq!(run.status, "succeeded");
    assert_eq!((run.done, run.total), (2, 2));
    let results = service.results(&run.id).unwrap().unwrap();
    assert_eq!(
        results.verdict, "needs_review",
        "testele Studio nu au rulat, deci nu „Finalizat”"
    );
    let statuses: Vec<&str> = results.tests.iter().map(|t| t.status.as_str()).collect();
    assert_eq!(
        statuses,
        vec!["passed", "passed", "passed", "not_run", "not_run"]
    );
    assert_eq!(results.cost_usd, 0.0);
    assert_eq!(results.report_path.as_deref(), Some(REPORT_PATH));

    let root = service.project_root();
    assert!(root.join("src/server/Course.server.luau").exists());
    assert!(
        root.join("src/server/CheckpointService.server.luau")
            .exists()
    );
    let report = std::fs::read_to_string(root.join(REPORT_PATH)).unwrap();
    assert!(report.contains("neexecutat"));
    let paths: Vec<&str> = results.changes.iter().map(|c| c.path.as_str()).collect();
    assert!(paths.contains(&"src/server/Course.server.luau") && paths.contains(&REPORT_PATH));
    assert!(
        results
            .changes
            .iter()
            .all(|c| c.is_new && c.added > 0 && c.removed == 0)
    );

    let events = service.events(&run.id, 0, 1000).unwrap();
    assert!(events.windows(2).all(|w| w[1].seq > w[0].seq));
    let after = service.events(&run.id, events[2].seq, 1000).unwrap();
    assert_eq!(
        after.len(),
        events.len() - 3,
        "reluarea de la o secvență nu pierde și nu dublează"
    );
}

#[test]
fn approval_must_match_the_plan_shown() {
    let dir = tempfile::tempdir().unwrap();
    let service = open(dir.path(), Pacing::INSTANT);
    let (_, plan) = service.prepare_plan("checkpoint").unwrap();
    let wrong = "0".repeat(64);
    assert!(matches!(
        service.approve_plan(&plan.id, plan.version, &wrong),
        Err(AppError::Denied(_))
    ));
    // Un plan nou înlocuiește versiunea veche; aprobarea veche nu se aplică noii versiuni.
    let (_, v2) = service.prepare_plan("checkpoint").unwrap();
    assert_eq!(v2.version, plan.version + 1);
    assert!(
        service
            .approve_plan(&plan.id, plan.version, &plan.sha256)
            .is_err()
    );
}

#[test]
fn s02_revoking_during_a_run_stops_new_effects() {
    let dir = tempfile::tempdir().unwrap();
    let service = open(dir.path(), slow());
    let (id, version) = approve_new_plan(&service);
    let run = service.start_run(&id, version).unwrap();
    wait_for_event(&service, &run.id, "file_written");
    let plan = service.revoke_approval(&id, version).unwrap();
    assert_eq!(plan.status, "awaiting_approval");
    assert!(service.wait_until_idle(WAIT));

    let run = service.get_run(&run.id).unwrap();
    assert_eq!(run.status, "cancelled");
    assert_eq!(
        count_events(&service, &run.id, "file_written"),
        1,
        "al doilea fișier nu se mai scrie"
    );
    assert!(!service.project_root().join(REPORT_PATH).exists());
    assert_eq!(
        service.results(&run.id).unwrap().unwrap().verdict,
        "cancelled"
    );
    assert!(
        service.start_run(&id, version).is_err(),
        "fără aprobare nu se poate relua"
    );
}

#[test]
fn s20_cancel_stops_without_new_effects_and_pause_resumes() {
    let dir = tempfile::tempdir().unwrap();
    let service = open(dir.path(), slow());
    let (id, version) = approve_new_plan(&service);
    let run = service.start_run(&id, version).unwrap();
    assert!(
        matches!(service.start_run(&id, version), Err(AppError::Busy)),
        "un singur job per PC"
    );

    service.pause_run(&run.id).unwrap();
    wait_for_event(&service, &run.id, "paused");
    let written = count_events(&service, &run.id, "file_written");
    std::thread::sleep(Duration::from_millis(400));
    assert_eq!(
        count_events(&service, &run.id, "file_written"),
        written,
        "în pauză nu se scrie nimic"
    );
    assert_eq!(service.get_run(&run.id).unwrap().status, "paused");

    service.resume_run(&run.id).unwrap();
    wait_for_event(&service, &run.id, "file_written");
    service.cancel_run(&run.id).unwrap();
    assert!(service.wait_until_idle(WAIT));
    let stopped_at = count_events(&service, &run.id, "file_written");
    std::thread::sleep(Duration::from_millis(300));
    assert_eq!(count_events(&service, &run.id, "file_written"), stopped_at);
    assert_eq!(service.get_run(&run.id).unwrap().status, "cancelled");
}

/// Simulează un crash: rularea rămâne „running” în baza de date.
fn crashed_run(dir: &Path, write_files: bool) -> String {
    let service = open(dir, Pacing::INSTANT);
    let (id, version) = approve_new_plan(&service);
    let plan_view = service.current_plan().unwrap().unwrap();
    let root = service.project_root().to_path_buf();
    drop(service);

    let store = Store::open(&dir.join("state.sqlite")).unwrap();
    let stored = store.plan(&id, version).unwrap().unwrap();
    let approval = store.latest_approval(&id, version).unwrap().unwrap();
    let mut run = Run::new(&stored.plan);
    run.start_task("generate").unwrap();
    store
        .insert_run(&RunRow {
            id: "run-crash".into(),
            plan_id: id,
            plan_version: version,
            plan_sha256: plan_view.sha256,
            approval_id: approval.id,
            run,
            results_json: None,
            created_at: "2026-10-22T12:00:00Z".into(),
            updated_at: "2026-10-22T12:00:00Z".into(),
        })
        .unwrap();
    if write_files {
        for (path, content) in rab_app::demo::PROPOSAL {
            let target = root.join(path);
            std::fs::create_dir_all(target.parent().unwrap()).unwrap();
            std::fs::write(target, content).unwrap();
        }
    }
    "run-crash".into()
}

#[test]
fn s11_restart_requires_reconciliation_and_never_blindly_repeats() {
    // Efectul nu s-a produs: pasul se reia.
    let dir = tempfile::tempdir().unwrap();
    let run_id = crashed_run(dir.path(), false);
    let service = open(dir.path(), Pacing::INSTANT);
    assert_eq!(
        service.get_run(&run_id).unwrap().status,
        "recovery_required"
    );
    assert!(
        matches!(
            service.start_run("plan-obby-demo", 1),
            Err(AppError::InvalidState(_))
        ),
        "nu pornește o rulare nouă peste una nereconciliată"
    );
    service.resolve_recovery(&run_id).unwrap();
    assert!(service.wait_until_idle(WAIT));
    assert_eq!(service.get_run(&run_id).unwrap().status, "succeeded");
    assert_eq!(count_events(&service, &run_id, "file_written"), 3);

    // Efectul s-a produs înainte de crash: nu se rescrie nimic.
    let dir = tempfile::tempdir().unwrap();
    let run_id = crashed_run(dir.path(), true);
    let service = open(dir.path(), Pacing::INSTANT);
    service.resolve_recovery(&run_id).unwrap();
    assert!(service.wait_until_idle(WAIT));
    let events = service.events(&run_id, 0, 1000).unwrap();
    assert!(events.iter().any(|e| e.message.contains("nu se repetă")));
    let writes: Vec<_> = events.iter().filter(|e| e.kind == "file_written").collect();
    assert_eq!(
        writes.len(),
        1,
        "doar raportul se scrie; codul nu se repetă"
    );
    assert!(writes[0].message.contains(REPORT_PATH));
    assert_eq!(service.get_run(&run_id).unwrap().status, "succeeded");
}

#[test]
fn user_edits_are_never_overwritten() {
    let dir = tempfile::tempdir().unwrap();
    let service = open(dir.path(), Pacing::INSTANT);
    let edited = service.project_root().join("src/server/Course.server.luau");
    std::fs::create_dir_all(edited.parent().unwrap()).unwrap();
    std::fs::write(&edited, "-- editat de mine\n").unwrap();
    let (id, version) = approve_new_plan(&service);
    let run = service.start_run(&id, version).unwrap();
    assert!(service.wait_until_idle(WAIT));
    assert_eq!(
        std::fs::read_to_string(&edited).unwrap(),
        "-- editat de mine\n"
    );
    let run = service.get_run(&run.id).unwrap();
    assert_eq!(run.status, "failed", "trei încercări, apoi oprire");
    assert_eq!(count_events(&service, &run.id, "task_failed"), 3);
    assert_eq!(service.results(&run.id).unwrap().unwrap().verdict, "failed");
}

fn reply(
    service: &AppService,
    conversation: &str,
    prompt: &str,
    cancel_immediately: bool,
) -> Message {
    let (tx, rx) = mpsc::channel();
    let tx = Mutex::new(tx);
    let request = service
        .start_reply(
            conversation,
            prompt,
            &format!("req-{}", prompt.len() + usize::from(cancel_immediately)),
            Box::new(move |event| {
                if let ReplyEvent::Done { message } = event {
                    let _ = tx.lock().unwrap().send(*message);
                }
            }),
        )
        .unwrap();
    if cancel_immediately {
        assert!(service.cancel_reply(&request));
    }
    rx.recv_timeout(WAIT).unwrap()
}

#[test]
fn conversations_survive_restart_and_stop_keeps_history() {
    let dir = tempfile::tempdir().unwrap();
    let conversation;
    {
        let service = open(dir.path(), Pacing::INSTANT);
        let created = service.create_conversation("project").unwrap();
        assert_eq!(created.visibility, "private");
        service.set_visibility(&created.id, "shared").unwrap();
        let (prompt, summary) = service
            .send_message(&created.id, "Explică-mi checkpointul pe gheață")
            .unwrap();
        assert_eq!(summary.title, "Explică-mi checkpointul pe gheață");
        assert!(
            service.set_visibility(&created.id, "private").is_err(),
            "după primul mesaj nu se mai schimbă"
        );
        let answer = reply(&service, &created.id, prompt.id(), false);
        assert!(matches!(
            &answer,
            Message::Assistant {
                stopped: false,
                attempt: 1,
                ..
            }
        ));
        conversation = created.id;
    }
    let service = open(
        dir.path(),
        Pacing {
            thinking_ms: 2000,
            ..Pacing::INSTANT
        },
    );
    let messages = service.load_messages(&conversation).unwrap();
    assert_eq!(messages.len(), 2);
    let hits = service.search_conversations("GHEAȚĂ").unwrap();
    assert!(hits.contains(&conversation) && hits.contains(&"ideas".to_string()));
    assert!(!hits.contains(&"thumb".to_string()));
    let prompt_id = messages[0].id().to_string();
    let stopped = reply(&service, &conversation, &prompt_id, true);
    assert!(matches!(
        &stopped,
        Message::Assistant {
            stopped: true,
            attempt: 2,
            ..
        }
    ));
    assert_eq!(
        service.load_messages(&conversation).unwrap().len(),
        3,
        "reîncercarea nu șterge istoricul"
    );
}

#[test]
fn approval_records_the_identity_as_unauthenticated() {
    let dir = tempfile::tempdir().unwrap();
    let service = open(dir.path(), Pacing::INSTANT);
    approve_new_plan(&service);
    let approval = service.current_plan().unwrap().unwrap().approval.unwrap();
    assert_eq!(approval.approved_by, IDENTITY);
    assert_eq!(approval.identity_kind, "os_user_unauthenticated");
}
