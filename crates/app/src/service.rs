//! Serviciul aplicației: comenzile apelate de shellul Tauri.
//!
//! În M1 rulează în procesul aplicației, pe fire separate; în M2 se mută în
//! worker, cu aceleași contracte. Toate efectele trec prin `rab_core` (policy
//! gate, executor) și sunt jurnalizate în SQLite.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use rab_core::approval::{Approval, ApprovalCheck};
use rab_core::config::AgentsConfig;
use rab_core::ids::{AgentRole, ProviderId};
use rab_core::plan::{Plan, PlanStatus, sha256_hex};
use rab_core::policy::{ProviderRequest, authorize_provider_call};
use rab_core::state::{Resolution, Run, RunStatus, TaskState};
use rab_storage::{ConversationRow, ProjectRow, RunRow, Store, StoredPlan, run_status_str};
use serde::Serialize;
use thiserror::Error;

use crate::chat::{DEMO_MODEL_LABEL, OWNER_NAME, demo_reply_for};
use crate::demo;
use crate::dto::*;
use crate::runner;

pub const IDENTITY_KIND: &str = "os_user_unauthenticated";
const SEED_JSON: &str = include_str!("../../../apps/desktop/src/mock/demo-seed.json");
const MAX_MESSAGE_CHARS: usize = 20_000;
const MAX_TITLE_CHARS: usize = 120;

#[derive(Debug, Error, Serialize)]
#[serde(tag = "code", content = "message", rename_all = "snake_case")]
pub enum AppError {
    #[error("stocare: {0}")]
    Storage(String),
    #[error("configurație: {0}")]
    Config(String),
    #[error("nu există: {0}")]
    NotFound(String),
    #[error("{0}")]
    InvalidState(String),
    #[error("refuzat: {0}")]
    Denied(String),
    #[error("o altă rulare este activă pe acest PC")]
    Busy,
    #[error("fișier: {0}")]
    Io(String),
}

impl From<rab_storage::StoreError> for AppError {
    fn from(e: rab_storage::StoreError) -> Self {
        AppError::Storage(e.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Io(e.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::Storage(e.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;

/// Notificare către UI că o rulare are evenimente noi (UI cere apoi `events`).
pub trait EventSink: Send + Sync {
    fn run_event(&self, run_id: &str, seq: i64);
}

pub struct NoopSink;
impl EventSink for NoopSink {
    fn run_event(&self, _run_id: &str, _seq: i64) {}
}

/// Ritmul demo-ului: pauze între pași ca progresul să fie vizibil. Nu inventează
/// progres: fiecare eveniment corespunde unui pas real.
#[derive(Debug, Clone, Copy)]
pub struct Pacing {
    pub step_ms: u64,
    pub thinking_ms: u64,
    pub tick_ms: u64,
    pub chars_per_tick: usize,
}

impl Pacing {
    pub const INTERACTIVE: Pacing = Pacing {
        step_ms: 700,
        thinking_ms: 900,
        tick_ms: 40,
        chars_per_tick: 5,
    };
    pub const INSTANT: Pacing = Pacing {
        step_ms: 0,
        thinking_ms: 0,
        tick_ms: 0,
        chars_per_tick: usize::MAX,
    };
}

#[derive(Default)]
pub(crate) struct RunControl {
    pub cancel: AtomicBool,
    pub paused: Mutex<bool>,
    pub cv: Condvar,
}

impl RunControl {
    /// Așteaptă `ms` milisecunde; întoarce `true` dacă s-a cerut oprirea.
    pub fn sleep(&self, ms: u64) -> bool {
        let mut left = ms;
        while left > 0 {
            if self.cancel.load(Ordering::SeqCst) {
                return true;
            }
            let step = left.min(20);
            std::thread::sleep(Duration::from_millis(step));
            left -= step;
        }
        self.cancel.load(Ordering::SeqCst)
    }
}

pub(crate) struct Inner {
    pub store: Store,
    pub data_dir: PathBuf,
    pub project_root: PathBuf,
    pub config: AgentsConfig,
    pub identity: String,
    pub pacing: Pacing,
    pub sink: Arc<dyn EventSink>,
    pub live: Mutex<HashMap<String, Arc<RunControl>>>,
    replies: Mutex<HashMap<String, Arc<AtomicBool>>>,
}

impl Inner {
    pub fn event(
        &self,
        run_id: &str,
        kind: &str,
        level: &str,
        message: &str,
        payload: serde_json::Value,
    ) {
        if let Ok(seq) = self.store.append_event(
            run_id,
            kind,
            level,
            message,
            &payload.to_string(),
            &now_iso(),
        ) {
            self.sink.run_event(run_id, seq);
        }
    }
}

#[derive(Clone)]
pub struct AppService {
    pub(crate) inner: Arc<Inner>,
}

pub fn now_iso() -> String {
    time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default()
}

fn iso_minus_hours(hours: i64) -> String {
    (time::OffsetDateTime::now_utc() - time::Duration::hours(hours))
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default()
}

/// Utilizatorul de sistem. Neautentificat în M1; legarea de SID vine în M2.
pub fn os_identity() -> String {
    let user = std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_else(|_| "necunoscut".into());
    let host = std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "PC".into());
    format!("{host}\\{user}")
}

fn new_id(prefix: &str) -> String {
    format!("{prefix}-{}", uuid::Uuid::new_v4())
}

fn agent_label(role: Option<AgentRole>) -> &'static str {
    match role {
        Some(AgentRole::Planner) => "Planificator",
        Some(AgentRole::Researcher) => "Cercetător",
        Some(AgentRole::Builder) => "Constructor",
        Some(AgentRole::Reviewer) => "Verificator",
        Some(AgentRole::MemoryCurator) => "Curator memorie",
        None => "Executor",
    }
}

pub(crate) fn plan_status_str(status: PlanStatus) -> &'static str {
    match status {
        PlanStatus::Draft => "draft",
        PlanStatus::AwaitingApproval => "awaiting_approval",
        PlanStatus::Approved => "approved",
        PlanStatus::Superseded => "superseded",
    }
}

pub(crate) fn task_state_str(state: &TaskState) -> (&'static str, u32) {
    match state {
        TaskState::Pending => ("pending", 0),
        TaskState::Running { attempt } => ("running", *attempt),
        TaskState::Succeeded { attempt, .. } => ("succeeded", *attempt),
        TaskState::Failed { attempts } => ("failed", *attempts),
        TaskState::RecoveryRequired { attempt } => ("recovery_required", *attempt),
    }
}

fn conversation_summary(row: &ConversationRow) -> ConversationSummary {
    ConversationSummary {
        id: row.id.clone(),
        title: row.title.clone(),
        visibility: row.visibility.clone(),
        scope: row.scope.clone(),
        last_activity: row.updated_at.clone(),
        is_new: row.is_new,
    }
}

impl AppService {
    /// Deschide baza de date, pregătește proiectul demo și reconciliază rulările întrerupte.
    pub fn open(
        data_dir: &Path,
        identity: String,
        pacing: Pacing,
        sink: Arc<dyn EventSink>,
    ) -> AppResult<Self> {
        std::fs::create_dir_all(data_dir)?;
        let store = Store::open(&data_dir.join("state.sqlite"))?;
        let project_root = data_dir.join("projects").join(demo::PROJECT_ID);
        demo::install_fixture(&project_root)?;
        let root_str = project_root.to_string_lossy().into_owned();
        let vault = data_dir.join("vault-demo").to_string_lossy().into_owned();
        let config =
            demo::demo_config(&root_str, &vault).map_err(|e| AppError::Config(e.to_string()))?;
        if store.project(demo::PROJECT_ID)?.is_none() {
            store.upsert_project(&ProjectRow {
                id: demo::PROJECT_ID.into(),
                name: demo::PROJECT_NAME.into(),
                root: root_str,
                created_at: now_iso(),
            })?;
        }
        let service = Self {
            inner: Arc::new(Inner {
                store,
                data_dir: data_dir.to_path_buf(),
                project_root,
                config,
                identity,
                pacing,
                sink,
                live: Mutex::new(HashMap::new()),
                replies: Mutex::new(HashMap::new()),
            }),
        };
        service.seed_conversations()?;
        service.recover_interrupted_runs()?;
        Ok(service)
    }

    fn seed_conversations(&self) -> AppResult<()> {
        let store = &self.inner.store;
        if store.conversation_count()? > 0 {
            return Ok(());
        }
        #[derive(serde::Deserialize)]
        struct Seed {
            conversations: Vec<ConversationSummary>,
            threads: HashMap<String, Vec<Message>>,
        }
        let seed: Seed = serde_json::from_str(SEED_JSON)?;
        let ages = [0, 24, 48, 96];
        for (i, c) in seed.conversations.iter().enumerate() {
            let at = iso_minus_hours(ages.get(i).copied().unwrap_or(120));
            store.append_conversation(&ConversationRow {
                id: c.id.clone(),
                project_id: (c.scope == "project").then(|| demo::PROJECT_ID.to_string()),
                title: c.title.clone(),
                visibility: c.visibility.clone(),
                scope: c.scope.clone(),
                is_new: false,
                created_at: at.clone(),
                updated_at: at.clone(),
            })?;
            for message in seed.threads.get(&c.id).into_iter().flatten() {
                store.append_message(
                    &c.id,
                    message.id(),
                    message.kind(),
                    &serde_json::to_string(message)?,
                    &message.search_text(),
                    &at,
                )?;
            }
        }
        Ok(())
    }

    /// S11: rulările rămase deschise la închiderea aplicației nu continuă orb.
    fn recover_interrupted_runs(&self) -> AppResult<()> {
        let inner = &self.inner;
        for row in inner.store.unfinished_runs()? {
            let mut run = row.run.clone();
            run.reconcile_after_restart();
            let message = match run.status {
                RunStatus::RecoveryRequired => {
                    "Aplicația s-a închis în timpul unui pas. Verificăm efectul înainte de a continua, fără să-l repetăm orbește."
                }
                RunStatus::Queued | RunStatus::Running => {
                    if run.status == RunStatus::Queued {
                        let _ = run.transition(RunStatus::Running);
                    }
                    let _ = run.transition(RunStatus::Paused);
                    "Aplicația s-a închis între pași. Rularea este în pauză; o poți relua."
                }
                _ => continue,
            };
            inner.store.save_run(&row.id, &run, &now_iso())?;
            inner.event(&row.id, "recovery", "warn", message, serde_json::json!({}));
        }
        Ok(())
    }

    pub fn workspace_info(&self) -> WorkspaceInfo {
        WorkspaceInfo {
            project_id: demo::PROJECT_ID.into(),
            project_name: demo::PROJECT_NAME.into(),
            project_root: self.inner.project_root.to_string_lossy().into_owned(),
            data_dir: self.inner.data_dir.to_string_lossy().into_owned(),
            identity: self.inner.identity.clone(),
        }
    }

    // ---- conversații ------------------------------------------------------

    pub fn list_conversations(&self) -> AppResult<Vec<ConversationSummary>> {
        Ok(self
            .inner
            .store
            .conversations()?
            .iter()
            .map(conversation_summary)
            .collect())
    }

    pub fn load_messages(&self, conversation_id: &str) -> AppResult<Vec<Message>> {
        self.require_conversation(conversation_id)?;
        self.inner
            .store
            .messages(conversation_id)?
            .iter()
            .map(|row| serde_json::from_str(&row.body_json).map_err(AppError::from))
            .collect()
    }

    pub fn search_conversations(&self, query: &str) -> AppResult<Vec<String>> {
        Ok(self.inner.store.search_conversations(query)?)
    }

    fn require_conversation(&self, id: &str) -> AppResult<ConversationRow> {
        self.inner
            .store
            .conversation(id)?
            .ok_or_else(|| AppError::NotFound(format!("conversația {id}")))
    }

    pub fn create_conversation(&self, scope: &str) -> AppResult<ConversationSummary> {
        if !matches!(scope, "project" | "general") {
            return Err(AppError::InvalidState(
                "tip de conversație necunoscut".into(),
            ));
        }
        let now = now_iso();
        let row = ConversationRow {
            id: new_id("conv"),
            project_id: (scope == "project").then(|| demo::PROJECT_ID.to_string()),
            title: "Conversație nouă".into(),
            visibility: "private".into(),
            scope: scope.into(),
            is_new: true,
            created_at: now.clone(),
            updated_at: now,
        };
        self.inner.store.insert_conversation(&row)?;
        Ok(conversation_summary(&row))
    }

    pub fn rename_conversation(&self, id: &str, title: &str) -> AppResult<ConversationSummary> {
        let mut row = self.require_conversation(id)?;
        let title: String = title.trim().chars().take(MAX_TITLE_CHARS).collect();
        if !title.is_empty() {
            row.title = title;
            row.updated_at = now_iso();
            self.inner.store.update_conversation(&row)?;
        }
        Ok(conversation_summary(&row))
    }

    /// Doar pentru o conversație nouă de proiect, fără mesaje: partajarea unui
    /// istoric existent cere selecția explicită a mesajelor (etapele C).
    pub fn set_visibility(&self, id: &str, visibility: &str) -> AppResult<ConversationSummary> {
        let mut row = self.require_conversation(id)?;
        if !matches!(visibility, "private" | "shared") {
            return Err(AppError::InvalidState("vizibilitate necunoscută".into()));
        }
        if !row.is_new || row.scope != "project" || self.inner.store.message_count(id)? > 0 {
            return Err(AppError::Denied(
                "vizibilitatea se alege doar pentru o conversație nouă de proiect".into(),
            ));
        }
        row.visibility = visibility.into();
        self.inner.store.update_conversation(&row)?;
        Ok(conversation_summary(&row))
    }

    fn append(&self, conversation_id: &str, message: &Message) -> AppResult<()> {
        self.inner.store.append_message(
            conversation_id,
            message.id(),
            message.kind(),
            &serde_json::to_string(message)?,
            &message.search_text(),
            &now_iso(),
        )?;
        Ok(())
    }

    pub fn send_message(
        &self,
        conversation_id: &str,
        text: &str,
    ) -> AppResult<(Message, ConversationSummary)> {
        let mut row = self.require_conversation(conversation_id)?;
        let body = text.trim();
        if body.is_empty() || body.chars().count() > MAX_MESSAGE_CHARS {
            return Err(AppError::InvalidState("mesaj gol sau prea lung".into()));
        }
        let message = Message::Person {
            id: new_id("msg"),
            author_id: "andrei".into(),
            time: now_iso(),
            text: body.into(),
            attachments: Vec::new(),
        };
        self.append(conversation_id, &message)?;
        if row.is_new && row.title == "Conversație nouă" {
            row.title = body.chars().take(40).collect();
        }
        row.is_new = false;
        row.updated_at = now_iso();
        self.inner.store.update_conversation(&row)?;
        Ok((message, conversation_summary(&row)))
    }

    /// Pornește un răspuns pe un fir separat; evenimentele ajung prin `sink`.
    /// `request_id` vine de la UI (ca evenimentele să nu poată sosi înaintea lui).
    pub fn start_reply(
        &self,
        conversation_id: &str,
        prompt_id: &str,
        request_id: &str,
        sink: Box<dyn Fn(ReplyEvent) + Send>,
    ) -> AppResult<String> {
        let valid_id = !request_id.is_empty()
            && request_id.len() <= 64
            && request_id
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-');
        if !valid_id || self.lock_replies().contains_key(request_id) {
            return Err(AppError::InvalidState(
                "identificator de cerere invalid".into(),
            ));
        }
        self.require_conversation(conversation_id)?;
        let messages = self.load_messages(conversation_id)?;
        let prompt = messages
            .iter()
            .find_map(|m| match m {
                Message::Person { id, text, .. } if id == prompt_id => Some(text.clone()),
                _ => None,
            })
            .ok_or_else(|| AppError::NotFound("mesajul la care se răspunde".into()))?;
        // Furnizorul local demonstrativ trece prin aceeași poartă ca un model real.
        authorize_provider_call(
            &self.inner.config,
            None,
            &ProviderRequest {
                provider: ProviderId::Ollama,
                model: demo::DEMO_MODEL,
                estimated_usd: Some(0.0),
            },
        )
        .map_err(|d| AppError::Denied(d.to_string()))?;
        let attempt = messages
            .iter()
            .filter(|m| matches!(m, Message::Assistant { reply_to: Some(r), .. } if r == prompt_id))
            .count() as u32
            + 1;

        let request_id = request_id.to_string();
        let cancel = Arc::new(AtomicBool::new(false));
        self.lock_replies()
            .insert(request_id.clone(), Arc::clone(&cancel));
        let service = self.clone();
        let conversation_id = conversation_id.to_string();
        let prompt_id = prompt_id.to_string();
        let request = request_id.clone();
        std::thread::spawn(move || {
            let result = service.run_reply(
                &conversation_id,
                &prompt_id,
                &prompt,
                attempt,
                &cancel,
                &sink,
            );
            match result {
                Ok(message) => sink(ReplyEvent::Done {
                    message: Box::new(message),
                }),
                Err(e) => sink(ReplyEvent::Error {
                    message: e.to_string(),
                }),
            }
            service.lock_replies().remove(&request);
        });
        Ok(request_id)
    }

    fn lock_replies(&self) -> std::sync::MutexGuard<'_, HashMap<String, Arc<AtomicBool>>> {
        self.inner
            .replies
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn run_reply(
        &self,
        conversation_id: &str,
        prompt_id: &str,
        prompt: &str,
        attempt: u32,
        cancel: &AtomicBool,
        sink: &dyn Fn(ReplyEvent),
    ) -> AppResult<Message> {
        let pacing = self.inner.pacing;
        let reply = demo_reply_for(prompt);
        let full: Vec<char> = reply.plain_text().chars().collect();
        let wait = |ms: u64| {
            let mut left = ms;
            while left > 0 && !cancel.load(Ordering::SeqCst) {
                let step = left.min(20);
                std::thread::sleep(Duration::from_millis(step));
                left -= step;
            }
            cancel.load(Ordering::SeqCst)
        };
        sink(ReplyEvent::Thinking);
        let mut shown = 0usize;
        let mut stopped = wait(pacing.thinking_ms);
        while !stopped && shown < full.len() {
            shown = shown
                .saturating_add(pacing.chars_per_tick.max(1))
                .min(full.len());
            sink(ReplyEvent::Streaming {
                text: full[..shown].iter().collect(),
            });
            stopped = wait(pacing.tick_ms);
        }
        let stopped = stopped && shown < full.len();
        let message = if stopped {
            let partial: String = full[..shown].iter().collect();
            Message::Assistant {
                id: new_id("msg"),
                time: now_iso(),
                model: DEMO_MODEL_LABEL.into(),
                tag: Some("Răspuns oprit".into()),
                paragraphs: vec![if partial.is_empty() {
                    "(oprit înainte de răspuns)".into()
                } else {
                    format!("{partial}…")
                }],
                list: None,
                code: None,
                extras: Vec::new(),
                reply_to: Some(prompt_id.into()),
                attempt,
                stopped: true,
            }
        } else {
            Message::Assistant {
                id: new_id("msg"),
                time: now_iso(),
                model: DEMO_MODEL_LABEL.into(),
                tag: reply.tag,
                paragraphs: reply.paragraphs,
                list: reply.list,
                code: None,
                extras: reply.extras,
                reply_to: Some(prompt_id.into()),
                attempt,
                stopped: false,
            }
        };
        self.append(conversation_id, &message)?;
        let mut row = self.require_conversation(conversation_id)?;
        row.updated_at = now_iso();
        self.inner.store.update_conversation(&row)?;
        Ok(message)
    }

    /// Oprirea cere anularea; costurile/efectele deja produse nu se anulează.
    pub fn cancel_reply(&self, request_id: &str) -> bool {
        match self.lock_replies().get(request_id) {
            Some(flag) => {
                flag.store(true, Ordering::SeqCst);
                true
            }
            None => false,
        }
    }

    // ---- planuri ----------------------------------------------------------

    /// Creează o versiune nouă a planului demonstrativ și o anunță în conversație.
    pub fn prepare_plan(&self, conversation_id: &str) -> AppResult<(Message, PlanView)> {
        self.require_conversation(conversation_id)?;
        let store = &self.inner.store;
        let previous = store.latest_plan(demo::PROJECT_ID)?;
        if let Some(prev) = &previous {
            if self.live_run_for_plan(&prev.plan).is_some() {
                return Err(AppError::Busy);
            }
            store.set_plan_status(&prev.plan.id, prev.plan.version, PlanStatus::Superseded)?;
        }
        let version = previous.map_or(1, |p| p.plan.version + 1);
        let plan = demo::demo_plan(&self.inner.project_root.to_string_lossy(), version);
        let issues = plan.validate(rab_core::config::Mode::Execution);
        if !issues.is_empty() {
            return Err(AppError::Config(format!(
                "{} probleme în plan",
                issues.len()
            )));
        }
        store.insert_plan(&plan, &now_iso())?;
        let view = self.plan_view(&StoredPlan {
            sha256: plan.sha256(),
            status: plan.status,
            created_at: now_iso(),
            plan: plan.clone(),
        })?;
        let message = Message::Assistant {
            id: new_id("msg"),
            time: now_iso(),
            model: DEMO_MODEL_LABEL.into(),
            tag: None,
            paragraphs: vec![
                "Am pregătit planul. Nimic nu se schimbă până nu îl aprobă proprietarul.".into(),
            ],
            list: None,
            code: None,
            extras: vec![AssistantExtra::PlanCard {
                title: format!(
                    "Plan v{} · {}",
                    plan.version,
                    plan.title.clone().unwrap_or_default()
                ),
                summary: format!(
                    "{} pași · {} teste · cost maxim 0,00 $ · date trimise extern: niciuna",
                    plan.tasks.len(),
                    view.tests.len()
                ),
                approver: format!("{OWNER_NAME} (proprietar)"),
                plan_id: Some(plan.id.clone()),
                plan_version: Some(plan.version),
            }],
            reply_to: None,
            attempt: 1,
            stopped: false,
        };
        self.append(conversation_id, &message)?;
        Ok((message, view))
    }

    pub fn current_plan(&self) -> AppResult<Option<PlanView>> {
        match self.inner.store.latest_plan(demo::PROJECT_ID)? {
            Some(stored) => Ok(Some(self.plan_view(&stored)?)),
            None => Ok(None),
        }
    }

    fn require_plan(&self, id: &str, version: u32) -> AppResult<StoredPlan> {
        self.inner
            .store
            .plan(id, version)?
            .ok_or_else(|| AppError::NotFound(format!("planul {id} v{version}")))
    }

    pub(crate) fn plan_view(&self, stored: &StoredPlan) -> AppResult<PlanView> {
        let plan = &stored.plan;
        let store = &self.inner.store;
        let number = |id: &str| {
            plan.tasks
                .iter()
                .position(|t| t.id == id)
                .map_or(0, |i| i + 1)
        };
        let tasks = plan
            .tasks
            .iter()
            .enumerate()
            .map(|(i, t)| PlanTaskView {
                id: t.id.clone(),
                number: i + 1,
                title: t.title.clone().unwrap_or_else(|| t.id.clone()),
                description: t.description.clone().unwrap_or_default(),
                agent: agent_label(t.agent).into(),
                location: "Fișiere locale".into(),
                zone: t.deliverables.join(", "),
                after: if t.depends_on.is_empty() {
                    "—".into()
                } else {
                    format!(
                        "după {}",
                        t.depends_on
                            .iter()
                            .map(|d| number(d).to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                },
            })
            .collect();
        let tests = plan
            .tasks
            .iter()
            .flat_map(|t| t.acceptance.iter())
            .enumerate()
            .map(|(i, title)| PlanTestView {
                id: format!("t{}", i + 1),
                title: title.clone(),
                kind: if demo::is_static_test(title) {
                    "static"
                } else {
                    "studio"
                }
                .into(),
            })
            .collect();
        let approval = store
            .latest_approval(&plan.id, plan.version)?
            .map(|a| ApprovalView {
                valid: a.approval.check(plan) == ApprovalCheck::Valid,
                approved_by: a.approval.approved_by,
                approved_at: a.approval.approved_at_utc,
                identity_kind: a.identity_kind,
                plan_sha256: a.approval.plan_sha256,
                revoked_at: a.approval.revoked_at_utc,
            });
        let latest_run_id = store
            .latest_run_for_plan(&plan.id, plan.version)?
            .map(|r| r.id);
        let write_paths = plan
            .scope
            .write_paths
            .iter()
            .map(|p| format!("{p}/"))
            .collect::<Vec<_>>()
            .join(", ");
        Ok(PlanView {
            id: plan.id.clone(),
            version: plan.version,
            status: plan_status_str(stored.status).into(),
            title: plan.title.clone().unwrap_or_else(|| plan.id.clone()),
            objective: plan.objective.clone(),
            sha256: stored.sha256.clone(),
            estimated_duration: "~1 min".into(),
            cost_estimate_usd: 0.0,
            cost_limit_usd: plan.budget.max_usd,
            max_retries: plan.budget.max_retries_per_task,
            provider_label: demo::PROVIDER_LABEL.into(),
            tasks,
            tests,
            permissions: vec![
                PermissionRow {
                    label: "Fișiere".into(),
                    value: format!("doar în {write_paths}"),
                },
                PermissionRow {
                    label: "Roblox Studio".into(),
                    value: "nu este folosit în acest plan".into(),
                },
                PermissionRow {
                    label: "Model AI".into(),
                    value: demo::PROVIDER_LABEL.into(),
                },
                PermissionRow {
                    label: "Date trimise pe internet".into(),
                    value: plan.scope.external_data.clone(),
                },
                PermissionRow {
                    label: "Git".into(),
                    value: "fără commit și fără push".into(),
                },
                PermissionRow {
                    label: "Publicare în Roblox".into(),
                    value: "nu e inclusă".into(),
                },
            ],
            approval,
            latest_run_id,
        })
    }

    /// Aprobarea este legată de hash-ul afișat; dacă planul salvat diferă, se refuză.
    pub fn approve_plan(&self, id: &str, version: u32, shown_sha256: &str) -> AppResult<PlanView> {
        let stored = self.require_plan(id, version)?;
        if stored.status != PlanStatus::AwaitingApproval {
            return Err(AppError::InvalidState("planul nu așteaptă aprobare".into()));
        }
        if stored.sha256 != shown_sha256 || stored.plan.sha256() != shown_sha256 {
            return Err(AppError::Denied(
                "planul afișat diferă de cel salvat".into(),
            ));
        }
        let approval = Approval::grant(&stored.plan, &self.inner.identity, &now_iso())
            .map_err(|e| AppError::Denied(e.to_string()))?;
        let store = &self.inner.store;
        store.insert_approval(&new_id("appr"), &approval, IDENTITY_KIND)?;
        store.set_plan_status(id, version, PlanStatus::Approved)?;
        store.append_audit(
            None,
            "plan_approved",
            &serde_json::json!({ "plan": id, "version": version, "sha256": shown_sha256 })
                .to_string(),
            &now_iso(),
        )?;
        self.plan_view(&self.require_plan(id, version)?)
    }

    /// Revocarea invalidează aprobarea; o rulare activă nu mai produce efecte noi.
    pub fn revoke_approval(&self, id: &str, version: u32) -> AppResult<PlanView> {
        let stored = self.require_plan(id, version)?;
        let store = &self.inner.store;
        let approval = store
            .latest_approval(id, version)?
            .filter(|a| a.approval.revoked_at_utc.is_none())
            .ok_or_else(|| AppError::InvalidState("nu există o aprobare activă".into()))?;
        store.revoke_approval(&approval.id, &now_iso())?;
        store.set_plan_status(id, version, PlanStatus::AwaitingApproval)?;
        store.append_audit(
            None,
            "approval_revoked",
            &serde_json::json!({ "plan": id, "version": version }).to_string(),
            &now_iso(),
        )?;
        if let Some(control) = self.live_run_for_plan(&stored.plan) {
            control.cancel.store(true, Ordering::SeqCst);
            control.cv.notify_all();
        }
        self.plan_view(&self.require_plan(id, version)?)
    }

    // ---- rulări -----------------------------------------------------------

    fn lock_live(&self) -> std::sync::MutexGuard<'_, HashMap<String, Arc<RunControl>>> {
        self.inner
            .live
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn live_run_for_plan(&self, plan: &Plan) -> Option<Arc<RunControl>> {
        let run = self
            .inner
            .store
            .latest_run_for_plan(&plan.id, plan.version)
            .ok()??;
        self.lock_live().get(&run.id).cloned()
    }

    fn spawn_runner(&self, run_id: &str) -> AppResult<()> {
        let control = {
            let mut live = self.lock_live();
            if !live.is_empty() {
                return Err(AppError::Busy);
            }
            let control = Arc::new(RunControl::default());
            live.insert(run_id.to_string(), Arc::clone(&control));
            control
        };
        let inner = Arc::clone(&self.inner);
        let run_id = run_id.to_string();
        std::thread::spawn(move || {
            runner::execute(&inner, &run_id, &control);
            inner
                .live
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .remove(&run_id);
            if let Ok(Some(last)) = inner.store.last_event(&run_id) {
                inner.sink.run_event(&run_id, last.seq);
            }
        });
        Ok(())
    }

    pub fn start_run(&self, plan_id: &str, version: u32) -> AppResult<RunView> {
        if !self.lock_live().is_empty() {
            return Err(AppError::Busy);
        }
        let stored = self.require_plan(plan_id, version)?;
        if stored.status != PlanStatus::Approved {
            return Err(AppError::Denied("planul nu este aprobat".into()));
        }
        if let Some(previous) = self.inner.store.latest_run_for_plan(plan_id, version)?
            && !previous.run.status.is_terminal()
        {
            return Err(AppError::InvalidState(
                "există o rulare nefinalizată pentru acest plan; reia-o sau oprește-o".into(),
            ));
        }
        let approval = self
            .inner
            .store
            .latest_approval(plan_id, version)?
            .ok_or_else(|| AppError::Denied("lipsește aprobarea".into()))?;
        if approval.approval.check(&stored.plan) != ApprovalCheck::Valid {
            return Err(AppError::Denied(
                "aprobarea nu mai corespunde planului".into(),
            ));
        }
        let now = now_iso();
        let run_id = new_id("run");
        self.inner.store.insert_run(&RunRow {
            id: run_id.clone(),
            plan_id: plan_id.into(),
            plan_version: version,
            plan_sha256: stored.sha256.clone(),
            approval_id: approval.id,
            run: Run::new(&stored.plan),
            results_json: None,
            created_at: now.clone(),
            updated_at: now,
        })?;
        self.inner.event(
            &run_id,
            "queued",
            "info",
            "Rulare creată după aprobarea planului",
            serde_json::json!({}),
        );
        self.spawn_runner(&run_id)?;
        self.get_run(&run_id)
    }

    fn require_run(&self, run_id: &str) -> AppResult<RunRow> {
        self.inner
            .store
            .run(run_id)?
            .ok_or_else(|| AppError::NotFound(format!("rularea {run_id}")))
    }

    pub fn get_run(&self, run_id: &str) -> AppResult<RunView> {
        let row = self.require_run(run_id)?;
        let stored = self.require_plan(&row.plan_id, row.plan_version)?;
        let title = |id: &str| {
            stored
                .plan
                .task(id)
                .and_then(|t| t.title.clone())
                .unwrap_or_else(|| id.to_string())
        };
        let tasks: Vec<RunTaskView> = row
            .run
            .tasks()
            .map(|(id, state)| {
                let (state, attempt) = task_state_str(state);
                RunTaskView {
                    id: id.into(),
                    title: title(id),
                    state: state.into(),
                    attempt,
                }
            })
            .collect();
        let last = self.inner.store.last_event(run_id)?;
        Ok(RunView {
            id: row.id.clone(),
            plan_id: row.plan_id.clone(),
            plan_version: row.plan_version,
            status: run_status_str(row.run.status),
            done: tasks.iter().filter(|t| t.state == "succeeded").count(),
            total: tasks.len(),
            tasks,
            activity: last.as_ref().map(|e| e.message.clone()).unwrap_or_default(),
            started_at: row.created_at.clone(),
            updated_at: row.updated_at.clone(),
            last_seq: last.map_or(0, |e| e.seq),
            live: self.lock_live().contains_key(run_id),
            has_results: row.results_json.is_some(),
        })
    }

    pub fn events(&self, run_id: &str, after_seq: i64, limit: u32) -> AppResult<Vec<RunEventView>> {
        self.require_run(run_id)?;
        Ok(self
            .inner
            .store
            .events_after(run_id, after_seq, limit.min(1000))?
            .into_iter()
            .map(|e| RunEventView {
                seq: e.seq,
                at: e.created_at,
                kind: e.kind,
                level: e.level,
                message: e.message,
            })
            .collect())
    }

    pub fn results(&self, run_id: &str) -> AppResult<Option<RunResults>> {
        let row = self.require_run(run_id)?;
        row.results_json
            .map(|json| serde_json::from_str(&json).map_err(AppError::from))
            .transpose()
    }

    pub fn pause_run(&self, run_id: &str) -> AppResult<RunView> {
        let control = self
            .lock_live()
            .get(run_id)
            .cloned()
            .ok_or_else(|| AppError::InvalidState("rularea nu este activă".into()))?;
        *control
            .paused
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = true;
        self.get_run(run_id)
    }

    pub fn resume_run(&self, run_id: &str) -> AppResult<RunView> {
        let live = self.lock_live().get(run_id).cloned();
        if let Some(control) = live {
            *control
                .paused
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = false;
            control.cv.notify_all();
            return self.get_run(run_id);
        }
        let row = self.require_run(run_id)?;
        if row.run.status != RunStatus::Paused {
            return Err(AppError::InvalidState("rularea nu este în pauză".into()));
        }
        self.spawn_runner(run_id)?;
        self.get_run(run_id)
    }

    pub fn cancel_run(&self, run_id: &str) -> AppResult<RunView> {
        let live = self.lock_live().get(run_id).cloned();
        if let Some(control) = live {
            control.cancel.store(true, Ordering::SeqCst);
            control.cv.notify_all();
            return self.get_run(run_id);
        }
        let row = self.require_run(run_id)?;
        if row.run.status.is_terminal() {
            return self.get_run(run_id);
        }
        let mut run = row.run.clone();
        run.transition(RunStatus::Cancelled)
            .map_err(|e| AppError::InvalidState(e.to_string()))?;
        self.inner.store.save_run(run_id, &run, &now_iso())?;
        self.inner.event(
            run_id,
            "cancelled",
            "warn",
            "Rulare oprită de utilizator",
            serde_json::json!({}),
        );
        runner::finalize(&self.inner, run_id, &run)?;
        self.get_run(run_id)
    }

    /// „Verifică și continuă”: inspectează efectele pașilor incerți, apoi reia.
    pub fn resolve_recovery(&self, run_id: &str) -> AppResult<RunView> {
        if self.lock_live().contains_key(run_id) {
            return Err(AppError::InvalidState("rularea este deja activă".into()));
        }
        let row = self.require_run(run_id)?;
        if row.run.status != RunStatus::RecoveryRequired {
            return Err(AppError::InvalidState("rularea nu cere recuperare".into()));
        }
        let mut run = row.run.clone();
        let uncertain: Vec<String> = run
            .tasks()
            .filter(|(_, s)| matches!(s, TaskState::RecoveryRequired { .. }))
            .map(|(id, _)| id.to_string())
            .collect();
        for task_id in uncertain {
            let resolution = runner::inspect_effect(&self.inner, &task_id);
            let message = match &resolution {
                Resolution::EffectConfirmed { .. } => format!(
                    "Pasul „{task_id}” a fost finalizat înainte de închidere; nu se repetă."
                ),
                Resolution::EffectAbsent => {
                    format!("Pasul „{task_id}” nu s-a terminat; se reia în siguranță.")
                }
            };
            run.resolve_recovery(&task_id, resolution)
                .map_err(|e| AppError::InvalidState(e.to_string()))?;
            self.inner.event(
                run_id,
                "recovery",
                "info",
                &message,
                serde_json::json!({ "task": task_id }),
            );
        }
        self.inner.store.save_run(run_id, &run, &now_iso())?;
        if run.status.is_terminal() {
            runner::finalize(&self.inner, run_id, &run)?;
        } else {
            self.spawn_runner(run_id)?;
        }
        self.get_run(run_id)
    }

    /// Ajutor pentru teste: așteaptă până nu mai există o rulare activă.
    pub fn wait_until_idle(&self, timeout: Duration) -> bool {
        let deadline = std::time::Instant::now() + timeout;
        while std::time::Instant::now() < deadline {
            if self.lock_live().is_empty() {
                return true;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        false
    }

    pub fn project_root(&self) -> &Path {
        &self.inner.project_root
    }

    /// Hash-ul conținutului unui fișier din proiect (pentru teste/recuperare).
    pub fn file_sha256(&self, rel: &str) -> Option<String> {
        std::fs::read(self.inner.project_root.join(rel))
            .ok()
            .map(|b| sha256_hex(&b))
    }
}
