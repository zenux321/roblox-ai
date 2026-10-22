//! Teste de persistență pe fișiere reale (repornire = redeschiderea bazei).
#![allow(clippy::unwrap_used)]

use std::sync::Arc;

use rab_core::approval::Approval;
use rab_core::examples::PLAN_EXAMPLE_JSON;
use rab_core::plan::{Plan, PlanStatus};
use rab_core::state::{Run, RunStatus};
use rab_storage::{
    ConversationRow, MIGRATIONS, ProjectRow, RunRow, Store, StoreError, backup_path,
};

const NOW: &str = "2026-10-22T12:00:00Z";

fn project() -> ProjectRow {
    ProjectRow {
        id: "demo-obby".into(),
        name: "Obby demo".into(),
        root: "/tmp/obby".into(),
        created_at: NOW.into(),
    }
}

fn plan() -> Plan {
    let mut v: serde_json::Value = serde_json::from_str(PLAN_EXAMPLE_JSON).unwrap();
    v["example_only"] = serde_json::json!(false);
    v["project_root"] = serde_json::json!("/tmp/obby");
    v["model"] = serde_json::json!("demo-fake");
    v["status"] = serde_json::json!("awaiting_approval");
    serde_json::from_value(v).unwrap()
}

fn conversation(id: &str) -> ConversationRow {
    ConversationRow {
        id: id.into(),
        project_id: Some("demo-obby".into()),
        title: "Checkpoint".into(),
        visibility: "private".into(),
        scope: "project".into(),
        is_new: false,
        created_at: NOW.into(),
        updated_at: NOW.into(),
    }
}

#[test]
fn migrations_create_schema_and_refuse_newer_databases() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("Date aplicație ăîș").join("state.sqlite");
    let store = Store::open(&path).unwrap();
    assert_eq!(store.schema_version().unwrap() as usize, MIGRATIONS.len());
    drop(store);

    let conn = rusqlite_like_set_version(&path, 99);
    drop(conn);
    assert!(matches!(
        Store::open(&path),
        Err(StoreError::SchemaTooNew { found: 99, .. })
    ));
}

fn rusqlite_like_set_version(path: &std::path::Path, version: u32) -> Store {
    // Deschide cu o listă de migrații mai lungă pentru a ridica user_version.
    let extra: Vec<&str> =
        std::iter::repeat_n("SELECT 1;", version as usize - MIGRATIONS.len()).collect();
    let all: Vec<&str> = MIGRATIONS.iter().copied().chain(extra).collect();
    Store::open_with(path, &all).unwrap()
}

#[test]
fn migration_on_existing_database_makes_a_backup_and_keeps_data() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.sqlite");
    let store = Store::open(&path).unwrap();
    store.upsert_project(&project()).unwrap();
    drop(store);

    let next = [
        MIGRATIONS[0],
        "CREATE TABLE extra_test (id INTEGER PRIMARY KEY);",
    ];
    let upgraded = Store::open_with(&path, &next).unwrap();
    assert_eq!(upgraded.schema_version().unwrap(), 2);
    assert!(upgraded.project("demo-obby").unwrap().is_some());
    let backup = backup_path(&path, 1);
    assert!(backup.exists(), "copia de siguranță lipsește");
    let from_backup = Store::open(&backup).unwrap();
    assert!(from_backup.project("demo-obby").unwrap().is_some());
}

#[test]
fn conversations_and_messages_survive_restart() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.sqlite");
    {
        let store = Store::open(&path).unwrap();
        store.upsert_project(&project()).unwrap();
        store.append_conversation(&conversation("c1")).unwrap();
        store.insert_conversation(&conversation("c2")).unwrap();
        store
            .append_message(
                "c1",
                "m1",
                "person",
                r#"{"text":"Gheață"}"#,
                "Gheață pe platforme",
                NOW,
            )
            .unwrap();
        store
            .append_message("c1", "m2", "assistant", r#"{"p":1}"#, "răspuns", NOW)
            .unwrap();
    }
    let store = Store::open(&path).unwrap();
    let ids: Vec<String> = store
        .conversations()
        .unwrap()
        .into_iter()
        .map(|c| c.id)
        .collect();
    assert_eq!(ids, vec!["c2", "c1"], "conversația nouă apare prima");
    let messages = store.messages("c1").unwrap();
    assert_eq!(
        messages.iter().map(|m| m.seq).collect::<Vec<_>>(),
        vec![1, 2]
    );
    assert_eq!(store.search_conversations("GHEAȚĂ").unwrap(), vec!["c1"]);
}

#[test]
fn approval_stays_bound_to_hash_after_restart() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.sqlite");
    let plan = plan();
    {
        let store = Store::open(&path).unwrap();
        store.upsert_project(&project()).unwrap();
        store.insert_plan(&plan, NOW).unwrap();
        let approval = Approval::grant(&plan, "PC\\andrei", NOW).unwrap();
        store
            .insert_approval("a1", &approval, "os_user_unauthenticated")
            .unwrap();
        store
            .set_plan_status(&plan.id, plan.version, PlanStatus::Approved)
            .unwrap();
    }
    let store = Store::open(&path).unwrap();
    let stored = store.plan(&plan.id, plan.version).unwrap().unwrap();
    assert_eq!(stored.status, PlanStatus::Approved);
    assert_eq!(stored.sha256, plan.sha256());
    let approval = store
        .latest_approval(&plan.id, plan.version)
        .unwrap()
        .unwrap();
    assert_eq!(
        approval.approval.check(&stored.plan),
        rab_core::approval::ApprovalCheck::Valid
    );

    // Un plan modificat (de ex. editat în fișier) nu mai corespunde aprobării.
    let mut tampered = stored.plan.clone();
    tampered.objective.push_str(" și publică");
    assert_eq!(
        approval.approval.check(&tampered),
        rab_core::approval::ApprovalCheck::HashMismatch
    );

    store
        .revoke_approval(&approval.id, "2026-10-22T12:10:00Z")
        .unwrap();
    let revoked = store
        .latest_approval(&plan.id, plan.version)
        .unwrap()
        .unwrap();
    assert_eq!(
        revoked.approval.check(&stored.plan),
        rab_core::approval::ApprovalCheck::Revoked
    );
}

#[test]
fn run_state_and_events_survive_restart() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.sqlite");
    let plan = plan();
    {
        let store = Store::open(&path).unwrap();
        store.upsert_project(&project()).unwrap();
        store.insert_plan(&plan, NOW).unwrap();
        let approval = Approval::grant(&plan, "PC\\andrei", NOW).unwrap();
        store
            .insert_approval("a1", &approval, "os_user_unauthenticated")
            .unwrap();
        let mut run = Run::new(&plan);
        store
            .insert_run(&RunRow {
                id: "r1".into(),
                plan_id: plan.id.clone(),
                plan_version: plan.version,
                plan_sha256: plan.sha256(),
                approval_id: "a1".into(),
                run: run.clone(),
                results_json: None,
                created_at: NOW.into(),
                updated_at: NOW.into(),
            })
            .unwrap();
        run.start_task("generate").unwrap();
        store.save_run("r1", &run, NOW).unwrap();
        for i in 0..3 {
            store
                .append_event("r1", "log", "info", &format!("pas {i}"), "{}", NOW)
                .unwrap();
        }
    }
    let store = Store::open(&path).unwrap();
    let unfinished = store.unfinished_runs().unwrap();
    assert_eq!(unfinished.len(), 1);
    let mut run = unfinished[0].run.clone();
    assert_eq!(run.status, RunStatus::Running);
    run.reconcile_after_restart();
    assert_eq!(run.status, RunStatus::RecoveryRequired);

    let all = store.events_after("r1", 0, 100).unwrap();
    assert_eq!(all.len(), 3);
    assert!(all.windows(2).all(|w| w[1].seq > w[0].seq));
    let after_first = store.events_after("r1", all[0].seq, 100).unwrap();
    assert_eq!(
        after_first
            .iter()
            .map(|e| e.message.as_str())
            .collect::<Vec<_>>(),
        vec!["pas 1", "pas 2"]
    );
}

#[test]
fn s07_concurrent_reservations_across_connections_respect_limit() {
    let dir = tempfile::tempdir().unwrap();
    let path = Arc::new(dir.path().join("state.sqlite"));
    Store::open(&path).unwrap();
    let handles: Vec<_> = (0..12)
        .map(|_| {
            let path = Arc::clone(&path);
            std::thread::spawn(move || {
                let store = Store::open(&path).unwrap();
                store.reserve_budget(None, 0.3, 1.0, NOW).is_ok()
            })
        })
        .collect();
    let granted = handles
        .into_iter()
        .map(|h| h.join().unwrap())
        .filter(|ok| *ok)
        .count();
    assert_eq!(granted, 3);
}

#[test]
fn uncertain_budget_is_not_released_and_zero_budget_denies() {
    let store = Store::open_in_memory().unwrap();
    let id = store.reserve_budget(None, 0.8, 1.0, NOW).unwrap();
    store.mark_budget_uncertain(id).unwrap();
    assert!(!store.release_budget(id).unwrap());
    assert!(matches!(
        store.reserve_budget(None, 0.5, 1.0, NOW),
        Err(StoreError::BudgetExceeded)
    ));
    let empty = Store::open_in_memory().unwrap();
    assert!(matches!(
        empty.reserve_budget(None, 0.01, 0.0, NOW),
        Err(StoreError::BudgetExceeded)
    ));
    assert!(
        empty.reserve_budget(None, 0.0, 0.0, NOW).is_ok(),
        "apelurile gratuite nu consumă buget"
    );
}

#[test]
fn file_changes_keep_the_original_content() {
    let store = Store::open_in_memory().unwrap();
    store.upsert_project(&project()).unwrap();
    let plan = plan();
    store.insert_plan(&plan, NOW).unwrap();
    let approval = Approval::grant(&plan, "PC\\andrei", NOW).unwrap();
    store
        .insert_approval("a1", &approval, "os_user_unauthenticated")
        .unwrap();
    store
        .insert_run(&RunRow {
            id: "r1".into(),
            plan_id: plan.id.clone(),
            plan_version: plan.version,
            plan_sha256: plan.sha256(),
            approval_id: "a1".into(),
            run: Run::new(&plan),
            results_json: None,
            created_at: NOW.into(),
            updated_at: NOW.into(),
        })
        .unwrap();
    store
        .record_file_change("r1", "src/a.luau", Some("vechi"), "nou 1")
        .unwrap();
    store
        .record_file_change("r1", "src/a.luau", Some("nou 1"), "nou 2")
        .unwrap();
    let changes = store.file_changes("r1").unwrap();
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].before_text.as_deref(), Some("vechi"));
    assert_eq!(changes[0].after_text, "nou 2");
}
