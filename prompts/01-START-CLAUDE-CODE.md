# Primul prompt pentru Claude Code Cloud sau local

Copiază textul de mai jos într-o sesiune deschisă în repository:

---

Continuă proiectul Roblox AI Builder pentru Windows din acest repository. Comunică în română. Citește CLAUDE.md, README.md și documentele din docs/, inclusiv COSTS_AND_AUTH.md și SECURITY_TEST_PLAN.md. Acestea sunt specificații țintă, nu funcții deja implementate.

Lucrăm acum numai la M0 din ROADMAP. Inspectează fișierele, starea Git și mediul disponibil. Verifică documentația oficială necesară alegerii versiunilor. Propune un plan concret pentru M0 cu fișierele de creat, dependențele/descărcările, testele, costul maxim și ce verificări vor necesita ulterior Windows local. Așteaptă aprobarea acestui plan înainte de modificări. Nu considera acest prompt aprobarea etapelor M1–M6.

După ce aprob planul M0, execută toate sarcinile lui fără să ceri confirmări repetitive. Creează scheletul Tauri 2 + React/TypeScript și workspace Rust pentru core/worker, păstrând minimalismul. Implementează mai întâi schemele config/plan și validarea strictă, policy engine testabil, provider fake și o execuție demonstrativă offline într-un fixture. Core trebuie să verifice plan hash, aprobare/revocare, instrumente/ținte permise, furnizori dezactivați și buget API zero. Configurațiile din config/ sunt exemple pentru runtime-ul nostru, nu config Claude Code.

Include teste reale pentru respingerea acțiunilor neaprobate, schimbarea planului după aprobare, căi în afara proiectului, provider plătit cu buget zero și reluarea stărilor. Adaugă verificări de formatare/lint/tipuri/build și CI fără secrete dacă sunt incluse în plan. Nu inventa rezultate ale testelor indisponibile în mediul Cloud.

Nu apela API-uri plătite, nu consuma credite de imagini, nu instala modele mari, nu citi vaultul meu personal, nu modifica Studio și nu publica nimic. Nu extrage cookie-uri sau tokenuri Claude/ChatGPT și nu folosi abonamentele ca proxy API. Nu încorpora Claude Code ca backend. Ollama va fi providerul local implicit într-o etapă viitoare, iar Claude API/OpenAI rămân dezactivate. Nu face push/PR/merge decât dacă operația și destinația au fost incluse în planul aprobat.

Dacă o limită a mediului împiedică un pas, continuă celelalte sarcini independente aprobate și documentează exact ce trebuie testat local. La final livrează schimbările, comenzile și rezultatele verificărilor efectiv rulate, limitările și propunerea etapei următoare. Nu marca aplicația gata sau sigură doar pentru că scheletul compilează.

---

După primirea planului, un răspuns potrivit este: „Aprob planul M0 versiunea [versiunea prezentată], cu buget API 0 USD și limitele descrise. Execută toate sarcinile din el.” Înlocuiește versiunea cu cea reală; nu aproba în avans un plan nevăzut.
