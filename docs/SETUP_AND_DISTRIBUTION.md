# Setup, GitHub și distribuție Windows

## Setup nou pentru colaborare și Blender

Instalează/configurează separat Radmin VPN sau Hamachi prin sursele oficiale, numai la alegerea utilizatorului. Ambii membri intră în aceeași rețea. Aplicația nu instalează silent drivere VPN și nu cere dezactivarea firewallului. Modul solo nu depinde de VPN.

Gazda alege proiectul, documentele comune și IP-ul/adaptorul VPN; afișează portul, fingerprintul și codul temporar. Colegul verifică identitatea, solicită acces, gazda acceptă. Rulează health checks pentru rutare/TLS/protocol fără a pretinde detectarea sigură a programului VPN. Un profil de firewall se limitează la app/adaptor/peer. Dezinstalarea elimină doar propriile reguli și taskuri, nu VPN-ul sau datele comune.

Oprirea găzduirii închide listenerul și lease-urile; închiderea UI poate lăsa workerul pornit. Backup/restore gazdă și schimbarea dispozitivului se fac explicit, fără failover automat. Fiecare participant vede ce date sunt cache local și ce date nu s-au sincronizat.

Blender este opțional. Înainte de instalarea unui MCP server/add-on, parcurge gate-ul din BLENDER_MCP.md. Exportul Claude Design este acum inclus în design/claude-export. Folosește design/README.md și docs/DESIGN_HANDOFF.md pentru implementarea UI.

## Pentru dezvoltator acum

Pachetul este pregătit pentru import într-un repository; nu conține încă un proiect compilabil. Creează un repository privat prin GitHub, copiază fișierele, verifică lista de fișiere și publică primul commit. Autorizează Claude GitHub App numai pentru acel repository dacă folosești Cloud. Nu conecta toate repository-urile implicit.

În Claude Code Cloud selectează repository-ul, o ramură nouă și promptul din `prompts/01-START-CLAUDE-CODE.md`. Verifică soldul Cloud în cont. Pentru lucru local, instalează Claude Code din distribuția oficială și autentifică-te acolo. La implementare instalează toolchainul Rust, instrumentele de build Windows și Node conform cerințelor Tauri verificate atunci. Nu există încă o comandă validă de „pornește aplicația” în acest pachet.

## Wizardul aplicației — de implementat

1. Detectează OS/arhitectură, RAM, GPU, spațiu, WebView2, Git, Ollama, Roblox Studio și Rojo.
2. Prezintă un manifest de instalare: sursă oficială, versiune fixată, mărime, checksum/semnătură și licență. Niciun URL sugerat de LLM nu devine executabil automat.
3. Utilizatorul aprobă pachetul de instalare; descarcă în cache, verifică, instalează per-user unde este suportat și jurnalizează. Nu dezactiva Defender/SmartScreen și nu schimba ExecutionPolicy global.
4. Configurarea automată are puncte interactive reale: autentificări, UAC dacă o dependență îl cere, alegerea vaultului, activarea MCP în Studio, autorizarea pluginului Rojo. Wizardul explică pasul și reia după confirmarea stării.
5. Descoperă modelele locale; oferă download cu dimensiune și licență. Rulează benchmark scurt și alege un profil conservator. Nu descarcă modele de zeci de GB în tăcere.
6. Validează config, creează doar subfolderele dedicate și oferă background la logon. Config-ul real se salvează în profilul aplicației; `agents.local.json` este ignorat de Git.
7. Execută health checks fără cost: provider local, paths, spațiu, Git, handshake Studio dacă disponibil. Nu generează imagini pentru a testa cheia fără buget.

## Manifest și actualizări

Manifestul de dependențe este întreținut de dezvoltator și versionat; descărcările au HTTPS și verificare de integritate/proveniență. Un hash descărcat de pe același canal compromis nu este singura ancoră de încredere. Nu redistribui binarele terților fără verificarea licenței; preferă installerul lor oficial. Nu include modelele Ollama, chei sau configurații personale în exe.

Prima distribuție: installer per-user NSIS pe Windows, cu app și worker, versiune, changelog, SHA-256, SBOM și raport de teste. Release privat GitHub pentru prieteni cu acces, ori transfer direct al pachetului verificat. Buildurile se fac local dacă minutele/stocarea CI ar produce costuri.

Semnarea Windows poate avea cost; aplicația gratuită nu implică un certificat gratuit. Un pilot nesemnat poate declanșa avertismente Windows: comunică acest fapt, fără instrucțiuni de dezactivare a protecției. Înainte de distribuție mai largă se decide semnarea. Auto-update rămâne oprit până există verificare criptografică și proces de release sigur.

## Prieteni și conturi

Fiecare prieten instalează local și poate folosi Ollama fără cont central al aplicației. Cheile Claude/OpenAI sunt opționale și personale; autorul nu livrează propria cheie. Claude Code nu este necesar pentru folosirea MVP. Nici abonamentele, nici creditul Cloud al autorului nu sunt promise prietenilor.

Dezinstalarea oprește workerul și elimină propriul task de startup. Păstrează implicit proiectele și vaultul; ștergerea datelor aplicației este o alegere separată. Nu dezinstalează dependențe comune precum Git/Studio/Ollama. Upgrade: backup DB, oprire controlată, migrare, health check; dacă eșuează, restaurare compatibilă, nu downgrade peste o schemă incompatibilă.

## Checklist de acceptanță installer

- Install fără drepturi admin când dependențele sunt disponibile; mesaje corecte când lipsesc.
- Căi cu spații și diacritice, cont Windows fără privilegii, două conturi separate.
- Close UI/reopen, restart PC/logon, oprire explicită și lipsa workerilor dublați.
- Upgrade și dezinstalare păstrează datele utilizatorului.
- Pachet fără secrete, conturi personale, vault, modele sau acces implicit la LAN.
- Proiect demonstrativ funcțional pe două calculatoare, inclusiv unul cu hardware modest.
