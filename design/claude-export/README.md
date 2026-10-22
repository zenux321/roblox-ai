# Handoff: Roblox AI Builder — interfața desktop (Windows)

## Prezentare
Interfața aplicației desktop care ajută un utilizator fără experiență tehnică să creeze jocuri Roblox cu AI: descrie ideea în română → primește un plan → îl aprobă → urmărește execuția → verifică testele și modificările. Acoperă cerințele din `docs/PRD.md` și `docs/ARCHITECTURE.md` (repo `zenux321/roblox-ai`).

## Despre fișierele de design
Fișierele din acest pachet sunt **referințe de design create în HTML**: prototipuri care arată aspectul și comportamentul dorit. **Nu sunt cod de producție de copiat.** Sarcina este să **recreezi aceste ecrane în stackul țintă: Tauri 2 + React/TypeScript**, cu motorul și workerul în Rust (vezi ARCHITECTURE.md). Toate datele din prototip sunt demonstrative; nu există apeluri reale.

Deschide `Roblox AI Builder v2.dc.html` într-un browser. `support.js` trebuie să stea în același folder. Prototipul e gândit pentru 1920×1080, cu minimum 1280×720. Butonul **„Demo: ecrane și stări”** din colțul dreapta-jos sare direct la orice ecran și la orice scenariu de eroare.

## Fidelitate
**Hi-fi.** Culorile, tipografia, spațierea și copy-ul sunt finale. Recreează-le cât mai exact, cu componente React reutilizabile. Textele UI sunt în română și trebuie păstrate verbatim. Pune-le într-un fișier de resurse `ro.json` ca să poată fi traduse mai târziu.

---

## Acoperire PRD / ARCHITECTURE → UI

| Cerință | Unde în UI |
|---|---|
| R1 Cercetare (Google, YouTube, Reddit, Creator Hub), proveniență, import manual | Plan → „Surse folosite și încredere”; Setări → „Cercetare și surse”; Memorie → Surse |
| R2 Joc nou / proiect existent; local / cloud | Configurare pas 4; ecranul „Continuă un joc existent”; Setări → Model AI |
| R3 Instalare/configurare automată, UAC, Studio interactiv | Configurare pașii 2–3 (detectare hardware, componente, licențe, MCP) |
| R4 Per-user, chei individuale | Setări → Chei și conturi |
| R5 Aprob planul, apoi execută tot | Fila Plan + panoul din dreapta; semnătura aprobării; „Revocă aprobarea”; scenariul „Aprobare nouă necesară” |
| R6 Continuă cu aplicația închisă | Setări → Lucru în fundal; overlay „Tray” (Redeschide / Pauză / Oprește tot); scenariul „Recuperare după restart” |
| R7 Ollama implicit, API plătit opt-in | Setări → Model AI (Ollama ↔ Claude API, cheie, buget; buget 0 = niciun apel plătit) |
| R8 Vault Obsidian | Ecranul Memorie (Inbox/Projects/Sources/Decisions/Plans/Runs/Assets, frontmatter, conflicte de editare) |
| R9 Imagini OpenAI | Setări → Buget → „Imagini generate cu OpenAI”; Memorie → Resurse |
| R10 Studio MCP / Rojo, GitHub, exe | Ecranul Roblox Studio; Rezultate → Commit local / Creează PR / Publică în Roblox (pas separat) |
| Roluri: planner, researcher, builder, reviewer, memory curator | Fila Agenți; coloana „Agent” din Plan |
| Stări Run: queued/running/paused/waiting_dependency/waiting_approval/recovery_required/succeeded/failed/cancelled | Panoul din dreapta, Execuție, scenariile Demo |
| „Finalizat” doar cu testele trecute; testele neexecutate etichetate | Rezultate (✓ trecut, ✕ eșuat, – neexecutat); scenariul „Test eșuat” |
| Lipsa Studio suspendă doar sarcinile dependente | Scenariul „Studio deconectat” (pașii 3–4 continuă, pașii 5–6 așteaptă) |
| Lipsa modelului → explicație, fără fallback plătit | Scenariul „Modelul local lipsește” |
| Conflict Rojo / modificări Studio divergente | Scenariul „Conflict Rojo” |

---

## Structură generală (shell)
- **Bara de titlu** (44px, `#0b0d0f`, border-bottom `#1f2328`). Conține logo-ul de 22px (fundal `#b6f36b`, text „AI”), numele aplicației și breadcrumb-ul. În dreapta sunt pastilele de stare (28px, radius 999, fundal `#15181c`, border `#262b31`, 12px): „Fără internet” (doar offline), **Studio** (punct `#5ee08a` sau `#f5b94a`; click deschide ecranul Studio) și **Cost estimat** (valoare mono; roșie peste limită; click deschide Setări → Buget). Urmează controalele de fereastră Windows (46px lățime; hover pe ✕: `#c42b1c`).
- **Bara laterală** (272px, `#0b0d0f`): Proiecte (+ Nou), Memorie (4 foldere cu numărătoare), Fișiere (fișierele vizate de plan, tag nou/modif.). Jos: Roblox Studio (cu punct de stare), Setări și linia modelului activ.
- **Zona principală:** un ecran sau spațiul de lucru al proiectului.

### Spațiul de lucru al proiectului
Grid `minmax(0,1fr) 400px`.
- **Antet:** numele proiectului (20/600), subtitlul „Lucrezi pe o copie…”, chip-uri „Memorie: …” și „Copie de rezervă: …”. Filele sunt Conversație · Plan · Execuție · Agenți · Rezultate: 14/500, cea activă are subliniere de 2px `#b6f36b`; un punct de 6px semnalează o stare care cere atenție.
- **Bannere** de stare (Conversație și Plan): card radius 12, titlu 14/600 colorat, text 13, buton secundar și buton principal (fundal `#e8eaed`). Tonuri: avertizare (`#241f12` / `#57471d`), eroare (`#241515` / `#6b2a2a`), info (`#15181c` / `#2c3238`).
- **Panoul din dreapta „Plan de lucru”** (`#121518`) e mereu vizibil. Conține: antetul cu insigna de stare, 3 metrici (Durată / Cost estimat / Fișiere), progresul (activitatea curentă, procent mono, bară de 6px), lista de pași și note (față de v1, date trimise, publicare). Acțiunile se schimbă după fază:
  - `awaiting`: **Aprobă planul** (46px, `#b6f36b`; dezactivat peste buget), Pauză/Oprește dezactivate, notă explicativă
  - `running`: **Pauză** / **Oprește** (contur roșu `#6b2a2a`, text `#f87171`)
  - `paused`: **Continuă** / Oprește
  - `waiting` | `recovery`: **Vezi ce trebuie făcut** (`#f5b94a`) / Oprește
  - `done`: **Vezi rezultatele** și Resetează (demo)

## Ecrane
1. **Pornire:** se verifică pe rând motorul, baza de date, Ollama și Studio (cerc ✓ sau !), cu o bară de 4px.
2. **Configurare** (5 pași, coloană stânga de 340px):
   - Bun venit.
   - Verificarea PC-ului: RAM, spațiu, GPU și viteza AI, verdictul „Merge, dar mai încet” și detaliile tehnice la cerere.
   - Componente: Git, Rojo, Ollama și model, cu licență și mărime; MCP-ul Studio necesită confirmarea „Am activat”; instalare simulată.
   - Proiect și memorie: nou sau import, calea vault-ului, comutatoarele pentru copia de rezervă și lucrul în fundal.
   - Recapitulare.
3. **Proiecte:** grid `auto-fill minmax(320px,1fr)` de carduri (tip, stare, descriere, plan, fișiere, dată), plus cardul „Proiect nou”. Butonul „Importă joc existent” deschide ecranul 4.
4. **Continuă un joc existent** (4 pași orizontali):
   - Alege jocul: folder Rojo sau .rbxl, plus calea.
   - Ce am găsit: Rojo, scripturi, Git, hook-urile neexecutate, notele din vault și comutatorul de asociere.
   - Copie de lucru: ramura, regula de sincronizare, blocarea și copia de rezervă obligatorie.
   - Gata: deschide Prezentarea proiectului.
5. **Prezentare proiect** (proiect existent): tip, stare, folder, vault, ramură; avertisment pentru un test neexecutat; „Ce știe asistentul” (note din vault); istoricul planurilor; caseta „Ce vrei să faci mai departe?” cu idei rapide. Trimiterea arată starea de încărcare, apoi un răspuns.
6. **Conversație:** mesajele utilizatorului sunt bule în dreapta (`#1b1f24`, radius 14/14/4/14). Asistentul are avatar de 28px și text 15/1.6, cu chip-uri de surse („din cache” când e offline). Indicatorul de încărcare e text pulsatoriu plus bare schelet. Composerul are câmp de 40px și butonul Trimite (activ când există text; dezactivat dacă modelul lipsește).
7. **Plan (aprobare):**
   - Titlu, obiectiv și versiunile planului.
   - 4 metrici: durată, cost, limită, reparații.
   - Tabelul de pași (#, Ce face, Agent, Unde, Fișier/zonă, După).
   - Teste obligatorii; Permisiuni și date (fișiere, Studio, model, date trimise, Git, publicare).
   - Surse și încredere; diferențele față de v1.
   - Acțiuni: Aprobă planul vN / Cere modificări. După aprobare apare semnătura (utilizatorul Windows, data, amprenta sha256) și butonul **Revocă aprobarea**.
8. **Execuție:**
   - Cardul jobului curent: activitatea la 22px, procentul mono la 56px, metrici.
   - Cardul de blocaj, după scenariu, cu două acțiuni.
   - Lista de pași și jurnalul mono (`#0b0d0f`).
   - Nota despre lucrul în fundal.
   - Stare goală înainte de aprobare.
9. **Agenți:** carduri pentru Planificator, Cercetător, Constructor, Verificator și Curator memorie (stare, rol, „Acum”, domeniu, model). Dedesubt, limitele comune: context, timeout, reparații, verificare.
10. **Rezultate:**
    - Schelet de încărcare la prima deschidere (~1,4 s).
    - Verdict: Finalizat (verde) sau Necesită verificare (galben), cu durată, cost și modificări.
    - Teste (✓ / ✕ / –), Limitări.
    - Diff cu file pe fișiere (+/−), numerotare și colorare pe rânduri.
    - Acțiuni: Acceptă livrarea / Cere o reparație / Acceptă cu testul eșuat, Deschide în Studio, Commit local, Creează PR (dezactivat dacă GitHub nu e conectat), Publică în Roblox… (deschide fereastra de publicare).
11. **Memorie:** 3 coloane (foldere 240px / note 340px / previzualizare). Previzualizarea are frontmatter mono (`id`, `kind`, `project_id`, `source_url`, `accessed_at`, `trust`) și corpul notei. Conflictul de editare are două opțiuni. Mai sunt indexul local cu „Reconstruiește” și nota despre fragmentele trimise.
12. **Roblox Studio:**
    - Starea conexiunii MCP (OK sau Reconectează).
    - **Alegerea explicită a instanței Studio** (radio).
    - Capabilitățile negociate și amprenta instrumentelor.
    - Rojo: versiuni compatibile, sincronizare, blocare.
    - Pașii de activare MCP; siguranța la testare.
13. **Setări** (navigare stânga de 260px):
    - **Model AI:** cardurile Ollama și Claude API. Pentru Ollama: modele, „Doar local”, Verifică / Refă testul. Pentru Claude: cheie (lipsă sau salvată), model Sonnet sau Haiku, limită pe plan (0 / 0,50 / 2 / 5 $), eroare la buget 0, nota despre datele trimise. Linia de stare arată furnizorul activ; Claude devine activ doar cu cheie **și** buget > 0.
    - **Buget:** limite, consum lunar, oprire automată, imagini OpenAI.
    - **Cercetare și surse:** surse comutabile, domenii permise, import manual.
    - **Lucru în fundal.**
    - **Chei și conturi:** chei ascunse implicit, Arată/Ascunde, Șterge; GitHub prin Git Credential Manager.
    - **Date și confidențialitate:** căi, export fără chei, ștergere index/cache, ștergere totală care păstrează vault-ul.
14. **Ferestre suprapuse:** Recuperare după restart; Publicare în Roblox (destinație exactă, creează un plan separat); **Tray** (simularea ferestrei închise: popover cu progres, Redeschide / Pauză / Oprește tot).

## Interacțiuni și stări
- Fazele planului/run-ului (`phase`): `awaiting → running ⇄ paused`, `running → waiting | recovery → running`, `running → done`, orice fază activă → `stopped`. În prototip, progresul avansează la fiecare 350 ms; în aplicația reală vine din `SubscribeEvents(afterSequence)`.
- Scenarii demo (prop `scenariu` sau meniul Demo):
  - `normal`
  - `studio_deconectat`: oprire la pasul 5
  - `model_lipsa`: composer dezactivat, aprobarea duce în `waiting`
  - `test_esuat`: 2 trecute, 1 eșuat, 1 neexecutat
  - `buget_depasit`: aprobare blocată; opțiunile „Folosește doar modelul local” sau „Mărește limita (v3)”
  - `conflict_rojo`: la 50%
  - `recuperare`: fereastra de recuperare, pasul 4 „incert”
  - `fara_internet`: surse din cache
  - `aprobare_noua`: la 72% e nevoie de un fișier nou în afara planului, deci se propune planul v3
- Tranzițiile CSS sunt doar `width .35s linear` (barele de progres) și `left .15s` (comutatoarele). Textele de încărcare pulsează cu `rab-pulse` (opacitate .35↔1, 1,2 s).
- Hover: rândurile primesc fundal `#15181c`; cardurile, border `#3a4148`.

## Mapare către API-ul intern (ARCHITECTURE §3)
Aprobă → `ApprovePlan(planHash)` + `StartRun`. Pauză → `PauseRun`. Oprește → `CancelRun`. Rezultate/diff → `GetDiff`. Progresul și jurnalul → `SubscribeEvents`. Revocă → invalidează `Approval`. Reconectarea Studio trece prin discovery MCP, cu instanța aleasă explicit. React nu ține secrete; cheile se afișează doar mascate.

## Componente React recomandate
`TitleBar`, `StatusPill`, `Sidebar`, `ProjectCard`, `Tabs`, `Banner`, `ChatMessage`, `Composer`, `PlanPanel`, `TaskRow`, `MetricCard`, `ProgressBar`, `StatusBadge`, `BlockCard`, `LogView`, `AgentCard`, `TestRow`, `DiffViewer`, `Toggle`, `RadioRow`, `ChipSelect`, `Stepper`, `Modal`, `TrayPopover`, `SkeletonBlock`, `EmptyState`.

## Design tokens
**Culori**
- Fundal aplicație `#0e1012`; fundal adânc (bara de titlu, bara laterală, jurnal) `#0b0d0f`; panou `#121518`
- Suprafață `#15181c`; suprafață ridicată / selectat `#1b1f24`; card metric `#181b1f`; rând activ `#181d22`
- Borduri: `#1f2328` (separatoare), `#23282d`, `#262b31` (carduri), `#2c3238` (inele, butoane), `#3a4148` (puternic, cerc inactiv)
- Text `#e8eaed`; text 2 `#dfe3e7`, `#c3c9cf`; atenuat `#a3abb4`; slab `#8b939c`; dezactivat `#5c636b`
- Accent `#b6f36b` (hover `#c8ff85`, text pe accent `#0e1012`, fundal selectat `#161a12`)
- Succes `#5ee08a` (fundal `#122018`, border `#1f4a2f`)
- Avertizare `#f5b94a` (fundal `#241f12`, border `#57471d`, text pe avertizare `#1a1206`)
- Eroare `#f87171` (fundal `#241515` / `#2a1616`, border `#6b2a2a`)
- Diff: adăugare `rgba(94,224,138,.08)` / text `#cdeed8`; ștergere `rgba(248,113,113,.1)` / text `#f3c1c1`

**Tipografie:** Onest 400/500/600/700 (UI) și JetBrains Mono 400/500 (căi, valori, cod, jurnal), din Google Fonts. Ambele suportă diacriticele ș, ț, ă, î, â. Scară: 11 (etichete uppercase, letter-spacing .08em), 12, 13, 14, 15 (text de bază în chat), 16 (titluri de secțiune), 20, 22, 28, 30/34 (titluri), 44/56 (procente).

**Spațiere:** 2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 24, 28, 32, 40, 48, 56, 64.

**Rază:** 6 (chip-uri mici, rânduri), 8 (butoane mici, rânduri), 10 (butoane, câmpuri), 12 (carduri), 14 (carduri mari, composer), 16 (ferestre modale), 999 (pastile).

**Înălțimi butoane:** 32/34/36 (mici), 38/40/42 (medii), 44/46 (principale). Ținta minimă de click e 32px.

**Umbre:** artboard/modal `0 30px 80px rgba(0,0,0,.6)`; popover `0 20px 50px rgba(0,0,0,.5)`.

## Adaptabilitate
Aplicația umple fereastra; minimum 1280×720.
- Coloanele fixe: bara laterală 272, panoul de plan 400, Setări 260, Memorie 240/340, Configurare 340.
- Conținutul central folosește `minmax(0,1fr)`, `max-width` (780 chat, 1080 plan/execuție, 1240 rezultate) și grid-uri `auto-fill`.
- Recomandare sub 1440px lățime: panoul de plan se strânge la 360; sub 1280, devine sertar care se deschide dintr-un buton din antet.

## Resurse
Nu există imagini sau iconuri externe. Logo-ul e un pătrat de 22px cu „AI”; iconurile sunt forme CSS simple. În implementare, folosește un set de iconuri (de ex. Lucide) și logo-ul final.

## Fișiere
- `Roblox AI Builder v2.dc.html`: prototipul complet, cu toate ecranele și stările (sursa principală)
- `support.js`: runtime-ul necesar pentru deschiderea prototipului
- `Ecran principal.dc.html`: cele două direcții inițiale (1a a fost aleasă), doar ca referință istorică


---

# ACTUALIZARE v4: chat, atașamente, imagini, colaborare

> **Funcții NOI.** Completează PRD.md și ARCHITECTURE.md; nu există încă în repository. Toate sunt **doar design și prototip cu date demonstrative**: fără apeluri API, fără conexiuni reale, fără rețea între PC-uri. Pentru fiecare funcție e nevoie de un ADR sau o extindere a specificației înainte de implementare (vezi „Puncte deschise”).

Fișier principal: `Roblox AI Builder v4.dc.html`. Butonul **„Demo: fluxuri și stări”** din centrul barei de titlu pornește fluxul A, fluxul B și toate stările de mai jos.

## Ce s-a schimbat în spațiul de lucru
- Grid `[stânga 280 | 64 restrâns] [chat minmax(0,1fr)] [dreapta 380 | 0 ascuns]`, cu tranziție `grid-template-columns .2s ease`. Ambele panouri se restrâng pentru mai mult spațiu în chat.
- **Stânga:**
  - Selectorul de proiect, cu „Proiecte recente” și conexiunile păstrate (Studio, vault, GitHub), plus „Joc nou”, „Continuă un joc existent” și „Alătură-te unui proiect”.
  - Căutarea în conversații (titlu și conținut).
  - Conversațiile proiectului, fiecare marcată Comună (`#c4a7ff`) sau Privată.
  - Secțiunea „Fără proiect” (chat general), cu acțiunea „Asociază unui proiect”.
  - Navigare: Plan și sarcini · Modificări · Memorie · Calculatoare · Modele și costuri, cu punct de stare.
- **Dreapta**, file Proiect / Agenți / Sarcini / Echipă:
  - Proiect: conexiunile și „Ce a găsit asistentul”, inclusiv **diferențe** joc ↔ fișiere ↔ notițe.
  - Agenți: stare (Disponibil / Lucrează / Așteaptă / Oprit / Eroare), sarcina curentă, ultimul rezultat, PC-ul.
  - Sarcini: atribuite unei persoane sau unui agent, cu dependențe și progres.
  - Echipă: membri, roluri, permisiuni, invitații, calculatoare.

## Chat (NOU)
- Mai multe conversații pe proiect: creare, redenumire (click pe titlu, Enter salvează, Esc anulează), căutare, istoric. Chat general fără proiect.
- **Persoanele** au avatar rotund (Andrei `#7dd3fc` pe `#1b2a33`, Mihai `#c4a7ff` pe `#251f33`), iar mesajele lor stau în bule `#1b1f24` (radius 4/14/14/14). **AI-ul și agenții** au avatar pătrat verde `#b6f36b` și text fără bulă.
- Răspunsurile sunt formatate: paragrafe, liste cu marcator verde, blocuri de cod (antet cu limbaj, fișier și **Copiază** → „Copiat” timp de ~2 s), etichete precum „Doar explicație · jocul nu se modifică”.
- Activitatea agenților apare în **carduri compacte** extensibile (rol, rezumat, PC), fără loguri brute.
- Câmpul de mesaj:
  - Textarea cu 2–8 rânduri, care crește odată cu textul; Enter trimite, Shift+Enter face rând nou.
  - Lipire de imagini (`onPaste` cu imagine), drag-and-drop cu suprapunere „Lasă fișierele aici”.
  - Butoanele **Trimite**, **Oprește răspunsul** (în timpul scrierii) și **Reîncearcă**.
  - Selectorul de model: Ollama local activ, Claude API neconfigurat, cu explicație.
  - Selectorul de **context din vault**: memorie comună, note personale (debifate implicit), documente selectate.
  - Sub câmp, o linie care spune cine vede mesajul și unde ajung datele.
- **Discuție sau implementare:** mesajele de discuție nu modifică jocul. O cerere de schimbare produce un card „Pregătește un plan”, apoi un card de plan cu „Poate aproba: proprietarul”.
- Stări: încărcare a istoricului (schelet), conversație goală (sugestii și alegerea Privată/Comună), „Citesc contextul…”, scriere progresivă cu cursor, răspuns oprit.

## Atașamente (NOU)
- **Înainte de trimitere** apare un card de 236px: miniatură sau tip, nume, tip și mărime, bară de încărcare de 3px, buton de eliminare, eroare (de ex. `.psd` sau `.blend` neacceptat, cu o alternativă) și avertizare (video neanalizabil).
- **În conversație:**
  - Imaginile (240px) se măresc la click, în lightbox cu Descarcă / Salvează în proiect / Folosește ca referință.
  - Clipurile au player: play/pauză, timp, bară.
  - Documentele și codul au un card cu primele rânduri.
  - Fiecare atașament poartă eticheta **„Analizat de AI”** (verde) sau **„Doar atașat”** (gri), plus cine îl vede și dacă pleacă de pe PC.
  - Pentru formatele neanalizabile se explică limita și se oferă o alternativă (de ex. „Extrage 3 cadre ca imagini”). **Nu se simulează analiza.**
- Butonul „Atașamente (n)” deschide un sertar de 440px cu toate atașamentele, filtre (Toate / Imagini / Video / Documente / Cod) și acțiuni.
- Atașamentele dintr-o conversație privată nu devin comune automat.

## Imagini și thumbnails · Flux A (NOU)
Cardul „Generare imagini · OpenAI Images” are stări:
- `unconf`: „Configurează OpenAI Images” plus import manual. Precizează explicit că abonamentul ChatGPT și creditul Claude Code nu acoperă API-ul.
- `nobudget`: cost > limită; „Mărește limita”.
- `ready`: arată costul estimat, limita aprobată, cine plătește și referințele trimise extern; „Aprobă și generează · 0,16 $”.
- `gen`: schelet pentru 2 imagini.
- `fail`: „Nu ai fost taxat”; Reîncearcă / Importă manual.
- `done`: „Aprobat · cheltuit X din Y”.

Variantele apar în chat ca mesaj `variants`: plăci 16:9 selectabile, cu Mărește / Descarcă / Salvează în asseturi (afișează calea `assets/thumbnails/…`) / Folosește ca referință.

O cerere ulterioară („fă lava mai strălucitoare în varianta B”) generează o variantă nouă **fără o nouă aprobare** cât timp rămâne în limită. Peste limită, cere mărirea limitei.

Deasupra câmpului de mesaj apare o **bandă de istoric** a variantelor. „Publică în Roblox…” e un pas separat și nu se declanșează automat.

## Colaborare · Flux B (NOU)
1. **Invită un colaborator:** fereastra arată ce primește colaboratorul (conversații comune, memorie comună, cod prin GitHub) și ce nu (note personale, PC, foldere, chei, buget). Apare invitația în așteptare (expiră în 23 h), cu Anulează.
2. Acceptarea face din colaborator membru, cu rol **Colaborator**: discută și propune, dar nu aprobă și nu integrează. **Proprietarul** aprobă și integrează.
3. **Calculatoare → „Cere conectarea PC-ului”:** pe PC-ul prietenului apare fereastra de **consimțământ** cu limite: GPU maxim, proiectele permise, folderele, notele inaccesibile, cheile nefolosite. Poate accepta sau refuza.
4. **Plan comun:** tabel cu Sarcină · Cine (persoană sau agent) · Pe ce PC (acțiunea „Mută” pentru agenți, înainte de aprobare) · Ordine (dependențe) · Test · Stare. Mai arată ce se modifică, regulile de după aprobare, costul, datele trimise extern și cine poate aproba.
5. Execuția are progres pe fiecare sarcină. Sarcinile dependente așteaptă rezultatele.
6. **PC offline:** sarcina întreruptă e marcată, iar rezultatul necunoscut **nu se repetă automat**. Opțiuni: așteaptă / mută pe PC-ul meu.
7. **Modificări**, cu stările Local → Trimis pentru verificare → Conflict → Integrat:
   - comparație și diff;
   - „Cine lucrează unde” și avertizarea de suprapunere, cu decizie manuală (nu toate conflictele se rezolvă automat);
   - **codul prin GitHub** e separat de **scena din Studio** (PC și instanță explicite);
   - doar proprietarul combină; altfel, „Acces refuzat”.
8. **Perspectiva colaboratorului** („Vezi ca Mihai”): conversațiile private ale proprietarului sunt ascunse, iar aprobarea și combinarea sunt refuzate cu explicație.

## Memorie, modele, costuri (NOU)
- **Memorie** în trei coloane: comună (vizibilă colaboratorului și agenților lui) / personală (doar tu, cu „Partajează”) / selectată pentru conversație.
- **Modele și costuri:** Ollama local (text, gratuit, datele rămân local) / Claude API (text, plătit, opt-in) / OpenAI Images (imagini, plătit). Pentru fiecare: cine plătește și plafonul. Cheile sunt personale. Nu se cheltuie pe contul altcuiva fără acordul lui. Buget implicit 0 $. Creditul Claude Code Cloud e pentru dezvoltarea aplicației.

## Stări demonstrative incluse
Încărcare, conversație goală, atașament incompatibil, generare eșuată, integrare neconfigurată, buget insuficient, invitație în așteptare, acces refuzat, PC offline, conflict de modificări.

## Componente noi recomandate
`ConversationList`, `ConversationHeader` (redenumire), `VisibilityBadge`, `MessagePerson`, `MessageAssistant`, `AgentActivityCard`, `CodeBlock`, `AttachmentChip` (încărcare/eroare), `AttachmentImage`, `VideoPlayer`, `DocPreview`, `AnalysisLabel`, `AttachmentsDrawer`, `Lightbox`, `Composer` (paste/drop/stop), `ModelPicker`, `ContextPicker`, `ImageGenCard` (6 stări), `VariantGrid`, `VariantHistory`, `InviteDialog`, `MemberCard`, `ConsentDialog`, `ComputerCard`, `SharedPlanTable`, `ChangeList`, `ChangeDetail`, `PerspectiveBadge`, `CollapsiblePanel`.

## Puncte deschise pentru ADR (nu sunt rezolvate de design)
- **Transportul între PC-uri pentru colaborare.** PRD exclude „server de agenți pe internet” și hosting central. Trebuie decis cum se sincronizează conversațiile comune, invitațiile și sarcinile: de exemplu doar prin GitHub, cu o conexiune directă cu consimțământ, sau cu un releu.
- **Identitatea colaboratorilor și semnarea aprobărilor** când există doi utilizatori Windows pe PC-uri diferite.
- **Adaptorul OpenAI Images:** limite pe conversație vs. pe plan, stocarea locală cu hash, moderarea.


## Aliniere la `config/agents.example.json` (v0.1)
Designul v4 urmează acum configurația exemplu:
- **Agenți:** `planner`, `researcher`, `builder`, `reviewer`, `memory_curator`, toți pe `ollama`. Fiecare card de agent arată id-ul, instrumentele și contractul de ieșire: plan_proposal / sourced_notes / patch_proposal / review_report / note_proposal. Agenții **propun**, iar executorul aplică.
- **Runtime:** `max_concurrent_tasks: 1` (afișat ca o sarcină odată pe fiecare PC), `max_agent_steps_per_task: 12`, `max_retries: 2`, `task_timeout_seconds: 600` (10 minute).
- **Buget:** `max_run_usd` și `max_month_usd` sunt 0, `unknown_price_action: deny`, `paid_fallback: false`. Textele din UI spun explicit că nu se trece automat la un serviciu plătit.
- **openai_images:** `enabled: false`, `max_images_per_run: 1`, `quality: low`. Fereastra de configurare are „Imagini maxime pe rulare” (implicit 1). Cardul de generare are starea nouă **„Peste limita de imagini”**, cu „Permite 2 imagini” sau „Generează 1 variantă”.
- **Politici:**
  - `network_default: deny` și `require_plan_for_network`: cercetătorul din chat folosește doar surse deja salvate; accesul online cere un plan.
  - `research_allowed_hosts` e limitat la create.roblox.com și rojo.space.
  - `allow_git_push` și `allow_github_pr` sunt false: „Combină prin GitHub” precizează că push și PR intră într-un pas aprobat.
  - `allow_upload` și `allow_publish` sunt false: publicarea rămâne un pas separat.
- **Vault:** `personal_notes_read_only` și `vault_write_subdirectory: RobloxAIBuilder`. „Partajează” a devenit **„Copiază în comună”**: se creează o copie în RobloxAIBuilder/, iar originalul nu se modifică.
- **Integrări:** Studio MCP cu `studio_id` explicit și `require_capability_discovery`; Rojo cu `require_backup_before_first_sync`; `claude_code.mode: external_development_tool_only` (afișat în Modele și costuri).
- **Diferențe care rămân deschise:**
  - Colaborarea între PC-uri nu are încă un câmp în config. `ipc_transport: windows_named_pipe` e doar local, deci e nevoie de o extindere a schemei.
  - `ollama.context_tokens: 4096`: v2 afișa 8 192. Folosește valoarea din config.
  - `start_at_logon: false`: v2 avea comutatorul pornit implicit. În implementare, pornește oprit.


---

# ACTUALIZARE v5: modul „Gazdă + Conectare prin VPN” (NOU)

> **Extensie nouă a arhitecturii.** Nu există în PRD/ARCHITECTURE și nici în `agents.example.json`. Prototipul nu creează conexiuni reale: detectarea VPN-ului, testul de conexiune, transferurile și sincronizarea sunt **simulate**.

Fișier principal: `Roblox AI Builder v5.dc.html`. Meniul Demo (dreapta-jos în zona de chat) are grupul **Colaborare prin VPN**, cu fluxul complet: găzduiește → cerere → conectare ca Mihai → transfer captură → sarcini pe PC-uri → deconectarea gazdei → reconectare și reconciliere → lucru individual.

## Decizia
Colaborarea e directă între PC-uri, prin **Radmin VPN sau Hamachi**, cu un **PC gazdă**. Nu se folosește Supabase și nici un server găzduit. VPN-ul dă doar legătura de rețea. Aplicația implementează chatul comun, transferurile, sincronizarea și coordonarea sarcinilor. Modul individual rămâne complet funcțional fără VPN.

## UI nou
- **Eticheta de conexiune** din bara de titlu e permanentă: „Găzduiești · N conectați · {sync}”, „Conectat la PC-ul lui Andrei · {sync}”, „Gazdă indisponibilă · lucrezi local” sau „Lucru individual · fără VPN”. Click deschide ecranul Găzduire sau Conectare.
- **Starea sincronizării** în antetul conversațiilor comune: Sincronizat (`#5ee08a`) · Transfer în curs (`#b6f36b`) · Modificări locale (`#f5b94a`) · Conflict (`#f87171`) · În așteptarea gazdei (`#f5b94a`).
- **Găzduiește proiectul:**
  1. Proiectul.
  2. Ce se partajează: subfoldere `RobloxAIBuilder/…` și documente, bifate explicit. Nu se partajează niciodată: notele personale, restul vault-ului, cheile, alte foldere.
  3. Rețeaua (Radmin / Hamachi / LAN, cu avertisment), adresa și portul `47820` cu Copiază, plus un **cod temporar** `RBX-XXXX-00`, valabil 10 minute, de o singură folosire, regenerabil.
  4. Cererile (Acceptă / Refuză, cu ce va primi) și membrii (Revocă accesul).

  Jos sunt separate „Închide fereastra și continuă în fundal” și „Oprește găzduirea” (cu confirmare; nu șterge proiectul).
- **Conectează-te la proiect:** pașii VPN (aplicația nu configurează VPN-ul), câmpurile nume afișat / adresă / port / cod, „Testează conexiunea” și „Solicită acces”. Stări: Se verifică legătura · Așteaptă acceptarea gazdei · Conectat · Gazdă indisponibilă · Cod expirat sau invalid · Cerere refuzată · Posibilă problemă de VPN sau firewall. Ultima vine cu diagnostic (adaptor VPN, ping gazdă, port aplicație) și instrucțiuni **doar pentru regula aplicației**, port 47820 TCP, rețea privată, fără dezactivarea firewall-ului.
- **Bannere în chat:** gazda e indisponibilă (datele sincronizate rămân citibile, mesajele noi rămân locale, operațiile care au nevoie de gazdă așteaptă); reconectat cu conflict (alegere explicită; sarcinile cu rezultat necunoscut nu se relansează); transfer de fișier cu progres, reluabil.
- **Calculatoare:** comutatorul **„Permite executarea sarcinilor aprobate pe acest PC”**, separat de conectarea la proiect, setat de fiecare utilizator pentru PC-ul lui. Gazda coordonează, iar fiecare worker își aplică propriile permisiuni. Sunt sarcini separate: nu se combină memoria GPU și nu se dublează viteza.

## Cerințe de implementare (de specificat în ADR)
- **Autentificarea participanților:** asocierea prin cod temporar de o singură folosire, acceptarea explicită de către gazdă, identitate persistentă per participant (de ex. cheie de dispozitiv), revocare.
- **Transport securizat:** criptat end-to-end peste VPN (nu te baza pe VPN pentru confidențialitate), ascultare doar pe interfața aleasă, fără expunere publică, rate limiting pentru încercările de asociere.
- **Permisiuni pe proiect:** rolurile Proprietar/Colaborator; doar subfolderele și documentele bifate; aprobarea planurilor și combinarea doar pentru proprietar; cheile și bugetele rămân per utilizator.
- **Transferuri recuperabile:** pe bucăți, cu hash, reluare după întrerupere, limită de mărime, indicator de progres.
- **Sincronizare și conflicte:** jurnal de evenimente cu secvențe (extinde `SubscribeEvents(afterSequence)`), stare locală nesincronizată după deconectare, reconciliere la reconectare, conflicte afișate explicit. Fără editare simultană „fără conflicte” și fără transferul automat al rolului de gazdă.
- **Worker:** găzduirea poate continua în fundal doar dacă utilizatorul a ales asta. PC-ul oprit sau în sleep nu găzduiește și nu execută. Fără relansarea automată a acțiunilor cu rezultat necunoscut (`recovery_required`).
- **Firewall:** regulă doar pentru aplicație și rețele private, creată cu acordul utilizatorului (UAC). Niciodată dezactivarea firewall-ului.
- **GitHub** rămâne pentru versionarea codului; nu e transportul chatului.
- **Schema de config** trebuie extinsă (de ex. `collaboration: { mode, listen_interface, port, pairing_code_ttl_seconds, shared_paths, allow_remote_task_execution }`), iar `ipc_transport` rămâne local.
