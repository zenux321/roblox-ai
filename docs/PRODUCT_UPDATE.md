# Cerințe consolidate — extensia 0.2

Acest document înregistrează deciziile ulterioare pachetului inițial și este obligatoriu pentru Claude Code. În caz de conflict, deciziile de aici și contractele din COLLABORATION.md și CHAT_AND_ASSETS.md înlocuiesc formulările restrictive din versiunea 0.1. Sunt specificații pentru implementare, nu funcții existente.

## Decizii confirmate

- Păstrează designul realizat de utilizator în Claude Design. Nu îl înlocui cu interfața din videoclip și nu refă onboarding-ul inutil. Exportul/designul final nu este încă în repository; înainte de implementarea fidelă a UI, cere linkul sau exportul. Nu pretinde că ai văzut designul.
- Joc nou și continuarea unui joc existent sunt ambele cerințe explicite. Proiectul existent poate avea deja un vault, poate exista doar în Studio sau poate avea surse locale Rojo.
- La revenire intră în proiectul recent; afișează starea conexiunilor și reconcilierea dintre notițe și cod. Vaultul nu este sursă infailibilă a stării jocului.
- Chat complet, conversații generale/per proiect, atașamente cu previzualizare și generare/editare thumbnails prin OpenAI Images API.
- Colaborare cu un prieten, cod comun, chat comun, note selectate, sarcini și rezultate partajate.
- Alegerea de conectivitate este **gazdă + participant prin Radmin VPN/Hamachi sau LAN**. Supabase și alt backend găzduit nu sunt necesare pentru acest mod. VPN-ul transportă date; aplicația trebuie să implementeze sincronizarea.
- PC-urile pot executa sarcini independente, fiecare cu workerul/modelul său. Nu se combină VRAM și nu se promite dublarea vitezei unui singur model.
- Blender MCP este integrare opțională pentru asseturi 3D, cu server/adaptor încă de selectat și verificat.
- Ollama rămâne local implicit. Claude API și OpenAI Images au chei/bugete separate. Creditul Claude Code Cloud nu este bugetul API al aplicației.

## Trasabilitate nouă

| ID | Cerință | Contract și dovadă de acceptanță |
|---|---|---|
| R11 | Păstrarea designului | Export/link real, hartă ecran→cerințe; nicio funcție nouă nu justifică schimbarea stilului fără cerere |
| R12 | Chat complet | CHAT_AND_ASSETS: istoric, căutare, stop/retry, general/proiect, privat/comun |
| R13 | Atașamente | Previzualizare reală, progres, limite, stare analizat/neanalizat, audiență |
| R14 | Thumbnail iterativ | Două variante mock, selecție, editare derivată, descărcare/salvare; live opt-in bugetat |
| R15 | Proiect comun | Membri, roluri, aprobare, chat, fișiere și conflicte fără suprascriere |
| R16 | Host/join VPN | COLLABORATION: două PC-uri Windows, pairing, revocare, reconnect fără servicii cloud proprii |
| R17 | Workeri pe două PC-uri | Job separat, lease, rezultat și acceptare; zero execuție fără consimțământ local |
| R18 | Blender | BLENDER_MCP: discovery verificat, copie de scenă, preview/export și validare Roblox |

## UI și ecrane de completat

Spațiul principal: navigare proiecte, chat central, context/vault, panou restrângibil echipă/sarcini; oameni și agenți vizual diferiți. Costul, providerul și ținta Studio/Blender sunt vizibile când sunt relevante. Fără loguri tehnice dominante sau stare fictivă „conectat”.

Ecrane/dialoguri: proiect existent/import, gazdă/join, cerere de acces, membri/permisiuni, calculatoare/capabilități, limite locale, conflict, fișiere comune/private, conexiuni Studio/Blender, provider și buget. Progresul agenților apare și în carduri extensibile în conversație.

Stări obligatorii: empty/loading/error, provider lipsă, format neanalizabil, buget insuficient, cod expirat, host offline, VPN inaccesibil, conflict, rezultat extern incert, coadă suspendată și transfer incomplet.

## Limitele actualei livrări

Nu există încă backend, protocol de colaborare, exe, loader de config sau adaptor Blender. Configurile sunt contracte propuse. Nu instala VPN, Blender, modele sau servere în această etapă de documentare. Implementarea se face pe etapele din ROADMAP, cu aprobarea planului și teste corespunzătoare.
