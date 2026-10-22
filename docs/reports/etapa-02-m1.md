# Raport etapa M1 — fluxul local minim

Data: 22 octombrie 2026 · plan aprobat „M1 v1”, buget API 0 USD · ramura `claude/jolly-cray-mqtuzu`.

## Ce s-a livrat

**Stocare (`crates/storage`)**
- SQLite cu WAL și chei străine, în `%LOCALAPPDATA%/RobloxAIBuilder/state.sqlite`.
- Migrații versionate, copie de siguranță înaintea migrării, refuz la o schemă mai nouă.
- Tabele: proiecte, conversații, mesaje, planuri, pași de plan, aprobări, rulări, încercări, evenimente cu secvență, fișiere modificate, rezervări de buget și audit.

**Serviciul aplicației (`crates/app`)**
- **Conversații persistente:** datele demo inițiale vin din același JSON ca în TS. Include căutare fără diferențe de majuscule (și pentru diacritice), oprire/reîncercare cu istoric păstrat și vizibilitate aleasă doar pentru o conversație nouă.
- **„Pregătește un plan”** creează o versiune nouă a planului demonstrativ. Versiunea veche devine „înlocuită”. Titlurile și descrierile pașilor fac parte din hash, deci din ce aprobi.
- **Aprobare:** e legată de hash-ul afișat (refuz dacă planul salvat diferă). Revocarea readuce planul în așteptare și oprește efectele unei rulări active.
- **Rulare:**
  - furnizorul fals propune un `patch_proposal`;
  - executorul scrie fișierele Luau în copia proiectului, numai după policy gate;
  - verificările statice rulează pe fișierele scrise (structura Rojo, fără `require` după ID, fișiere în căile aprobate);
  - raportul `reports/static-checks.md` se scrie prin instrumentul `reports.write`;
  - evenimentele au secvență, iar auditul și consumul de buget (0 USD) se salvează.
- **Pauză, oprire și revocare:** se verifică înaintea fiecărui efect. O singură rulare pe PC.
- **Repornire:** o rulare întreruptă trece în „recuperare necesară”. „Verifică și continuă” inspectează fișierele: pasul deja făcut nu se repetă, pasul neterminat se reia în siguranță.
- **Fișierele editate de utilizator nu se suprascriu:** conflictul face pasul să eșueze; după 3 încercări rularea eșuează.
- **Rezultate oneste:**
  - „Finalizat” doar dacă toate testele au trecut;
  - testele Studio sunt „neexecutate”, iar verdictul devine „Necesită verificare”;
  - diff pe fișiere și limitările sunt incluse.

**Shellul Tauri**
- 23 de comenzi tipizate peste serviciu.
- Evenimentele `rab://run-event` și `rab://reply`.
- Baza de date în `%LOCALAPPDATA%/RobloxAIBuilder`.
- Permisiuni adăugate: doar `core:event:allow-listen` și `allow-unlisten`.

**Interfață**
- `TauriWorkspaceGateway` (real) și gateway-ul demo (browser), cu același contract.
- Ecranele **Plan**, **Execuție** și **Rezultate** după v2, integrate în shell-ul v5:
  - **Plan:** metrici, tabelul pașilor, teste (automat/Studio), permisiuni, semnătură cu amprentă, „Revocă aprobarea”, „Pornește execuția”.
  - **Execuție:** progres calculat din pașii terminați, jurnal, Pauză/Continuă/Oprește.
  - **Rezultate:** verdict, teste ✓/✕/–, limitări, diff pe fișiere.
- Fereastra „Recuperare după repornire”.
- Cardul „Plan curent” în panoul din dreapta și „Plan și sarcini” activ în bara laterală.
- Cardul de plan din chat deschide planul.
- Datele ISO (mesaje, conversații) se afișează în română.

## Verificări rulate în Cloud (Linux)

| Verificare | Rezultat |
|---|---|
| `cargo fmt --all --check` | trecut |
| `cargo clippy -p rab-core -p rab-storage -p rab-app --all-targets -D warnings` | trecut |
| `cargo test` (core + storage + app) | 78 trecute, 0 eșuate; testele de flux rulate de 5 ori, stabile |
| `prettier`, `eslint`, `tsc`, `vite build` | trecute |
| `vitest run` | 34 trecute (inclusiv fluxul plan → aprobare → execuție → rezultate, IPC Tauri simulat și contractul DTO) |
| `playwright test` | 11 trecute: ecranele noi la 1440×900 și 1280×720, recuperare; zero încălcări CSP, zero cereri externe |
| `tauri.conf.json` față de schema oficială 2.12.5 | valid |
| API-uri și permisiuni Tauri | verificate în sursele `tauri` 2.12.5 (nu compilate) |

Matricea de securitate, acoperire nouă:

| ID | Ce s-a verificat |
|---|---|
| S02 | Aprobarea rămâne legată de hash după repornire. Revocarea în timpul rulării oprește al doilea fișier, iar rularea nu se mai poate porni fără o aprobare nouă. Aprobarea cu alt hash decât cel salvat este refuzată. |
| S07 | 12 conexiuni SQLite separate rezervă concurent: exact 3 × 0,30 $ în 1,00 $. O rezervă incertă nu se eliberează. |
| S11/S12 (în proces) | O rulare întreruptă devine `recovery_required`. Efectul existent este confirmat și nu se repetă; efectul lipsă se reia. Nu se pornește o rulare nouă peste una nereconciliată. |
| S17 (parțial) | Un fișier editat de utilizator nu este suprascris; pasul eșuează. |
| S20 | După „Oprește” nu mai apare niciun fișier nou. Pauza nu scrie nimic. |

## Neexecutat (necesită Windows)

- `npm run tauri dev`:
  - compilarea shellului cu cele 23 de comenzi;
  - crearea bazei în `%LOCALAPPDATA%\RobloxAIBuilder`, inclusiv la căi cu diacritice;
  - demo-ul cap-coadă în aplicația reală;
  - evenimentele `listen` în WebView2.
- Repornirea reală a aplicației în timpul unei rulări (testată în Cloud doar prin simularea stării în baza de date).
- Junction, ADS și cursele între validare și scriere pe NTFS (S03 complet, S04).

## Limite

- Furnizorul este fals (răspuns prestabilit); nu există model real, Studio, Git sau worker separat.
- Agenții, sarcinile, echipa și calculatoarele din panoul drept rămân date demonstrative.
- Identitatea aprobării nu este autentificată (M2).
- Rularea se oprește la închiderea ferestrei; continuarea în fundal vine cu worker-ul (M2).

## Următorul pas propus

M2: `worker.exe` separat (serviciul din `crates/app` mutat acolo), named pipe cu ACL pe SID, single-instance, tray (Redeschide/Pauză/Oprește tot), continuare după închiderea UI, aprobare legată de identitatea Windows, vault Markdown cu detectarea conflictelor de editare. Necesită Windows.
