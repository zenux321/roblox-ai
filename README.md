# Roblox AI Builder pentru Windows

**Design primit:** [ghidul exportului](design/README.md) și [predarea către Claude Code](docs/DESIGN_HANDOFF.md). Versiunea v5 este referința principală; v2/v4 sunt păstrate pentru ecrane complementare. Acestea sunt prototipuri, nu aplicația funcțională.

## Actualizare 0.2 — cerințe noi aprobate ca direcție

Citește întâi [cerințele consolidate](docs/PRODUCT_UPDATE.md), apoi [chat și asseturi](docs/CHAT_AND_ASSETS.md), [colaborarea prin VPN](docs/COLLABORATION.md) și [Blender MCP](docs/BLENDER_MCP.md). Acestea completează pachetul inițial: designul existent se păstrează; colaborarea este gazdă/participant prin Radmin/Hamachi, fără Supabase obligatoriu; workerii pot împărți sarcini independente. Nu sunt încă implementate.

Exemplu de configurare nou: [colaborare, chat și Blender](config/collaboration.example.json). Pentru Claude Code folosește [promptul actualizat de continuare](prompts/02-CONTINUE-EXPANDED-SCOPE.md).

Pachet inițial de specificații și configurare • 22 octombrie 2026 • versiunea 0.1-draft.

Aplicație desktop pentru a transforma o idee de joc Roblox într-un proiect local verificabil: cercetare, plan aprobat, cod Luau, scene, testare și raport. Gândită pentru autor și câțiva prieteni, cu date locale, Obsidian și Ollama ca bază.

**Starea livrării:** etapa 1 implementată: nucleul Rust (`crates/core`) cu validare strictă și policy engine, plus ecranul „Spațiu de lucru” în React/Tauri cu **date demonstrative** (`apps/desktop`). Vezi [raportul etapei 1](docs/reports/etapa-01.md). Nu există încă worker, persistență, integrări reale sau installer testat.

**Starea inițială a pachetului (0.1):** documentație și exemple, nu aplicație funcțională și nu installer. Nu au fost instalate dependențe, consumate credite de generare, create repository-uri online sau modificate jocuri Roblox. Fișierele JSON sunt exemple pentru viitorul runtime propriu; nu sunt configurații Claude Code executabile.

## Începe aici

1. Citește [PRD](docs/PRD.md), [arhitectura](docs/ARCHITECTURE.md) și [etapele](docs/ROADMAP.md).
2. Verifică [costurile și autentificarea](docs/COSTS_AND_AUTH.md). Bugetul API inițial este **0 USD**.
3. Pune conținutul acestui folder într-un repository GitHub privat, ales de tine. Nu include vaultul personal, chei sau datele runtime. Conectează numai acel repository la Claude Code Cloud.
4. Pornește o sesiune Cloud și copiază [primul prompt](prompts/01-START-CLAUDE-CODE.md). Prima etapă cere un plan concret; implementarea începe după aprobarea lui.
5. Pentru integrarea Windows/Studio, continuă aceeași ramură într-un checkout local separat, cu Claude Code oficial. Testele locale reale sunt obligatorii înainte de distribuție.

Alternativ, începe direct local: deschide folderul cu Claude Code oficial și folosește același prompt. Autentificarea se face în produsul Anthropic. Nu introduce tokenuri de sesiune în configurația aplicației.

## Conținut

| Fișier | Rol |
|---|---|
| `CLAUDE.md` | Reguli de implementare pentru Claude Code |
| `docs/PRD.md` | Cerințe, fluxuri, limite și acceptanță |
| `docs/ARCHITECTURE.md` | Module, date, aprobări, worker și adaptoare |
| `docs/ROADMAP.md` | Livrări Cloud/local și criterii de trecere |
| `docs/COSTS_AND_AUTH.md` | Creditul Cloud, API-uri și politici de buget |
| `docs/SECURITY_TEST_PLAN.md` | Amenințări și teste obligatorii |
| `docs/SETUP_AND_DISTRIBUTION.md` | Configurare, GitHub, installer și prieteni |
| `docs/SOURCES.md` | Surse oficiale și incertitudini |
| `config/agents.example.json` | Roluri logice și permisiuni propuse |
| `config/plan.example.json` | Exemplu de plan care încă nu este aprobat |
| `vault-template/` | Structură Markdown de copiat într-un vault ales |
| `prompts/01-START-CLAUDE-CODE.md` | Promptul primei sesiuni |

## Principii

- Local implicit; fără server central și fără abonament suplimentar obligatoriu.
- Un plan aprobat autorizează toate acțiunile enumerate în limitele lui. Schimbarea domeniului sau a bugetului cere o versiune nouă.
- Agenții propun; executorul determinist verifică permisiunile înainte de fiecare efect.
- Închiderea ferestrei lasă workerul pornit. „Oprește tot” oprește lucrul. Sleep, oprirea PC-ului și logout nu înseamnă execuție continuă.
- Fiecare prieten își folosește propriile conturi și chei opționale.
- Claude Code Cloud construiește aplicația; nu este backendul permanent al aplicației.

„Ambele variante” din conversația veche nu avea întrebarea asociată disponibilă. Specificația acoperă atât joc nou cât și proiect existent, plus dezvoltare Cloud și local, ca presupuneri explicite. Răspunsurile vechi ale asistentului nu au putut fi recuperate ca text.

Licența codului viitor va fi aleasă de proprietar înainte de publicare; acest pachet nu declară licențe asupra Roblox, Claude Code, modelelor sau asseturilor terțe.
