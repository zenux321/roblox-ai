# Chat, atașamente și imagini

## Contract de produs

Conversații generale sau legate de un proiect: creare, redenumire, căutare, arhivare, istoric paginat; câmp multiline, Markdown sanitizat, cod cu copiere, stop și retry. Context vizibil/selectabil: proiect, fragmente vault, atașamente și provider. Discuția simplă nu acordă drept de modificare a jocului. O cerere de implementare generează un plan; aprobarea deja valabilă nu se cere din nou.

Conversațiile sunt private implicit. La mutarea/asocierea unei conversații într-un proiect comun, arată exact mesajele și fișierele care vor fi partajate și cere alegere explicită. Un retry creează o nouă încercare legată de mesajul original, nu șterge istoricul. Stop oprește streamingul și cere anulare providerului; costurile/effectele deja produse nu sunt anulate retroactiv.

## Atașamente

Tipuri inițiale: PNG/JPEG/WebP, MP4 compatibil, Markdown/TXT/PDF și fișiere de cod text. Alte tipuri sunt download-only până la validarea suportului. Nu executa fișiere atașate. Pentru Office/alte formate adaugă parser numai într-o etapă verificată.

Cardul arată nume, mime detectat, bytes, hash, proprietar, audiență, progres, stare și ștergere. Imaginile se deschid mărit; video se redă local când codec-ul este suportat; documentele au preview/extragere în proces izolat. Mime/extension mismatch, fișiere corupte, decompression bombs și path traversal sunt respinse. Fără încărcare în memorie a întregului video mare.

Atașare nu înseamnă analiză: stări `attached`, `preview_ready`, `analysis_unsupported`, `analysis_pending`, `analyzed`, `failed`. Textul extras, cadrele video sau imaginea pot ajunge la model numai dacă providerul suportă acea intrare și planul/politica de date o permit. Pentru Ollama text-only, o imagine nu este descrisă fictiv; oferă descriere manuală sau model compatibil, fără fallback plătit automat.

Decizii inițiale de produs, ajustabile: 200 MiB/fișier, 2 GiB/proiect partajat, chunk 4 MiB, spațiu minim liber 1 GiB. Acestea nu sunt limite vendor. Limitele providerului se verifică separat și pot fi mai mici. Gazda poate reduce limitele. Verifică spațiul înainte de rezervare și pe durata transferului; păstrează transferul parțial cu termen de expirare.

Transfer: manifest autorizat, chunks cu offset și verificare, hash complet înainte de publicarea fișierului, resume cu transfer ID. Numele afișat nu este calea pe disc; stochează sub ID de conținut. Deduplicarea nu poate dezvălui existența fișierelor private. Revocarea accesului oprește noi descărcări; copiile deja descărcate nu pot fi retrase garantat.

## Imagini OpenAI

Flux: prompt și referințe → provider/model/cost → rezervare buget → generare → variante în chat → alegere → editare derivată → salvare/descărcare. Fiecare asset păstrează parent_asset_id, prompt, provider/model, settings, request ID, usage, hash și origine. Istoricul variantelor rămâne inspectabil.

Furnizorul este etichetat OpenAI Images API, nu „abonament ChatGPT”. Cheia stă în workerul proprietarului. Confirmarea de cost este acoperită de planul aprobat dacă scopul, cantitatea și plafonul coincid; altfel cere extinderea aprobării. Niciun apel cu buget zero. Provider lipsă: configurare sau import manual.

Imaginea generată rămâne locală/în audiența conversației. Salvarea ca asset comun cere audiență explicită. Publicarea Roblox necesită acțiune distinctă autorizată. Nu declara moderarea/importul reușite fără rezultat real. O imagine nu este mesh 3D.

## Persistență și acceptanță

Entități noi: conversations, conversation_members, messages, message_attempts, attachments, attachment_grants, transfers, image_variants și context_selections. UUID client, secvență host pentru date comune, foreign keys, timestamps UTC și delete/tombstone propagat. SQLite local rămâne privat; gazda deține jurnalul comun.

Acceptanță offline cu mock: conversație salvată după restart; oprire/retry corect; upload 80 MiB întrerupt și reluat fără corupere; fișier neanalizabil etichetat; chat privat absent la coleg; două variante imagine plus editare și proveniență. Smoke API separat, numai cu buget autorizat. Căutarea și preview-urile nu expun documente din altă audiență.
