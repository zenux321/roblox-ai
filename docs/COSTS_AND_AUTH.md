# Costuri, credit Cloud și autentificare

Verificat la 22 octombrie 2026. Reguli comerciale și accesul efectiv se reverifică înaintea implementării și lansării. Soldurile nu au fost citite din conturile live.

## Creditul de 250 USD

Captura din conversația referită arată „Cloud session credits”, 250 USD din 250 USD rămași și expirare „9:59 AM GMT+2, November 5”. Este credit monetar promoțional pentru sesiuni Claude Code Cloud, nu 250 de tokeni și nu credit GitHub/API. Documentația promoției indică revendicare până la 7 octombrie și expirare 4 noiembrie, 23:59 PT, adică **5 noiembrie 2026, 09:59 în București**, concordant cu captura. La 22 octombrie, termenul de revendicare a trecut; soldul din captură nu confirmă soldul actual.

Promoția nu finanțează Claude API, OpenAI, workerul local sau Remote Control. După epuizare/expirare se aplică limitele planului; usage credits activate pot produce costuri. Evită opțiunile care cer PAYG și păstrează consumul suplimentar dezactivat. GitHub servește drept sursă pentru cod și ramuri, nu transformă creditul în cheie API. [Sursa promoției](https://support.claude.com/en/articles/17152539-cloud-sessions-bonus-credit-promotion).

## Moduri distincte

| Mod | Autentificare | Buget / utilizare în proiect |
|---|---|---|
| Ollama local | API local, fără cheie implicit | Bază gratuită ca serviciu; hardware, energie și downloaduri rămân costuri reale |
| Claude Code Cloud, dezvoltare | Cont propriu în produsul oficial | Credit promoțional eligibil, apoi limitele planului |
| Claude Code local, dezvoltare | Login oficial sau cheie API proprie | Limitele abonamentului sau facturare API, în funcție de autentificare |
| Claude API în aplicație | Cheie Claude Console/furnizor suportat | Separat de abonament; dezactivat implicit |
| OpenAI Images în aplicație | Cheie API OpenAI proprie | Facturare API separată; dezactivat implicit |
| Import manual de imagini | Utilizatorul aduce fișierul | Nu presupune cost API al aplicației |
| GitHub | Git Credential Manager / autentificare oficială | Repo privat și servicii în limitele contului; minutele CI și stocarea se verifică |

Nu se bugetează un număr garantat de sarcini din 250 USD. Modelul, contextul și reîncercările modifică consumul. Lucrează pe etape mici cu teste și checkpoint-uri, verificând soldul în interfața oficială. Nu construi un scraper al paginii de billing.

## Claude: delimitarea permisă

Aplicația folosește chei API pentru propriul adaptor Claude. Nu oferă un login Claude.ai propriu, nu colectează tokenuri și nu rutează cererile prietenilor prin abonamentul autorului. Claude Code oficial rămâne instrument separat, nemodificat, cu autentificarea utilizatorului în fluxul Anthropic. Condițiile actuale permit anumite utilizări ale binarului oficial în produse, cu obligații comerciale; de aceea launcherul încorporat este o extensie separată, nu un proxy ascuns inclus în MVP. [Reguli oficiale](https://code.claude.com/docs/en/legal-and-compliance).

În sesiuni locale, prezența `ANTHROPIC_API_KEY` poate selecta facturarea API în locul abonamentului. Verifică metoda afișată de Claude Code; pentru modul abonament nu injecta cheia în proces. Nu modifica automat secretele globale ale utilizatorului. [Ghid Pro/Max](https://support.claude.com/en/articles/11145838-use-claude-code-with-your-pro-or-max-plan).

## OpenAI imagini

Folosește Image API pentru generare/editare simplă; Responses cu image tool doar dacă este necesar un flux conversațional. Modelul se selectează explicit din oferta disponibilă contului, la implementare; exemplul nu fixează un model posibil indisponibil. Numărul, dimensiunea și calitatea sunt limitate în plan. Unele conturi pot necesita verificarea organizației. Salvează promptul, modelul, usage, request ID și proveniența assetului, fără cheia API. O imagine nu este automat mesh, animație sau asset Roblox publicat. [Image generation](https://developers.openai.com/api/docs/guides/image-generation).

Abonamentele și prețurile API sunt sisteme distincte; nu utiliza un abonament ChatGPT drept portofel implicit pentru aplicație. [Prețuri și separarea API](https://learn.chatgpt.com/docs/pricing), [tarife API](https://developers.openai.com/api/docs/pricing).

## Politica de cost a aplicației — decizie de proiectare

1. `max_run_usd = 0` și `max_month_usd = 0` implicit. Creditul Cloud de dezvoltare nu se adună la acestea.
2. Activarea unui provider plătit cere cheie validă, model explicit, estimare și buget aprobat. Zero blochează orice cerere plătită chiar dacă furnizorul oferă credite.
3. Înainte de apel se rezervă tranzacțional plafonul estimat conservator pentru toate cererile concurente. Se plafonează și tokeni, imagini, retry-uri și durata.
4. Fără tarif verificat sau fără o limită superioară utilizabilă, nu se lansează apelul. După apel se reconciliază usage; la rezultat incert rezerva rămâne blocată până la clarificare.
5. Limita locală nu este o garanție de facturare a furnizorului: costurile în curs, tarifele și rotunjirile pot varia. UI separă estimat, rezervat, confirmat și necunoscut. Bugetele providerului sunt protecții suplimentare, nu presupuse hard caps.
6. Fără auto-recharge și fără trecere silențioasă la un model/furnizor plătit. La limită: pauză, raport sau variantă locală dacă aceasta este deja aprobată.
7. Dacă Ollama lipsește ori modelul nu încape în memorie, oferă model mai mic/download aprobat sau workflow manual; nu promite zero cost pentru orice cerere.
