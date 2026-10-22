# Plan etapizat pentru Claude Code Cloud și local

## Extinderea scopului 0.2

Etapele inițiale M0–M6 rămân fundația. Estimarea lor inițială nu include extensiile de mai jos; acestea se estimează după spike-urile de conectivitate și adaptor. Pachetul complet cerut acum include și C0–C3 și B1, cu Blender dezactivabil. Nu marca produsul complet doar după M6.

| Etapă nouă | Dependență și mediu | Livrabil / gate |
|---|---|---|
| C0 — chat și design | M1–M2; Cloud + Windows UI | Integrează designul real furnizat, conversații/audiențe, preview, context și mock thumbnails; istoric/retry/stop și privat→comun testate |
| C1 — host/join | M2; core Cloud, două PC-uri Windows | Pairing TLS, fingerprint, ACL, revocare, jurnal și reconnect; chat comun funcțional pe VPN, niciun port public/MCP expus |
| C2 — fișiere și conflicte | C0+C1 | Transfers resume/hash, vault selectat, outbox offline, conflicts și Git branch workflow; transfer mare întrerupt și pierdere de membership testate |
| C3 — workeri distribuiți | M3+C2 | Capabilities, consimțământ device, plan comun + grant local, lease/fencing, rezultate validate; crash/partition fără execuție duplicată a efectelor |
| B1 — Blender opțional | M4, C3 pentru scenariul remote | ADR server verificat, adaptor restrâns, copie scenă, preview/export/import; gate Python/izolare și probe Windows/Studio |
| R1 — release extins | Etapele aplicabile de mai sus | Installer upgrade, firewall per adaptor, două PC-uri, testele S23–S32 și documentație pentru host/join |

API-ul real pentru thumbnails se conectează în M5, după mockul C0; teste plătite doar cu buget. C3 nu necesită API cloud. B1 poate fi livrat separat, dar interfața trebuie să arate „neconfigurat” până la validarea adaptorului. R1 trebuie să testeze proiect existent și vault real de test, nu doar obby nou.

Prima sesiune după această actualizare citește promptul 02-CONTINUE-EXPANDED-SCOPE.md. Construiește fundația pe etape; cerințele noi nu autorizează cheltuieli, publicare sau instalări în afara planului aprobat.

Etapele sunt backlog, nu autorizare pentru toate operațiile viitoare. Se aprobă planul concret al etapei; după aprobare, Claude Code execută sarcinile incluse fără confirmări repetitive. Estimările sunt orientative pentru un dezvoltator asistat de AI, nu conversie a creditelor în timp garantat.

| Etapă | Mediu | Livrabil și criteriu de trecere | Estimare |
|---|---|---|---|
| M0 — contracte și schelet | Cloud, apoi build local | Workspace Rust/TS, config schema, provider fake, policy core, teste offline, CI fără secrete; exemplul config valid și acțiunile neaprobate respinse | 1–3 zile |
| M1 — flux local minim | Cloud + Windows | UI plan/aprobare/execuție, SQLite, patch în fixture, raport; demo cap-coadă fără rețea și fără chei | 3–5 zile |
| M2 — worker și vault | Windows, core în Cloud | Worker independent, IPC, tray, crash recovery, Markdown și conflicte; închidere UI și restart fără pierdere/dublare | 3–6 zile |
| M3 — Ollama și setup | Windows | Wizard hardware, download controlat, alegere model, health, timeouts; generare reală pe PC și lipsa fallbackului plătit | 2–4 zile |
| M4 — Roblox/Rojo | Windows + Studio | Proiect demo, sincronizare controlată, MCP discovery, playtest și rapoarte; obby verificat cu două personaje/clienți unde este relevant | 4–8 zile |
| M5 — cercetare și API opționale | Cloud mock + local real | Surse/citări, buget atomic, Claude API și imagini OpenAI opționale; teste de provider cu mock, smoke real numai cu buget aprobat | 3–6 zile |
| M6 — distribuție prieteni | Windows curat/VM | Installer per-user, uninstall/upgrade, release GitHub, SBOM, raport securitate; test pe minimum două PC-uri și niciun secret în pachet | 3–5 zile |

Total orientativ: 19–37 zile efective, cu posibilă extindere pentru compatibilitate Studio, hardware și semnare. M0–M4 formează MVP; API-urile nu blochează funcțiile locale. Rezervă de timp pentru defecte, fără promisiune că 250 USD ajung pentru toate etapele.

## M0 — prima sesiune

1. Confirmă starea Git și citește specificațiile; listează doar ambiguitățile blocante.
2. Verifică versiuni oficiale ale toolchainului, decide pachetele minime și creează lockfile-uri.
3. Definește schemele pentru config, plan, approval, tool calls, usage și run events. `agents.example.json` nu este un format vendor.
4. Implementează policy engine pur: allowlist, plan hash, revocare, provider disabled, buget 0, target mismatch.
5. Implementează executor fake și un patch limitat la fixture; fără shell arbitrar.
6. Creează teste unitare pentru contracte și CI Linux/Windows dacă bugetul contului permite. Build Windows poate fi executat local pentru a evita minute plătite.
7. Raportează testele trecute, cele indisponibile și deciziile ADR. Nu adăuga secret sau integrare live pentru a demonstra scheletul.

## Strategie Cloud → local

Cloud lucrează pe documentație, core, UI, schema, mockuri și teste independente de desktop. Sesiunea folosește repository-ul ales și o ramură distinctă. Nu încărca vaultul personal, cheile sau place-uri private fără necesitate și autorizare.

La trecerea locală: salvează/împinge ramura aprobată, verifică diff-ul, descarcă în checkout dedicat și continuă cu Claude Code oficial. Rulează build/test pe Windows, apoi integrarea Studio. Cloud nu este dovadă că named pipes, DPAPI, installerul sau Studio funcționează. [Sesiuni Cloud](https://code.claude.com/docs/en/claude-code-on-the-web).

Nu expune Studio, Ollama sau workerul pe internet pentru a le face accesibile sesiunii Cloud. Nu folosi un runner personal cu secrete pentru PR-uri neîncrezute. Munca de dezvoltare Cloud și joburile aplicației sunt sisteme separate.

## Demonstrația MVP

Input: „Creează un obby de test cu cinci platforme, checkpoint și finish, fără publicare.” Output: plan aprobat, fișiere Luau și Rojo, backup local, modificări în Studio de test, teste pentru checkpoint/respawn/finish și raport. Include un test de cerere RemoteEvent falsificată: serverul respinge progresul nevalid. Fără DataStore real, reclame, achiziții sau publicare.

## Definition of Done pe etapă

Livrabilul este inspectabil, testele relevante au rezultat și dovezi, regresiile critice sunt rezolvate, configurația implicită nu cheltuie bani, documentația reflectă implementarea. Funcțiile de Windows/Studio fără test real rămân „neverificate”. Nu se marchează etapa completă doar pentru că modelul a spus că funcționează.

Gate release: toate testele critice din SECURITY_TEST_PLAN trecute, niciun defect critic/high deschis, scanarea secretelor curată, installer testat, recuperarea verificată și instrucțiuni de dezinstalare. Dacă un test este amânat, funcția asociată rămâne dezactivată în release.
