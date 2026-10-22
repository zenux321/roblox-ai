# ADR 0001 — Stack și versiuni fixate pentru etapa 1

Stare: acceptat · 22 octombrie 2026 · etapa 1 (M0 + primul ecran C0)

## Context

ARCHITECTURE.md fixează Tauri 2 + React/TypeScript, Rust pentru nucleu și SQLite. CLAUDE.md cere versiuni verificate, fixate, cu lockfile, și interzice reutilizarea runtime-ului prototipului. Versiunile de mai jos au fost citite din registry-urile oficiale (crates.io, npm) în ziua implementării.

## Decizie

| Componentă | Versiune | Motiv |
|---|---|---|
| Rust toolchain | 1.97.0 (`rust-toolchain.toml`), MSRV 1.90 | Stabil curent; Tauri 2.12 cere ≥ 1.90 |
| `tauri` / `tauri-build` | =2.12.5 / =2.7.4 | Ultimele 2.x; 3.0 este încă alpha |
| `@tauri-apps/api` / `@tauri-apps/cli` | 2.12.4 | Aceeași linie 2.12 ca crate-ul |
| React / React DOM | 19.3.1 | Ultima stabilă |
| Vite / `@vitejs/plugin-react` | 8.3.5 / 6.1.2 | Plugin-ul 6 cere Vite 8 |
| TypeScript | **6.0.3** | 7.0.2 există, dar `typescript-eslint` 8.72 acceptă `<6.1` |
| Vitest / jsdom | 5.0.5 / **29.1.1** | jsdom 30 cere Node ≥ 22.22.2; mediul are 22.22.0 |
| ESLint / typescript-eslint / react-hooks | 10.13.0 / 8.72.0 / 7.1.1 | Config flat, reguli type-checked |
| Playwright | **1.56.1** | Corespunde Chromium-ului preinstalat (build 1194); fără descărcare de browser |
| Fonturi | `@fontsource/onest` 5.3.1, `@fontsource/jetbrains-mono` 5.3.0 | OFL-1.1, incluse local (subseturi latin + latin-ext pentru diacritice) |
| Iconuri | `lucide-react` 1.61.0 | ISC; recomandat de handoff |
| Rust core | serde 1.0.229, serde_json 1.0.154, sha2 0.11.1, thiserror 2.0.21, schemars 1.2.3 | Minim necesar; `tempfile` doar pentru teste |

npm folosește `save-exact`; Cargo folosește `Cargo.lock` comis. Workspace-ul Cargo are `default-members = ["crates/core"]`: shellul Tauri cere WebView2 (Windows) sau webkit2gtk (Linux) și se compilează explicit cu `cargo build --workspace` pe Windows.

## Securitate

- CSP strictă în `tauri.conf.json` și injectată în build-ul web: `script-src 'self'`, `style-src 'self'`, fără CDN, fără `eval`, fără conexiuni externe. `freezePrototype: true`.
- Capabilities Tauri minime: doar minimize / toggle-maximize / close / start-dragging pentru bara de titlu proprie. Fără plugin-uri fs/shell/http.
- O singură comandă IPC, `runtime_info`, fără efecte și fără secrete.
- ESLint interzice `dangerouslySetInnerHTML`, `innerHTML`, `eval`, `new Function`.
- Prototipul HTML din `design/claude-export` nu este încărcat de aplicație; este randat doar în testul Playwright de comparație, într-un browser izolat.

## Consecințe

- Pe Windows: Node ≥ 22, Rust 1.97 (rustup îl instalează din `rust-toolchain.toml`), WebView2 și Build Tools MSVC, conform cerințelor Tauri.
- Revizuim TypeScript 7 când typescript-eslint îl suportă.
- Instalarea pachetelor Linux (webkit2gtk) a fost exclusă la cererea utilizatorului; compilarea shellului este verificată numai pe Windows.
