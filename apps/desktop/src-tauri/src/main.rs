// Fără consolă suplimentară în build-ul release pe Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use rab_core::examples::{ExampleVerdict, example_verdicts};
use serde::Serialize;

#[derive(Serialize)]
struct RuntimeInfo {
    app_version: &'static str,
    core_version: &'static str,
    /// În această etapă UI-ul folosește numai date demonstrative.
    demo_mode: bool,
    example_configs: Vec<ExampleVerdict>,
}

/// Singura comandă expusă UI-ului: informații fără secrete și fără efecte.
#[tauri::command]
fn runtime_info() -> RuntimeInfo {
    RuntimeInfo {
        app_version: env!("CARGO_PKG_VERSION"),
        core_version: rab_core::VERSION,
        demo_mode: true,
        example_configs: example_verdicts(),
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![runtime_info])
        .run(tauri::generate_context!())
        .expect("aplicația Tauri nu a putut porni");
}
