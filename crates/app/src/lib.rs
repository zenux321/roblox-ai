//! Serviciul aplicației Roblox AI Builder (etapa M1): conversații persistente,
//! planuri, aprobări, rulări demonstrative pe un fixture local și rezultate.
//!
//! Testabil fără Tauri și fără Windows. Shellul Tauri doar expune comenzile.

pub mod chat;
pub mod checks;
pub mod demo;
pub mod diff;
pub mod dto;
mod runner;
pub mod service;

pub use runner::REPORT_PATH;
pub use service::{AppError, AppResult, AppService, EventSink, NoopSink, Pacing, os_identity};
