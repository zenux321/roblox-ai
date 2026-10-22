# Specificație de produs

## 1. Scop și utilizatori

Utilizatorul descrie un joc sau o modificare în română. Aplicația cercetează surse, citește memoria proiectului, propune un plan și, după aprobare, execută sarcinile, testează și livrează modificări verificabile. Public: autorul și un grup mic de prieteni, fiecare pe propriul Windows. Fără taxă proprie de utilizare; serviciile externe pot avea costuri separate.

Platformă inițială: Windows 11 x64. Alte ediții/ARM nu sunt promise înainte de testare. Hardware-ul utilizatorului nu este cunoscut; onboarding-ul detectează RAM, spațiu și GPU și rulează un benchmark mic. Un calculator modest primește un model local mic și context limitat, cu avertizare despre calitate/viteză, nu promisiunea unui joc complet autonom.

## 2. Cerințe confirmate și trasabilitate

| ID | Cerință | Soluție / acceptanță |
|---|---|---|
| R1 | Google, YouTube, Reddit, Roblox și alte surse | Adaptoare de cercetare, proveniență, linkuri și import manual când accesul automat nu este disponibil |
| R2 | „Ambele variante” | Presupunere: joc nou/proiect existent și Cloud/local; se validează înaintea extinderilor specifice |
| R3 | Instalare/configurare automată | Wizard detectează dependențe, oferă plan de instalare, verifică distribuții; loginul, UAC și activarea Studio rămân interactive unde este necesar |
| R4 | Autor și câțiva prieteni, gratuit | Installer per-user, funcții locale fără cont central, chei individuale |
| R5 | Aprob planul, apoi execută tot | Aprobare versionată cu limite, execuție fără întrebări repetitive în domeniul aprobat |
| R6 | Continuă cu aplicația închisă, PC pornit | Worker separat de UI, coadă persistentă, control în tray |
| R7 | Abonament/credite existente când se poate | Ollama implicit; Cloud pentru dezvoltare; API plătit numai opt-in |
| R8 | Obsidian vault | Markdown local cu surse, decizii, planuri și rapoarte; index local |
| R9 | OpenAI imagini | Adaptor opțional cu limită de imagini/cost; import manual disponibil |
| R10 | Studio MCP/Rojo, GitHub, exe | Adaptoare locale, branch/diff, installer Windows verificat |

## 3. Flux principal

1. Wizard: alege proiectul și vaultul; detectează dependențele; afișează spațiul necesar, licențele și ce se va instala.
2. Creează proiect din șablon minimal sau importă un proiect existent într-o copie/ramură de lucru. Creează backup înaintea primei sincronizări.
3. Utilizatorul descrie cererea. Cercetarea locală pornește imediat; accesul extern respectă politica proiectului și destinațiile permise.
4. Plannerul produce obiective, sarcini și dependențe, fișiere/zone Studio vizate, teste, furnizori, date trimise extern, timp estimativ și buget maxim.
5. UI afișează planul și diferențele față de versiunea anterioară. Utilizatorul aprobă versiunea concretă.
6. Workerul execută sarcinile aprobate, cu jurnal și checkpoint. Reparațiile pot continua în aceeași aprobare numai în limitele fișierelor, instrumentelor și numărului de încercări aprobate.
7. Raportul include diff, rezultate reale, capturi dacă sunt disponibile, cost măsurat/estimat și limitări. „Finalizat” cere testele obligatorii trecute; altfel „Necesită verificare”.
8. Commit/push/PR și publicarea în Roblox sunt acțiuni distincte incluse explicit în plan sau aprobate ulterior.

## 4. Ecrane

- Proiecte și setup: stare dependențe, creare/import, vault și backup.
- Chat și cercetare: surse, fragmente, marcaje de încredere și limite de acces.
- Plan: sarcini, schimbări, teste, cost, permisiuni, Aprobare/Revizuire.
- Execuție: job curent, progres real, motiv de așteptare, Pauză/Anulare/Oprește tot.
- Modificări și teste: diff, rezultate Studio, rapoarte și acceptarea livrării.
- Setări: modele, chei, bugete, background, export/ștergere date; fără chei vizibile implicit.

## 5. MVP și extinderi

MVP: proiect local, Ollama, plan aprobat, modificări în copie de lucru, vault, worker persistent, Rojo și un flux Studio testabil. Demonstrație: un obby mic cu spawn, 5 platforme, checkpoint și finish; serverul validează progresul, fără monetizare sau DataStore live.

v0.2: cercetare web cu adaptoare, imagini OpenAI opționale, integrare GitHub cu permisiuni restrânse și installer pentru prieteni. Claude API este opțional, nu precondiție pentru MVP.

În afara primei versiuni: hosting central, conturi partajate, server de agenți pe internet, jocuri comerciale autonome, generare garantată de mesh-uri rigged din imagini, publicare automată implicită, execuție host arbitrară, cumpărarea de asseturi și abonamente.

## 6. Cercetare și memorie

Google este o sursă de descoperire, nu promisiunea unui API gratuit nelimitat. Folosește interfețe permise, linkuri introduse de utilizator sau import manual. YouTube: metadate și transcrieri accesibile legal/autorizat; nu se presupune că orice video poate fi transcris automat. Reddit: acces oficial/autorizat, cache și rate limits. Roblox Creator Hub este prioritar pentru detalii tehnice. Alte domenii intră în politica de egress aprobată.

Fiecare notă derivată păstrează URL, data accesului, titlul, scurt rezumat, drepturi cunoscute și nivelul de încredere. Nu se copiază masiv conținut protejat. Memoria personală existentă este read-only implicit; aplicația scrie într-un subfolder dedicat și nu instalează pluginuri Obsidian.

## 7. Criterii de acceptanță produs

- O instalare fără chei API poate crea și testa un proiect demonstrativ cu un model local disponibil.
- Nicio modificare a proiectului țintă înainte de aprobare; niciun apel plătit cu buget 0.
- Închiderea UI nu oprește jobul; reconectarea arată aceeași stare și același jurnal.
- Oprirea forțată a workerului nu dublează un upload/commit la reluare.
- Lipsa Studio suspendă doar sarcinile dependente; lipsa modelului produce o explicație, fără fallback plătit.
- Un diff/raport este inspectabil înainte de publicare. Testele neexecutate sunt etichetate explicit.
- Instalarea și dezinstalarea pe un Windows curat nu șterg vaultul/proiectele și nu cer cheia autorului.

Ținte propuse de calitate: UI răspunde în sub o secundă la acțiuni locale obișnuite; progresul se actualizează în câteva secunde; jurnalul rezistă unui restart. Timpii de inferență se măsoară pe hardware-ul real, fără SLA inventat.
