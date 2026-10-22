//! Stocarea locală a aplicației (SQLite, WAL, chei străine, migrații versionate).
//!
//! Baza de date stă în `%LOCALAPPDATA%/RobloxAIBuilder/state.sqlite`, nu în
//! vault. Înaintea unei migrații pe o bază existentă se face o copie de
//! siguranță; o bază cu schemă mai nouă decât aplicația este refuzată.

mod migrations;

use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

use rab_core::approval::Approval;
use rab_core::plan::{Plan, PlanStatus};
use rab_core::state::{Run, RunStatus};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use thiserror::Error;

pub use migrations::MIGRATIONS;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("SQLite: {0}")]
    Sql(#[from] rusqlite::Error),
    #[error("fișier: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("baza de date are schema v{found}, mai nouă decât aplicația (v{supported})")]
    SchemaTooNew { found: u32, supported: u32 },
    #[error("nu există: {0}")]
    NotFound(String),
    #[error("bugetul ar fi depășit")]
    BudgetExceeded,
}

pub type Result<T> = std::result::Result<T, StoreError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectRow {
    pub id: String,
    pub name: String,
    pub root: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConversationRow {
    pub id: String,
    pub project_id: Option<String>,
    pub title: String,
    pub visibility: String,
    pub scope: String,
    pub is_new: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageRow {
    pub id: String,
    pub conversation_id: String,
    pub seq: i64,
    pub kind: String,
    pub body_json: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StoredPlan {
    pub plan: Plan,
    pub status: PlanStatus,
    pub sha256: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredApproval {
    pub id: String,
    pub approval: Approval,
    pub identity_kind: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunRow {
    pub id: String,
    pub plan_id: String,
    pub plan_version: u32,
    pub plan_sha256: String,
    pub approval_id: String,
    pub run: Run,
    pub results_json: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventRow {
    pub seq: i64,
    pub run_id: String,
    pub kind: String,
    pub level: String,
    pub message: String,
    pub payload_json: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileChangeRow {
    pub path: String,
    pub before_text: Option<String>,
    pub after_text: String,
}

struct RawRun {
    id: String,
    plan_id: String,
    plan_version: u32,
    plan_sha256: String,
    approval_id: String,
    state_json: String,
    results_json: Option<String>,
    created_at: String,
    updated_at: String,
}

pub struct Store {
    conn: Mutex<Connection>,
    path: Option<PathBuf>,
}

fn status_str(status: PlanStatus) -> &'static str {
    match status {
        PlanStatus::Draft => "draft",
        PlanStatus::AwaitingApproval => "awaiting_approval",
        PlanStatus::Approved => "approved",
        PlanStatus::Superseded => "superseded",
    }
}

fn parse_status(value: &str) -> PlanStatus {
    match value {
        "awaiting_approval" => PlanStatus::AwaitingApproval,
        "approved" => PlanStatus::Approved,
        "superseded" => PlanStatus::Superseded,
        _ => PlanStatus::Draft,
    }
}

pub fn run_status_str(status: RunStatus) -> String {
    serde_json::to_value(status)
        .ok()
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_default()
}

impl Store {
    /// Deschide (sau creează) baza de date și aplică migrațiile.
    pub fn open(path: &Path) -> Result<Self> {
        Self::open_with(path, MIGRATIONS)
    }

    /// Variantă pentru teste: permite o listă de migrații dată.
    pub fn open_with(path: &Path, migrations: &[&str]) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let existed = path.exists();
        let conn = Connection::open(path)?;
        conn.busy_timeout(Duration::from_secs(5))?;
        conn.query_row("PRAGMA journal_mode = WAL", [], |_| Ok(()))?;
        conn.pragma_update(None, "foreign_keys", true)?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        let store = Self {
            conn: Mutex::new(conn),
            path: Some(path.to_path_buf()),
        };
        store.migrate(migrations, existed)?;
        Ok(store)
    }

    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.pragma_update(None, "foreign_keys", true)?;
        let store = Self {
            conn: Mutex::new(conn),
            path: None,
        };
        store.migrate(MIGRATIONS, false)?;
        Ok(store)
    }

    fn lock(&self) -> MutexGuard<'_, Connection> {
        self.conn
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    pub fn schema_version(&self) -> Result<u32> {
        Ok(self
            .lock()
            .query_row("PRAGMA user_version", [], |r| r.get(0))?)
    }

    fn migrate(&self, migrations: &[&str], existed: bool) -> Result<()> {
        let target = u32::try_from(migrations.len()).unwrap_or(u32::MAX);
        let current = self.schema_version()?;
        if current > target {
            return Err(StoreError::SchemaTooNew {
                found: current,
                supported: target,
            });
        }
        if current == target {
            return Ok(());
        }
        if existed && current > 0 {
            self.backup_before_migration(current)?;
        }
        let mut conn = self.lock();
        for (index, sql) in migrations.iter().enumerate().skip(current as usize) {
            let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
            tx.execute_batch(sql)?;
            tx.pragma_update(None, "user_version", index as u32 + 1)?;
            tx.commit()?;
        }
        Ok(())
    }

    fn backup_before_migration(&self, version: u32) -> Result<()> {
        let Some(path) = &self.path else {
            return Ok(());
        };
        let backup = backup_path(path, version);
        // Copie consecventă prin VACUUM INTO (include conținutul din WAL).
        let _ = std::fs::remove_file(&backup);
        self.lock().execute(
            "VACUUM INTO ?1",
            params![backup.to_string_lossy().into_owned()],
        )?;
        Ok(())
    }

    // ---- proiecte ---------------------------------------------------------

    pub fn upsert_project(&self, p: &ProjectRow) -> Result<()> {
        self.lock().execute(
            "INSERT INTO projects (id, name, root, created_at) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(id) DO UPDATE SET name = excluded.name, root = excluded.root",
            params![p.id, p.name, p.root, p.created_at],
        )?;
        Ok(())
    }

    pub fn project(&self, id: &str) -> Result<Option<ProjectRow>> {
        Ok(self
            .lock()
            .query_row(
                "SELECT id, name, root, created_at FROM projects WHERE id = ?1",
                params![id],
                |r| {
                    Ok(ProjectRow {
                        id: r.get(0)?,
                        name: r.get(1)?,
                        root: r.get(2)?,
                        created_at: r.get(3)?,
                    })
                },
            )
            .optional()?)
    }

    // ---- conversații ------------------------------------------------------

    pub fn conversation_count(&self) -> Result<i64> {
        Ok(self
            .lock()
            .query_row("SELECT COUNT(*) FROM conversations", [], |r| r.get(0))?)
    }

    /// Inserează o conversație nouă în capul listei.
    pub fn insert_conversation(&self, c: &ConversationRow) -> Result<()> {
        let conn = self.lock();
        let position: i64 = conn.query_row(
            "SELECT COALESCE(MIN(position), 0) - 1 FROM conversations",
            [],
            |r| r.get(0),
        )?;
        conn.execute(
            "INSERT INTO conversations (id, project_id, title, visibility, scope, is_new, created_at, updated_at, position)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![c.id, c.project_id, c.title, c.visibility, c.scope, c.is_new, c.created_at, c.updated_at, position],
        )?;
        Ok(())
    }

    /// Inserează la finalul listei (folosit pentru datele inițiale).
    pub fn append_conversation(&self, c: &ConversationRow) -> Result<()> {
        let conn = self.lock();
        let position: i64 = conn.query_row(
            "SELECT COALESCE(MAX(position), 0) + 1 FROM conversations",
            [],
            |r| r.get(0),
        )?;
        conn.execute(
            "INSERT INTO conversations (id, project_id, title, visibility, scope, is_new, created_at, updated_at, position)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![c.id, c.project_id, c.title, c.visibility, c.scope, c.is_new, c.created_at, c.updated_at, position],
        )?;
        Ok(())
    }

    pub fn update_conversation(&self, c: &ConversationRow) -> Result<()> {
        let changed = self.lock().execute(
            "UPDATE conversations SET title = ?2, visibility = ?3, is_new = ?4, updated_at = ?5 WHERE id = ?1",
            params![c.id, c.title, c.visibility, c.is_new, c.updated_at],
        )?;
        if changed == 0 {
            return Err(StoreError::NotFound(c.id.clone()));
        }
        Ok(())
    }

    fn map_conversation(r: &rusqlite::Row<'_>) -> rusqlite::Result<ConversationRow> {
        Ok(ConversationRow {
            id: r.get(0)?,
            project_id: r.get(1)?,
            title: r.get(2)?,
            visibility: r.get(3)?,
            scope: r.get(4)?,
            is_new: r.get(5)?,
            created_at: r.get(6)?,
            updated_at: r.get(7)?,
        })
    }

    pub fn conversations(&self) -> Result<Vec<ConversationRow>> {
        let conn = self.lock();
        let mut stmt = conn.prepare(
            "SELECT id, project_id, title, visibility, scope, is_new, created_at, updated_at
             FROM conversations ORDER BY position",
        )?;
        let rows = stmt.query_map([], Self::map_conversation)?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    pub fn conversation(&self, id: &str) -> Result<Option<ConversationRow>> {
        Ok(self
            .lock()
            .query_row(
                "SELECT id, project_id, title, visibility, scope, is_new, created_at, updated_at
                 FROM conversations WHERE id = ?1",
                params![id],
                Self::map_conversation,
            )
            .optional()?)
    }

    pub fn append_message(
        &self,
        conversation_id: &str,
        id: &str,
        kind: &str,
        body_json: &str,
        search_text: &str,
        created_at: &str,
    ) -> Result<i64> {
        let conn = self.lock();
        let seq: i64 = conn.query_row(
            "SELECT COALESCE(MAX(seq), 0) + 1 FROM messages WHERE conversation_id = ?1",
            params![conversation_id],
            |r| r.get(0),
        )?;
        conn.execute(
            "INSERT INTO messages (id, conversation_id, seq, kind, body_json, search_text, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![id, conversation_id, seq, kind, body_json, search_text, created_at],
        )?;
        Ok(seq)
    }

    pub fn messages(&self, conversation_id: &str) -> Result<Vec<MessageRow>> {
        let conn = self.lock();
        let mut stmt = conn.prepare(
            "SELECT id, conversation_id, seq, kind, body_json, created_at
             FROM messages WHERE conversation_id = ?1 ORDER BY seq",
        )?;
        let rows = stmt.query_map(params![conversation_id], |r| {
            Ok(MessageRow {
                id: r.get(0)?,
                conversation_id: r.get(1)?,
                seq: r.get(2)?,
                kind: r.get(3)?,
                body_json: r.get(4)?,
                created_at: r.get(5)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    pub fn message_count(&self, conversation_id: &str) -> Result<i64> {
        Ok(self.lock().query_row(
            "SELECT COUNT(*) FROM messages WHERE conversation_id = ?1",
            params![conversation_id],
            |r| r.get(0),
        )?)
    }

    /// Caută în titluri și în textul mesajelor. Compararea fără majuscule se
    /// face în Rust, ca să funcționeze și pentru diacritice (LIKE din SQLite
    /// ignoră majusculele doar pentru ASCII).
    pub fn search_conversations(&self, needle: &str) -> Result<Vec<String>> {
        let needle = needle.trim().to_lowercase();
        let conn = self.lock();
        let mut stmt = conn.prepare(
            "SELECT c.id, c.title, COALESCE(GROUP_CONCAT(m.search_text, ' '), '')
             FROM conversations c LEFT JOIN messages m ON m.conversation_id = c.id
             GROUP BY c.id ORDER BY c.position",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
            ))
        })?;
        let mut hits = Vec::new();
        for row in rows {
            let (id, title, text) = row?;
            if needle.is_empty()
                || title.to_lowercase().contains(&needle)
                || text.to_lowercase().contains(&needle)
            {
                hits.push(id);
            }
        }
        Ok(hits)
    }

    // ---- planuri și aprobări ---------------------------------------------

    pub fn insert_plan(&self, plan: &Plan, created_at: &str) -> Result<()> {
        let mut conn = self.lock();
        let tx = conn.transaction()?;
        tx.execute(
            "INSERT INTO plans (id, version, project_id, status, plan_json, plan_sha256, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                plan.id,
                plan.version,
                plan.project_id,
                status_str(plan.status),
                serde_json::to_string(plan)?,
                plan.sha256(),
                created_at
            ],
        )?;
        for (position, task) in plan.tasks.iter().enumerate() {
            tx.execute(
                "INSERT INTO plan_tasks (plan_id, plan_version, task_id, position) VALUES (?1, ?2, ?3, ?4)",
                params![plan.id, plan.version, task.id, position as i64],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    fn map_plan(r: &rusqlite::Row<'_>) -> rusqlite::Result<(String, String, String, String)> {
        Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
    }

    fn decode_plan(raw: (String, String, String, String)) -> Result<StoredPlan> {
        let (status, json, sha256, created_at) = raw;
        let mut plan: Plan = serde_json::from_str(&json)?;
        plan.status = parse_status(&status);
        Ok(StoredPlan {
            plan,
            status: parse_status(&status),
            sha256,
            created_at,
        })
    }

    pub fn plan(&self, id: &str, version: u32) -> Result<Option<StoredPlan>> {
        let raw = self
            .lock()
            .query_row(
                "SELECT status, plan_json, plan_sha256, created_at FROM plans WHERE id = ?1 AND version = ?2",
                params![id, version],
                Self::map_plan,
            )
            .optional()?;
        raw.map(Self::decode_plan).transpose()
    }

    pub fn latest_plan(&self, project_id: &str) -> Result<Option<StoredPlan>> {
        let raw = self
            .lock()
            .query_row(
                "SELECT status, plan_json, plan_sha256, created_at FROM plans
                 WHERE project_id = ?1 AND status != 'superseded'
                 ORDER BY created_at DESC, version DESC LIMIT 1",
                params![project_id],
                Self::map_plan,
            )
            .optional()?;
        raw.map(Self::decode_plan).transpose()
    }

    pub fn set_plan_status(&self, id: &str, version: u32, status: PlanStatus) -> Result<()> {
        let changed = self.lock().execute(
            "UPDATE plans SET status = ?3 WHERE id = ?1 AND version = ?2",
            params![id, version, status_str(status)],
        )?;
        if changed == 0 {
            return Err(StoreError::NotFound(format!("{id} v{version}")));
        }
        Ok(())
    }

    pub fn insert_approval(
        &self,
        id: &str,
        approval: &Approval,
        identity_kind: &str,
    ) -> Result<()> {
        self.lock().execute(
            "INSERT INTO approvals (id, plan_id, plan_version, plan_sha256, approved_by, identity_kind, approved_at, revoked_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                id,
                approval.plan_id,
                approval.plan_version,
                approval.plan_sha256,
                approval.approved_by,
                identity_kind,
                approval.approved_at_utc,
                approval.revoked_at_utc
            ],
        )?;
        Ok(())
    }

    /// Cea mai recentă aprobare (inclusiv revocată) pentru planul dat.
    pub fn latest_approval(&self, plan_id: &str, version: u32) -> Result<Option<StoredApproval>> {
        Ok(self
            .lock()
            .query_row(
                "SELECT id, plan_id, plan_version, plan_sha256, approved_by, approved_at, revoked_at, identity_kind
                 FROM approvals WHERE plan_id = ?1 AND plan_version = ?2
                 ORDER BY approved_at DESC, rowid DESC LIMIT 1",
                params![plan_id, version],
                |r| {
                    Ok(StoredApproval {
                        id: r.get(0)?,
                        approval: Approval {
                            plan_id: r.get(1)?,
                            plan_version: r.get(2)?,
                            plan_sha256: r.get(3)?,
                            approved_by: r.get(4)?,
                            approved_at_utc: r.get(5)?,
                            revoked_at_utc: r.get(6)?,
                        },
                        identity_kind: r.get(7)?,
                    })
                },
            )
            .optional()?)
    }

    pub fn approval_by_id(&self, id: &str) -> Result<Option<Approval>> {
        Ok(self
            .lock()
            .query_row(
                "SELECT plan_id, plan_version, plan_sha256, approved_by, approved_at, revoked_at
                 FROM approvals WHERE id = ?1",
                params![id],
                |r| {
                    Ok(Approval {
                        plan_id: r.get(0)?,
                        plan_version: r.get(1)?,
                        plan_sha256: r.get(2)?,
                        approved_by: r.get(3)?,
                        approved_at_utc: r.get(4)?,
                        revoked_at_utc: r.get(5)?,
                    })
                },
            )
            .optional()?)
    }

    pub fn revoke_approval(&self, id: &str, at: &str) -> Result<()> {
        self.lock().execute(
            "UPDATE approvals SET revoked_at = ?2 WHERE id = ?1 AND revoked_at IS NULL",
            params![id, at],
        )?;
        Ok(())
    }

    // ---- rulări -----------------------------------------------------------

    pub fn insert_run(&self, row: &RunRow) -> Result<()> {
        self.lock().execute(
            "INSERT INTO runs (id, plan_id, plan_version, plan_sha256, approval_id, status, state_json, results_json, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                row.id,
                row.plan_id,
                row.plan_version,
                row.plan_sha256,
                row.approval_id,
                run_status_str(row.run.status),
                serde_json::to_string(&row.run)?,
                row.results_json,
                row.created_at,
                row.updated_at
            ],
        )?;
        Ok(())
    }

    pub fn save_run(&self, run_id: &str, run: &Run, updated_at: &str) -> Result<()> {
        let changed = self.lock().execute(
            "UPDATE runs SET status = ?2, state_json = ?3, updated_at = ?4 WHERE id = ?1",
            params![
                run_id,
                run_status_str(run.status),
                serde_json::to_string(run)?,
                updated_at
            ],
        )?;
        if changed == 0 {
            return Err(StoreError::NotFound(run_id.into()));
        }
        Ok(())
    }

    pub fn save_results(&self, run_id: &str, results_json: &str) -> Result<()> {
        self.lock().execute(
            "UPDATE runs SET results_json = ?2 WHERE id = ?1",
            params![run_id, results_json],
        )?;
        Ok(())
    }

    fn map_run(r: &rusqlite::Row<'_>) -> rusqlite::Result<RawRun> {
        Ok(RawRun {
            id: r.get(0)?,
            plan_id: r.get(1)?,
            plan_version: r.get(2)?,
            plan_sha256: r.get(3)?,
            approval_id: r.get(4)?,
            state_json: r.get(5)?,
            results_json: r.get(6)?,
            created_at: r.get(7)?,
            updated_at: r.get(8)?,
        })
    }

    fn decode_run(raw: RawRun) -> Result<RunRow> {
        Ok(RunRow {
            run: serde_json::from_str(&raw.state_json)?,
            id: raw.id,
            plan_id: raw.plan_id,
            plan_version: raw.plan_version,
            plan_sha256: raw.plan_sha256,
            approval_id: raw.approval_id,
            results_json: raw.results_json,
            created_at: raw.created_at,
            updated_at: raw.updated_at,
        })
    }

    const RUN_COLUMNS: &'static str = "id, plan_id, plan_version, plan_sha256, approval_id, state_json, results_json, created_at, updated_at";

    pub fn run(&self, id: &str) -> Result<Option<RunRow>> {
        let raw = self
            .lock()
            .query_row(
                &format!("SELECT {} FROM runs WHERE id = ?1", Self::RUN_COLUMNS),
                params![id],
                Self::map_run,
            )
            .optional()?;
        raw.map(Self::decode_run).transpose()
    }

    pub fn latest_run_for_plan(&self, plan_id: &str, version: u32) -> Result<Option<RunRow>> {
        let raw = self
            .lock()
            .query_row(
                &format!(
                    "SELECT {} FROM runs WHERE plan_id = ?1 AND plan_version = ?2
                     ORDER BY created_at DESC, rowid DESC LIMIT 1",
                    Self::RUN_COLUMNS
                ),
                params![plan_id, version],
                Self::map_run,
            )
            .optional()?;
        raw.map(Self::decode_run).transpose()
    }

    /// Rulările care nu au ajuns într-o stare finală.
    pub fn unfinished_runs(&self) -> Result<Vec<RunRow>> {
        let conn = self.lock();
        let mut stmt = conn.prepare(&format!(
            "SELECT {} FROM runs WHERE status NOT IN ('succeeded', 'failed', 'cancelled') ORDER BY created_at",
            Self::RUN_COLUMNS
        ))?;
        let raw: Vec<RawRun> = stmt
            .query_map([], Self::map_run)?
            .collect::<rusqlite::Result<_>>()?;
        drop(stmt);
        drop(conn);
        raw.into_iter().map(Self::decode_run).collect()
    }

    pub fn start_attempt(&self, run_id: &str, task_id: &str, attempt: u32, at: &str) -> Result<()> {
        self.lock().execute(
            "INSERT OR REPLACE INTO task_attempts (run_id, task_id, attempt, status, started_at) VALUES (?1, ?2, ?3, 'running', ?4)",
            params![run_id, task_id, attempt, at],
        )?;
        Ok(())
    }

    pub fn finish_attempt(
        &self,
        run_id: &str,
        task_id: &str,
        attempt: u32,
        status: &str,
        receipt_sha256: Option<&str>,
        at: &str,
    ) -> Result<()> {
        self.lock().execute(
            "UPDATE task_attempts SET status = ?4, receipt_sha256 = ?5, finished_at = ?6
             WHERE run_id = ?1 AND task_id = ?2 AND attempt = ?3",
            params![run_id, task_id, attempt, status, receipt_sha256, at],
        )?;
        Ok(())
    }

    // ---- evenimente, fișiere, audit ---------------------------------------

    pub fn append_event(
        &self,
        run_id: &str,
        kind: &str,
        level: &str,
        message: &str,
        payload_json: &str,
        at: &str,
    ) -> Result<i64> {
        let conn = self.lock();
        conn.execute(
            "INSERT INTO run_events (run_id, kind, level, message, payload_json, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![run_id, kind, level, message, payload_json, at],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// Evenimentele de după `after_seq`, pentru reconectare (`SubscribeEvents`).
    pub fn events_after(&self, run_id: &str, after_seq: i64, limit: u32) -> Result<Vec<EventRow>> {
        let conn = self.lock();
        let mut stmt = conn.prepare(
            "SELECT seq, run_id, kind, level, message, payload_json, created_at FROM run_events
             WHERE run_id = ?1 AND seq > ?2 ORDER BY seq LIMIT ?3",
        )?;
        let rows = stmt.query_map(params![run_id, after_seq, limit], |r| {
            Ok(EventRow {
                seq: r.get(0)?,
                run_id: r.get(1)?,
                kind: r.get(2)?,
                level: r.get(3)?,
                message: r.get(4)?,
                payload_json: r.get(5)?,
                created_at: r.get(6)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    pub fn last_event(&self, run_id: &str) -> Result<Option<EventRow>> {
        Ok(self
            .lock()
            .query_row(
                "SELECT seq, run_id, kind, level, message, payload_json, created_at FROM run_events
                 WHERE run_id = ?1 ORDER BY seq DESC LIMIT 1",
                params![run_id],
                |r| {
                    Ok(EventRow {
                        seq: r.get(0)?,
                        run_id: r.get(1)?,
                        kind: r.get(2)?,
                        level: r.get(3)?,
                        message: r.get(4)?,
                        payload_json: r.get(5)?,
                        created_at: r.get(6)?,
                    })
                },
            )
            .optional()?)
    }

    pub fn record_file_change(
        &self,
        run_id: &str,
        path: &str,
        before: Option<&str>,
        after: &str,
    ) -> Result<()> {
        // Păstrează conținutul inițial dacă fișierul se scrie de mai multe ori în aceeași rulare.
        self.lock().execute(
            "INSERT INTO run_file_changes (run_id, path, before_text, after_text) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(run_id, path) DO UPDATE SET after_text = excluded.after_text",
            params![run_id, path, before, after],
        )?;
        Ok(())
    }

    pub fn file_changes(&self, run_id: &str) -> Result<Vec<FileChangeRow>> {
        let conn = self.lock();
        let mut stmt = conn.prepare(
            "SELECT path, before_text, after_text FROM run_file_changes WHERE run_id = ?1 ORDER BY path",
        )?;
        let rows = stmt.query_map(params![run_id], |r| {
            Ok(FileChangeRow {
                path: r.get(0)?,
                before_text: r.get(1)?,
                after_text: r.get(2)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    pub fn append_audit(
        &self,
        run_id: Option<&str>,
        kind: &str,
        detail_json: &str,
        at: &str,
    ) -> Result<()> {
        self.lock().execute(
            "INSERT INTO audit_events (run_id, kind, detail_json, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![run_id, kind, detail_json, at],
        )?;
        Ok(())
    }

    pub fn audit_kinds(&self, run_id: &str) -> Result<Vec<String>> {
        let conn = self.lock();
        let mut stmt =
            conn.prepare("SELECT kind FROM audit_events WHERE run_id = ?1 ORDER BY id")?;
        let rows = stmt.query_map(params![run_id], |r| r.get(0))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    // ---- buget ------------------------------------------------------------

    /// Rezervă suma într-o tranzacție IMMEDIATE: două procese nu pot depăși
    /// împreună limita. Rezervările incerte rămân numărate.
    pub fn reserve_budget(
        &self,
        run_id: Option<&str>,
        amount_usd: f64,
        limit_usd: f64,
        at: &str,
    ) -> Result<i64> {
        let mut conn = self.lock();
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let used: f64 = tx.query_row(
            "SELECT COALESCE(SUM(CASE state WHEN 'committed' THEN COALESCE(actual_usd, amount_usd)
                                           WHEN 'released' THEN 0 ELSE amount_usd END), 0)
             FROM budget_reservations",
            [],
            |r| r.get(0),
        )?;
        if !amount_usd.is_finite() || amount_usd < 0.0 || used + amount_usd > limit_usd + 1e-9 {
            return Err(StoreError::BudgetExceeded);
        }
        tx.execute(
            "INSERT INTO budget_reservations (run_id, amount_usd, state, created_at) VALUES (?1, ?2, 'open', ?3)",
            params![run_id, amount_usd, at],
        )?;
        let id = tx.last_insert_rowid();
        tx.commit()?;
        Ok(id)
    }

    pub fn commit_budget(&self, id: i64, actual_usd: f64) -> Result<()> {
        self.lock().execute(
            "UPDATE budget_reservations SET state = 'committed', actual_usd = ?2 WHERE id = ?1",
            params![id, actual_usd],
        )?;
        Ok(())
    }

    /// Eliberează doar rezervările sigure; cele incerte rămân.
    pub fn release_budget(&self, id: i64) -> Result<bool> {
        let changed = self.lock().execute(
            "UPDATE budget_reservations SET state = 'released' WHERE id = ?1 AND state = 'open'",
            params![id],
        )?;
        Ok(changed == 1)
    }

    pub fn mark_budget_uncertain(&self, id: i64) -> Result<()> {
        self.lock().execute(
            "UPDATE budget_reservations SET state = 'uncertain' WHERE id = ?1 AND state = 'open'",
            params![id],
        )?;
        Ok(())
    }
}

pub fn backup_path(db: &Path, version: u32) -> PathBuf {
    let mut name = db.file_name().map(|n| n.to_os_string()).unwrap_or_default();
    name.push(format!(".pre-migration-v{version}.bak"));
    db.with_file_name(name)
}
