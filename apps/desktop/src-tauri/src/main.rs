// Fără consolă suplimentară în build-ul release pe Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! Shellul desktop: comenzi tipizate subțiri peste `rab_app::AppService`.
//! Logica și testele sunt în `crates/app`; aici doar legăm IPC-ul Tauri.

use std::sync::Arc;

use rab_app::dto::{
    ConversationSummary, Message, PlanView, ReplyEvent, RunEventView, RunResults, RunView,
    WorkspaceInfo,
};
use rab_app::{AppError, AppService, EventSink, Pacing, os_identity};
use rab_core::examples::{ExampleVerdict, example_verdicts};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};

type Svc<'a> = State<'a, AppService>;
type Res<T> = Result<T, AppError>;

#[derive(Serialize)]
struct RuntimeInfo {
    app_version: &'static str,
    core_version: &'static str,
    /// Datele din panouri (agenți, echipă) sunt încă demonstrative.
    demo_mode: bool,
    example_configs: Vec<ExampleVerdict>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct RunEventNotice {
    run_id: String,
    seq: i64,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ReplyNotice {
    request_id: String,
    event: ReplyEvent,
}

/// Anunță UI-ul că o rulare are evenimente noi; UI cere apoi `run_events`.
struct TauriSink(AppHandle);

impl EventSink for TauriSink {
    fn run_event(&self, run_id: &str, seq: i64) {
        let _ = self.0.emit(
            "rab://run-event",
            RunEventNotice {
                run_id: run_id.into(),
                seq,
            },
        );
    }
}

#[tauri::command]
fn runtime_info() -> RuntimeInfo {
    RuntimeInfo {
        app_version: env!("CARGO_PKG_VERSION"),
        core_version: rab_core::VERSION,
        demo_mode: true,
        example_configs: example_verdicts(),
    }
}

#[tauri::command]
fn workspace_info(svc: Svc<'_>) -> WorkspaceInfo {
    svc.workspace_info()
}

#[tauri::command]
fn list_conversations(svc: Svc<'_>) -> Res<Vec<ConversationSummary>> {
    svc.list_conversations()
}

#[tauri::command]
fn load_messages(svc: Svc<'_>, conversation_id: String) -> Res<Vec<Message>> {
    svc.load_messages(&conversation_id)
}

#[tauri::command]
fn search_conversations(svc: Svc<'_>, query: String) -> Res<Vec<String>> {
    svc.search_conversations(&query)
}

#[tauri::command]
fn create_conversation(svc: Svc<'_>, scope: String) -> Res<ConversationSummary> {
    svc.create_conversation(&scope)
}

#[tauri::command]
fn rename_conversation(svc: Svc<'_>, id: String, title: String) -> Res<ConversationSummary> {
    svc.rename_conversation(&id, &title)
}

#[tauri::command]
fn set_visibility(svc: Svc<'_>, id: String, visibility: String) -> Res<ConversationSummary> {
    svc.set_visibility(&id, &visibility)
}

#[derive(Serialize)]
struct SentMessage {
    message: Message,
    conversation: ConversationSummary,
}

#[tauri::command]
fn send_message(svc: Svc<'_>, conversation_id: String, text: String) -> Res<SentMessage> {
    let (message, conversation) = svc.send_message(&conversation_id, &text)?;
    Ok(SentMessage {
        message,
        conversation,
    })
}

#[tauri::command]
fn start_reply(
    app: AppHandle,
    svc: Svc<'_>,
    conversation_id: String,
    prompt_message_id: String,
    request_id: String,
) -> Res<String> {
    let id = request_id.clone();
    svc.start_reply(
        &conversation_id,
        &prompt_message_id,
        &request_id,
        Box::new(move |event| {
            let _ = app.emit(
                "rab://reply",
                ReplyNotice {
                    request_id: id.clone(),
                    event,
                },
            );
        }),
    )
}

#[tauri::command]
fn cancel_reply(svc: Svc<'_>, request_id: String) -> bool {
    svc.cancel_reply(&request_id)
}

#[derive(Serialize)]
struct PreparedPlan {
    message: Message,
    plan: PlanView,
}

#[tauri::command]
fn prepare_plan(svc: Svc<'_>, conversation_id: String) -> Res<PreparedPlan> {
    let (message, plan) = svc.prepare_plan(&conversation_id)?;
    Ok(PreparedPlan { message, plan })
}

#[tauri::command]
fn current_plan(svc: Svc<'_>) -> Res<Option<PlanView>> {
    svc.current_plan()
}

#[tauri::command]
fn approve_plan(svc: Svc<'_>, plan_id: String, version: u32, sha256: String) -> Res<PlanView> {
    svc.approve_plan(&plan_id, version, &sha256)
}

#[tauri::command]
fn revoke_approval(svc: Svc<'_>, plan_id: String, version: u32) -> Res<PlanView> {
    svc.revoke_approval(&plan_id, version)
}

#[tauri::command]
fn start_run(svc: Svc<'_>, plan_id: String, version: u32) -> Res<RunView> {
    svc.start_run(&plan_id, version)
}

#[tauri::command]
fn pause_run(svc: Svc<'_>, run_id: String) -> Res<RunView> {
    svc.pause_run(&run_id)
}

#[tauri::command]
fn resume_run(svc: Svc<'_>, run_id: String) -> Res<RunView> {
    svc.resume_run(&run_id)
}

#[tauri::command]
fn cancel_run(svc: Svc<'_>, run_id: String) -> Res<RunView> {
    svc.cancel_run(&run_id)
}

#[tauri::command]
fn get_run(svc: Svc<'_>, run_id: String) -> Res<RunView> {
    svc.get_run(&run_id)
}

#[tauri::command]
fn run_events(svc: Svc<'_>, run_id: String, after_seq: i64) -> Res<Vec<RunEventView>> {
    svc.events(&run_id, after_seq, 500)
}

#[tauri::command]
fn run_results(svc: Svc<'_>, run_id: String) -> Res<Option<RunResults>> {
    svc.results(&run_id)
}

#[tauri::command]
fn resolve_recovery(svc: Svc<'_>, run_id: String) -> Res<RunView> {
    svc.resolve_recovery(&run_id)
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // %LOCALAPPDATA%/RobloxAIBuilder pe Windows (ARCHITECTURE §6).
            let data_dir = app.path().local_data_dir()?.join("RobloxAIBuilder");
            let sink = Arc::new(TauriSink(app.handle().clone()));
            let service = AppService::open(&data_dir, os_identity(), Pacing::INTERACTIVE, sink)?;
            app.manage(service);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            runtime_info,
            workspace_info,
            list_conversations,
            load_messages,
            search_conversations,
            create_conversation,
            rename_conversation,
            set_visibility,
            send_message,
            start_reply,
            cancel_reply,
            prepare_plan,
            current_plan,
            approve_plan,
            revoke_approval,
            start_run,
            pause_run,
            resume_run,
            cancel_run,
            get_run,
            run_events,
            run_results,
            resolve_recovery,
        ])
        .run(tauri::generate_context!())
        .expect("aplicația Tauri nu a putut porni");
}
