# ADR 0002 — Serviciul local M1: SQLite, rulare în proces, evenimente Tauri

Stare: acceptat · 22 octombrie 2026 · etapa M1

## Context

ARCHITECTURE.md cere SQLite pentru stare, `SubscribeEvents(afterSequence)` cu secvențe persistente și un worker separat (M2). M1 trebuie să livreze un demo cap-coadă cu plan, aprobare, execuție și rezultate, fără rețea și fără chei.

## Decizie

- **`crates/storage`**: SQLite prin `rusqlite` 0.40.2 cu SQLite inclus în build (`bundled`), WAL, chei străine, migrații versionate în `PRAGMA user_version`. Înaintea unei migrații pe o bază existentă se face `VACUUM INTO` într-o copie `*.pre-migration-vN.bak`. O bază cu schemă mai nouă decât aplicația este refuzată (fără downgrade).
- **`crates/app`**: serviciul aplicației (comenzile `CreatePlan`/`prepare_plan`, `ApprovePlan(planHash)`, `RevokeApproval`, `StartRun`, `PauseRun`, `CancelRun`, `GetRun`, `SubscribeEvents`, `GetDiff`/rezultate, conversații). Testabil fără Tauri. În M1 rulează **în procesul aplicației**, pe fire separate. În M2 același serviciu se mută în `worker.exe`, cu aceleași DTO-uri.
- **Shellul Tauri** expune comenzi sincrone subțiri. Progresul ajunge în UI prin evenimentele `rab://run-event` (doar `runId` + `seq`); UI cere apoi `run_events(afterSeq)`. Răspunsurile din chat vin pe `rab://reply`, cu `requestId` generat de UI, ca evenimentele să nu poată sosi înaintea identificatorului. Capabilities: `core:event:allow-listen`/`allow-unlisten`, nimic altceva nou. Nu am folosit `ipc::Channel`, ca să nu adăugăm permisiuni neverificate.
- **Identitatea aprobării**: numele utilizatorului de sistem (`COMPUTERNAME\USERNAME`), marcat `os_user_unauthenticated`. Legarea de SID-ul Windows prin named pipe cu ACL vine în M2.
- **Furnizor**: furnizorul fals din `rab-core`, trecut prin `GuardedProvider`, policy gate și o rezervare de buget de 0 USD. Propunerea lui este un JSON `patch_proposal`. Executorul aplică patch-urile numai după policy gate, în copia proiectului demonstrativ din `%LOCALAPPDATA%/RobloxAIBuilder/projects/obby-vulcanic`.
- **Contract DTO**: `tests/fixtures/contract/dto-samples.json` este generat din Rust și verificat de un test TypeScript (chei exacte, camelCase).

## Versiuni noi

| Crate | Versiune | Motiv |
|---|---|---|
| `rusqlite` | 0.40.2 (`bundled`) | SQLite fără dependență de sistem; compilat cu MSVC pe Windows |
| `uuid` | 1.28.0 (`v4`) | Identificatori de entități |
| `time` | 0.3.55 | Timp UTC RFC 3339 |

Nicio dependență npm nouă: `@tauri-apps/api` include `listen` și `mockIPC`.

## Consecințe și limite

- Închiderea ferestrei oprește rularea. La pornire, rulările rămase deschise trec în `recovery_required` (pas întrerupt) sau `paused` (oprire între pași). Reluarea inspectează efectul pe disc și nu repetă orbește.
- O singură rulare activă pe PC. O rulare nouă nu pornește peste una nereconciliată a aceluiași plan.
- Ritmul demo-ului (`Pacing::INTERACTIVE`, ~0,7 s între pași) este ales pentru vizibilitate. Fiecare eveniment corespunde însă unui pas real, iar procentul vine din sarcinile terminate.
- Shellul Tauri nu a fost compilat în Cloud: utilizatorul a exclus pachetele Linux (webkit2gtk). API-urile Tauri folosite (`local_data_dir`, `Emitter::emit`, `Manager::manage`, `setup`) și identificatorii de permisiuni au fost verificați în sursele `tauri` 2.12.5.
