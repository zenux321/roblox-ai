//! Exemple JSON ale DTO-urilor, verificate și de testele TypeScript
//! (`apps/desktop/src/domain/contract.test.ts`). Regenerare:
//! `UPDATE_CONTRACT=1 cargo test -p rab-app --test contract`.
#![allow(clippy::unwrap_used)]

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use rab_app::dto::Message;
use rab_app::{AppService, NoopSink, Pacing};
use serde_json::{Value, json};

/// Înlocuiește valorile variabile (id-uri, timp, hash) ca fișierul să fie stabil.
fn normalize(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for (key, v) in map.iter_mut() {
                let volatile = matches!(
                    key.as_str(),
                    "id" | "time"
                        | "at"
                        | "startedAt"
                        | "updatedAt"
                        | "approvedAt"
                        | "sha256"
                        | "planSha256"
                        | "latestRunId"
                        | "lastActivity"
                        | "durationSeconds"
                        | "seq"
                        | "lastSeq"
                        | "approvedBy"
                ) && !v.is_object()
                    && !v.is_array();
                if volatile {
                    *v = match v {
                        Value::Number(_) => json!(0),
                        _ => json!("<variabil>"),
                    };
                } else {
                    normalize(v);
                }
            }
        }
        Value::Array(items) => items.iter_mut().for_each(normalize),
        _ => {}
    }
}

#[test]
fn dto_samples_match_committed_contract() {
    let dir = tempfile::tempdir().unwrap();
    let service = AppService::open(
        dir.path(),
        "PC\\test".into(),
        Pacing::INSTANT,
        Arc::new(NoopSink),
    )
    .unwrap();
    let messages: Vec<Message> = ["thumb", "checkpoint", "general"]
        .iter()
        .flat_map(|id| service.load_messages(id).unwrap())
        .collect();
    let (_, plan) = service.prepare_plan("checkpoint").unwrap();
    let approved = service
        .approve_plan(&plan.id, plan.version, &plan.sha256)
        .unwrap();
    let run = service.start_run(&plan.id, plan.version).unwrap();
    assert!(service.wait_until_idle(Duration::from_secs(10)));
    let run_view = service.get_run(&run.id).unwrap();
    let events: Vec<_> = service.events(&run.id, 0, 3).unwrap();
    let mut results = service.results(&run.id).unwrap().unwrap();
    for change in &mut results.changes {
        // Primele două rânduri: raportul conține apoi hash-ul planului (depinde de calea temporară).
        change.lines.truncate(2);
    }

    let mut sample = json!({
        "conversations": service.list_conversations().unwrap(),
        "messages": messages,
        "plan": approved,
        "run": run_view,
        "events": events,
        "results": results,
    });
    normalize(&mut sample);
    let generated = format!("{}\n", serde_json::to_string_pretty(&sample).unwrap());
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/contract/dto-samples.json");
    if std::env::var_os("UPDATE_CONTRACT").is_some() {
        std::fs::write(&path, &generated).unwrap();
        return;
    }
    let committed = std::fs::read_to_string(&path)
        .unwrap_or_default()
        .replace("\r\n", "\n");
    assert_eq!(
        committed, generated,
        "contractul DTO s-a schimbat; rulează cu UPDATE_CONTRACT=1 și verifică TS"
    );
}
