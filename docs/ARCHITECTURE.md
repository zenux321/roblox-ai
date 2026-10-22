# Arhitectură tehnică

## 1. Decizii de bază

Tauri 2 + React/TypeScript pentru interfață, Rust pentru motor și worker, SQLite pentru stare tranzacțională. Această alegere evită un backend găzduit și un runtime Python/Node separat pe calculatorul prietenilor. Rust adaugă complexitate de dezvoltare, acceptată pentru integrarea Windows și livrarea unor executabile. Node este necesar la build pentru frontend, nu pentru utilizatorul final. Verificarea fezabilității are loc în M0; nu se schimbă stackul fără un ADR motivat.

```text
UI Tauri/React ── comenzi tipizate ── bridge Rust
                                      │ IPC local autentificat
                               worker.exe (per-user)
                                      │
               plan → policy gate → scheduler → tool executor
                         │               │           │
                      SQLite          providers    adaptoare
                         │            Ollama       filesystem/Git
                    audit/outbox      API opt-in   Rojo/Studio MCP
                         │                         cercetare/imagini
                    vault Markdown + artifacts locale
```

UI nu poate apela direct furnizorii, shellul sau Studio. Agenții produc propuneri JSON. Numai executorul validează și aplică efecte. Rolurile logice nu presupun procese sau modele simultane.

## 2. Module și contracte

| Modul | Responsabilitate și limită |
|---|---|
| desktop | onboarding, plan/diff, progres, setări; fără secrete în React |
| core | tipuri, validare config, policy engine, mașină de stări; testabil fără Windows |
| worker | scheduler, leases, cancellation, checkpoints, IPC, single-instance |
| storage | SQLite migrations, audit, outbox, backup și restaurare |
| memory | Markdown, frontmatter, SQLite FTS local, conflicte de editare |
| providers | `health`, `capabilities`, `generate`, `cancel`, `usage`; Ollama și fake, apoi API |
| tools | acțiuni tipizate, allowlist, validare căi/destinații și rezultate |
| research | fetch limitat, proveniență, cache, extragere; nu execută conținut |
| roblox | discovery MCP, țintă explicită, Rojo, testare și snapshots |
| assets | imagini, hash, dimensiuni, origine/licență, upload separat |
| distribution | installer, diagnostic redacted, update manual inițial |

Structură țintă, de creat în M0–M1:

```text
apps/desktop/                  React + Tauri shell
crates/core/                   policy, config, plans, state machine
crates/worker/                 exe independent de UI
crates/storage/                SQLite, audit, migrations
crates/adapters/                providers, Git, research, Rojo, Studio
schemas/                       JSON schemas versionate
tests/fixtures/                intrări malițioase și provider fake
tests/integration/             recovery, IPC, filesystem, budgets
tests/windows/                 lifecycle și installer
config/ docs/ prompts/         contracte și documentație
vault-template/                memorie demonstrativă fără date personale
```

## 3. Date și stări

Tabele propuse: `projects`, `plans`, `plan_tasks`, `approvals`, `runs`, `task_attempts`, `leases`, `tool_calls`, `artifacts`, `sources`, `budget_reservations`, `usage_events`, `audit_events`, `outbox`. UUID pentru entități, UTC pentru timp, hash SHA-256 pentru conținut. UI convertește în fusul local. Foreign keys și migrații versionate; backup înaintea migrațiilor, tranzacții pentru claims și buget.

Plan: `draft → awaiting_approval → approved`; revocarea sau modificarea conținutului invalidează aprobarea. Run: `queued → running → succeeded | failed | cancelled`, cu `paused`, `waiting_dependency`, `waiting_approval`, `recovery_required` ca stări reluabile. Un task reușit nu este executat din nou doar pentru că UI s-a reconectat.

`Plan` conține project ID/root, versiune, snapshot de bază, DAG de sarcini, resurse, instrumente, destinații, provider/model, limite de cost și încercări, teste de acceptanță și acțiuni externe. `Approval` este legată de hash-ul JSON canonicalizat al planului și de identitatea Windows a utilizatorului, data acordului și starea revocării. Aprobarea este scrisă de worker numai prin acțiune UI autentificată; textul „approved” dintr-o notă nu este suficient.

API intern propus, nu endpointuri publice: `CreatePlan`, `ValidatePlan`, `ApprovePlan(planHash)`, `StartRun`, `PauseRun`, `CancelRun`, `GetRun`, `SubscribeEvents(afterSequence)`, `GetDiff`. Fiecare mesaj are versiune, request ID, limită de dimensiune și schemă strictă. Evenimentele au secvențe persistente pentru reconectare.

## 4. Executor și aprobarea planului

Pentru fiecare tool call: verifică approval activ și hash, dependențe, țintă, versiunea politicii, cale finală, instrument, argumente, egress, buget și starea anulării. Rezervă resursele, jurnalizează intenția, execută, verifică rezultatul și înregistrează efectul. Nu execută direct textul unui LLM.

Planul poate include push/PR/upload/publicare cu repo, branch, place/universe, asset, cantitate și limite exacte. În lipsa acestor câmpuri efectul este interzis. Modificarea scopului, a destinației sau creșterea bugetului produce o nouă versiune pentru aprobare. Reparațiile în domeniul deja aprobat nu cer aprobări repetitive. Dacă UI e închisă, noua aprobare rămâne în așteptare, fără dialog ascuns sau aprobare dedusă din timp.

Planul nu poate acorda capabilități interzise de politica globală. Registry-ul toolurilor este static și versionat. Adăugarea unui server MCP sau modificarea schema/tool fingerprint invalidează capabilitățile aprobate pentru server până la revalidare.

## 5. Worker Windows și recuperare

- `worker.exe` rulează ca utilizator standard, independent de procesul UI, nu Windows Service/LocalSystem. Startup per-user prin Task Scheduler la logon, numai după alegerea opțiunii background în setup.
- Un mutex per SID împiedică dublarea workerului. IPC prin named pipe cu ACL pentru SID-ul utilizatorului, verificarea clientului și handshake per sesiune. Niciun listener LAN. ACL-ul nu protejează împotriva malware-ului cu același cont: această limită este explicită.
- Tray oferă stare, Redeschide, Pauză și Oprește tot. Închiderea UI lasă workerul; comanda de oprire anulează tokenurile și închide procesele copil controlate.
- Un Job Object limitează procesele copil și facilitează terminarea lor; nu este tratat drept sandbox de securitate.
- Heartbeat/lease, checkpoint la tranziții și SQLite WAL. La restart, taskurile fără lease valid trec prin reconciliere. Fișierele folosesc scriere temporară și rename; SQLite și filesystem folosesc outbox plus reconciliere, nu presupun tranzacție comună.
- Efectele externe sunt cel puțin o dată ca încercare, nu „exactly once” garantat. ToolCall păstrează idempotency key și receipt. Dacă uploadul/pushul a reușit dar receiptul lipsește, caută efectul după identificatori; dacă nu se poate decide, `recovery_required`, fără repetare oarbă.
- Retry implicit maximum 2 pentru operații sigure/idempotente, backoff și jitter. Autentificarea invalidă, quota, moderarea sau efectul incert nu sunt retry automat.
- PC în sleep/hibernare/oprit: nu se execută. La resume se revalidează lease, Studio și buget. Logout oprește sesiunea per-user; la logon se reia recuperarea. Fără modificarea automată a politicii de sleep.

## 6. Obsidian și retenție

Vaultul este un folder Markdown; nu este necesar API Obsidian sau plugin. Aplicația scrie doar în subfolderul dedicat `RobloxAIBuilder/` ales de utilizator: Inbox, Projects, Sources, Decisions, Plans, Runs, Assets. Șablonul livrat poate fi deschis separat ca vault. [Format local Obsidian](https://obsidian.md/help/data-storage).

State DB, logurile brute și cheile stau în `%LOCALAPPDATA%/RobloxAIBuilder/`, nu în vault. Căile proiectelor sunt selectate de utilizator. Indexul FTS este regenerabil, fără embeddings cloud; embeddings locale sunt extensie după măsurarea necesității. Notele au `id`, `kind`, `project_id`, `source_url`, `accessed_at`, `trust` și linkuri către artefacte.

Watcher cu debounce, verificare hash înainte de scriere și fișier de conflict când utilizatorul a editat între timp. Nu suprascrie automat note personale. Nu trimite întregul vault unui provider; doar fragmente selectate, afișate în politica de date a planului. Loguri locale rotite, retenție propusă 30 zile, rapoarte păstrate până la ștergerea proiectului. Exportul exclude secretele; ștergerea datelor distinge vaultul de index/cache și backupuri.

## 7. Modele și cercetare

Ollama folosește endpoint local fixat și modele locale instalate, selectate după benchmark și licență. Endpointul local poate rula și modele cloud; verifică modelul și blochează variantele remote în modul local-only, nu doar hostname-ul. API-ul local nu are autentificare implicită, deci nu îl expune în LAN. [Ollama](https://docs.ollama.com/api/authentication).

Roluri: planner, researcher, builder, reviewer, memory curator. Toate pot folosi același model, serial. Reviewerul LLM nu înlocuiește teste deterministe. Context maxim, task timeout, plafon de output și număr finit de cicluri sunt obligatorii. Un răspuns invalid se validează/repară limitat, apoi sarcina e oprită.

Research folosește fetch read-only în proces separat, cu timeout, limită de bytes, redirecturi revalidate și blocarea adreselor private/link-local/loopback/metadata pentru surse externe. Ollama și IPC au adaptoare dedicate; nu sunt accesibile prin fetchul web. HTML este sanitizat, fără scripturi. O pagină nu poate adăuga tooluri ori extinde allowlist-ul. Sursa și data sunt păstrate chiar dacă rezumatul este modificat.

## 8. Roblox Studio, Rojo și asseturi

MCP se configurează din instrucțiunile afișate de Studio și negociază capabilitățile reale. Documentația curentă descrie activare în Assistant și transport stdio; alegerea instanței Studio trebuie să fie explicită. Nu presupune endpoint HTTP sau plugin vechi necesar pentru MCP. Conectarea la Studio a acestei conversații nu configurează automat viitoarea aplicație. [Studio MCP](https://create.roblox.com/docs/studio/mcp).

Rojo sincronizează structura descrisă de fișierele proiectului. Pin CLI și plugin compatibile; folosește inițial o copie locală a place-ului. [Rojo](https://rojo.space/docs/v7/).

Politică de ownership: `src/` și subarborii mapați de Rojo sunt modificați pe disc; scenele nemapate pot fi editate prin MCP în limitele planului. Înaintea unui sync, detectează modificările Studio divergente și oprește sincronizarea conflictuală. Snapshot/backup și lock per proiect/Studio previn doi writeri simultani. Nu pretinde rollback universal: fișierele au diff/Git, scenele au backup, iar asseturile deja publicate pot necesita remediere separată.

Adapterul expune operații restrânse: inspect, script diff, scene mutation, playtest și report. Luau arbitrar din model este risc major; pentru MVP se limitează la șabloane revizuite și proiect demonstrativ. Codul jocului rezultat se testează în Studio cu cont/proiect de test, HTTP/DataStore și acces live dezactivate unde se poate. Scripturile din Creator Store se inspectează înainte de inserare; nu se execută module externe doar după ID.

Imaginile OpenAI sunt salvate local cu hash și metadate. Uploadul în Roblox este un pas extern separat, legat de destinație, drepturi și verificarea moderării. Dacă toolul cere URL public, nu publica fișierul automat pe un serviciu temporar: oferă import local/manual sau un canal autorizat explicit.

## 9. GitHub și distribuție

Git local este disponibil independent de GitHub. Repozitoriu privat implicit; branch per run și commit doar în scopul aprobat. Git Credential Manager sau flux oficial cu scope minim, niciodată parolă/token în command line. Hooks din repo-uri importate nu se execută implicit. Nu împinge vaultul personal sau state DB. CI pentru PR-uri rulează fără secrete, iar release-urile au permisiuni distincte.

Installer per-user NSIS `.exe`, creat și testat pe Windows; Tauri permite acest format. Workerul se distribuie separat în același pachet. [Tauri Windows installer](https://v2.tauri.app/distribute/windows-installer/). Modelul Ollama nu se include în installer; se descarcă separat după alegere. Update manual inițial, release cu hash și proveniență; auto-update semnat într-o etapă ulterioară.
