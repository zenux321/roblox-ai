# Raport etapa 1 — fundația (M0) și ecranul „Spațiu de lucru” (C0, primul pas)

Data: 22 octombrie 2026 · plan aprobat: „Etapa 1 v1”, buget API 0 USD · ramura `claude/jolly-cray-mqtuzu`.
Decizia utilizatorului: fără pachete Linux (webkit2gtk) și fără CI Linux.

## Ce s-a livrat

**Nucleul Rust (`crates/core`)**: testabil fără Windows, fără rețea, secrete sau shell.

- Configurații `agents` v0.1 și `collaboration` v0.2, cu parsare strictă (`deny_unknown_fields`, enum-uri și instrumente necunoscute respinse).
- Validare în două moduri:
  - `Contract`: invariante de siguranță, pe care exemplele le respectă.
  - `Execution`: în plus, respinge `example_only`, placeholder-ele `<...>` și funcțiile neimplementate (host/join, sarcini remote, Blender).
- Planul are hash SHA-256 pe JSON canonic. Aprobarea e legată de hash, versiune și identitatea celui care aprobă și poate fi revocată. Un document cu `approval` completat de mână este respins.
- Policy gate verifică, în ordine:
  - anularea rulării;
  - că configurația nu e exemplu;
  - proiectul;
  - aprobarea și hash-ul;
  - sarcina și instrumentele ei;
  - căile (lexical: `..`, absolute, UNC, `\\?\`, ADS, nume rezervate, nume scurte 8.3, fără diferențe de majuscule);
  - rețeaua: https, hostname public, allowlist și destinație din plan;
  - efectele externe: flag global, scope de plan și destinație exactă. Publicarea și uploadul rămân blocate, fiindcă schema 0.1 nu are place/asset exact.
- Furnizori:
  - `authorize_provider_call` aplică regulile: furnizor dezactivat, model cloud Ollama, apeluri plătite oprite, preț necunoscut, buget 0.
  - `BudgetLedger` face rezervări atomice; o rezervare incertă nu se eliberează automat.
- Furnizor fals (spy) și `GuardedProvider`.
- Mașina de stări plan/run:
  - un task reușit nu se reexecută;
  - reîncercările sunt limitate;
  - după restart, sarcinile în curs trec în `recovery_required`;
  - reluarea cere o reconciliere explicită.
- Executor demonstrativ care aplică un patch text numai după policy gate:
  - verifică rădăcina planului, refuză symlink-urile și hash-ul de bază vechi;
  - scrie temporar, apoi face rename;
  - păstrează audit.
- Scheme JSON generate în `schemas/`, cu test de sincronizare.

**Interfața (`apps/desktop`)**: ecranul v5 „Spațiu de lucru” recreat în React. Prototipul nu este importat.

- Bara de titlu are insigna „Mod demonstrativ · date fictive” și pastilele „Lucru individual · fără VPN”, „Studio · neconfigurat” și „Cost 0,00 $”. Controalele ferestrei apar doar în Tauri.
- Bara stângă:
  - selector de proiect (acțiunile viitoare sunt dezactivate, cu explicație);
  - căutare în titluri și conținut;
  - conversații Comună/Privată și „Fără proiect”;
  - navigare dezactivată către etapele următoare;
  - restrângere 280 ↔ 64.
- Chat:
  - redenumire cu Enter/Esc;
  - mesaje de persoană (bule), agent (card extensibil) și asistent (paragrafe, listă, cod cu Copiază → Copiat, etichete);
  - atașamente demo cu „Analizat de AI” sau „Doar atașat”;
  - cardul OpenAI Images în starea „Neconfigurat”;
  - „Pregătește un plan” → card de plan care așteaptă aprobarea;
  - stări: încărcare, conversație goală (vizibilitate Privată/Comună doar pentru o conversație nouă), „Citesc contextul…”, scriere progresivă, „Răspuns oprit”, reîncercare cu „Încercarea N” (istoricul rămâne).
- Composer:
  - 2–8 rânduri; Enter trimite, Shift+Enter face rând nou;
  - Trimite / Oprește răspunsul;
  - selector de model (furnizor demo; Ollama neconfigurat; Claude API cu avertisment, fără fallback plătit);
  - selector de context din vault;
  - linia de vizibilitate.
- Panoul drept, restrângibil 380 ↔ 0 (340 sub 1440 px), are filele Proiect, Agenți, Sarcini și Echipă. Conexiunile sunt marcate „demo” sau „neconfigurat”.
- Meniul „Demo: stări”: Normal, Încărcare lentă, Conversație goală, Model lipsă.
- Textele sunt în `src/i18n/ro.json`, iar tokenurile din DESIGN_HANDOFF în `src/theme/tokens.css`. Fonturile sunt locale.

**Shellul Tauri (`apps/desktop/src-tauri`)**:
- CSP strictă (și `devCsp` pentru Vite);
- `freezePrototype`;
- capabilities minime;
- comanda `runtime_info`, care afișează verdictul nucleului pentru configurațiile exemplu;
- installer NSIS per-user configurat;
- iconuri generate local.

## Reconcilieri aplicate față de prototip

| În prototipul v5 | În implementare |
|---|---|
| „Studio conectat · PC-ul meu” | „Studio · neconfigurat”, fără stare fictivă |
| „Găzduiești · 1 conectat · Sincronizat” | „Lucru individual · fără VPN” (C1 nu există încă) |
| Conexiuni bifate (✓) în fila Proiect | Marcate „demo” sau „neconfigurat” |
| Model „qwen2.5-coder 7B · local” | „Furnizor demo · răspunsuri prestabilite”, iar mesajele poartă eticheta „model simulat” |
| Port 47820, cod de 10 minute, „Nu ai fost taxat” | Neafișate; schema impune un TTL ≤ 300 s |
| Atașare, transferuri, lightbox, host/join, invitații | Dezactivate, cu explicația etapei |

## Verificări efectiv rulate (Linux, în Cloud)

| Verificare | Rezultat |
|---|---|
| `cargo fmt --all --check` | trecut |
| `cargo clippy -p rab-core --all-targets -- -D warnings` | trecut |
| `cargo test -p rab-core` | 57 trecute, 0 eșuate |
| `prettier --check`, `eslint`, `tsc --noEmit` | trecute |
| `vitest run` | 20 trecute |
| `vite build` (cu CSP meta) | trecut |
| `playwright test` | 8 trecute: capturi la 1920×1080, 1440×900, 1280×720; panouri restrânse; text lung; navigare cu tastatura; stări. Zero încălcări CSP, zero cereri externe |
| Prototipul v5 randat pentru comparație | trecut (biblioteci din registry-ul npm, identice cu unpkg; unpkg este blocat de proxy) |
| `tauri.conf.json` validat cu schema oficială 2.12.5 | valid |
| `npm audit --omit=dev` | 0 vulnerabilități |
| Scanare simplă pentru prefixe de chei API/GitHub/AWS și chei private | nicio potrivire (nu este un audit complet) |

Matricea de securitate acoperită parțial, prin teste unitare și de integrare:

| ID | Acoperire și criteriu verificat |
|---|---|
| S02 | Plan modificat după aprobare, inclusiv după reîncărcare → `PlanHashMismatch` |
| S03 | Validare lexicală completă; symlink pe Unix refuzat. Junction și ADS reale pe NTFS: neverificate |
| S06 | Furnizor dezactivat, plăți oprite, buget 0, model cloud → spy-ul confirmă 0 apeluri |
| S07 | 16 rezervări concurente → exact 3 × 0,30 $ în 1,00 $; rezerva incertă nu se eliberează |
| S11/S12 | Starea după restart: `recovery_required` și reluare doar după reconciliere. Procese reale: neverificate |
| S19 | Textul `<img onerror>` și `<script>` se randează ca text; CSP strictă |
| S20 | După anulare, niciun tool call nou și niciun fișier scris |
| S22 | Publish/push fără plan → refuz; push cu repo exact și aprobare → permis; alt repo → refuz |

Capturi incluse în `docs/reports/etapa-01/`. Restul se generează local cu `npm run test:visual`.

## Neexecutat (necesită Windows)

- `cargo build --workspace` / `npm run tauri dev`: compilarea shellului Tauri, WebView2, bara de titlu custom (drag, minimize, maximize, close), CSP-ul aplicat de Tauri.
- `npm run tauri build`: installer NSIS, căi cu spații și diacritice.
- S03 complet și S04 (junction, ADS, cursă între validare și scriere) pe NTFS.
- Persistență, IPC, worker, tray și restart: nu există încă (M1/M2).

## Limite

- Toate datele UI sunt demonstrative și se pierd la închidere. Nicio integrare nu este reală.
- Validarea căilor este lexicală. Deschiderea pe handle și backup-ul aparțin worker-ului.
- Detectarea modelelor cloud Ollama este o euristică pe nume; verificarea prin metadate vine în M3.
- Protecția DNS/IP privat la conectare (S08 complet) aparține adaptorului de fetch din M5.

## Următorul pas propus

M1: SQLite cu migrații (proiecte, conversații, mesaje, planuri, aprobări, rulări, audit), ecranele Plan, Aprobare și Execuție legate de `rab-core` prin comenzi Tauri tipizate, persistența conversațiilor după restart și demo cap-coadă cu patch în fixture. Necesită build și verificare pe Windows.
