# Colaborare gazdă/participant prin VPN

## Decizie

Același exe poate funcționa solo, gazdă sau participant. Radmin VPN/Hamachi ori LAN oferă conectivitate IP; nu stochează chatul și nu sincronizează proiectul pentru noi. Fără Supabase obligatoriu, port forwarding automat sau backend propriu găzduit. VPN-ul rămâne software separat, configurat prin fluxul său oficial. Nu promite absența infrastructurii furnizorului VPN.

Închiderea UI poate lăsa workerul/gazda pornită, dacă utilizatorul a activat background. Sleep, logout și oprirea gazdei întrerup colaborarea live. Nu există failover automat de gazdă în prima versiune. Se pot folosi datele deja sincronizate offline, apoi se reconciliază la revenirea aceleiași gazde.

## Topologie și frontiere

UI → IPC local → worker propriu → adaptor colaborare TLS pe IP VPN selectat → coordinator gazdă → jurnal comun/fișiere partajate. Workerii accesează local Ollama, Studio și Blender. IPC named pipe, Ollama și MCP nu devin servicii de rețea. Adaptorul de colaborare este un modul separat de endpointurile locale.

Listener dezactivat implicit; activarea „Găzduiește” alege explicit IP/adaptor VPN și port. Nu bind la toate interfețele/0.0.0.0. Firewall limitat la aplicație/interfață/peer, fără dezactivare globală. La schimbarea IP-ului oprește listenerul și cere reconfirmare. TLS și permisiunile aplicației sunt necesare chiar dacă VPN-ul este criptat.

Protocol propus: HTTPS pentru pairing și transferuri, WebSocket TLS pentru evenimente. Versiune negociată, dimensiuni maxime, timeout, rate limits și heartbeat configurabile. Endpointurile sunt contract intern de implementat, nu funcții ale Radmin.

## Asociere și autentificare

Gazda generează certificat/identitate persistată protejat și cod aleator temporar, o singură folosire, expirare 5 minute și limită de încercări. Participantul verifică fingerprintul gazdei pe un canal separat; codul nu înlocuiește verificarea identității. Gazda acceptă explicit cererea și rolul. Emite credențiale individuale revocabile, stocate în Credential Manager, nu în repo/log/vault. Refolosește biblioteci criptografice mature; nu inventa protocol criptografic.

Roluri inițiale: owner administrează membri și aprobă integrarea/planurile comune; collaborator propune și lucrează în scope. Nimeni nu poate acorda drepturi asupra PC-ului sau bugetului altui membru. Fiecare resursă are project_id și audience; verificarea ACL se face pe server pentru fiecare cerere, inclusiv fișiere și reconnect. Tokenul revocat închide sesiunile și blochează reconectarea.

## Date comune și sincronizare

Gazda este autoritatea pentru jurnalul comun și membership. Fiecare operație are operation_id UUID, project_id, author/device, base_revision și payload validat. Gazda deduplică, aplică în tranzacție și atribuie sequence. Clientul păstrează outbox și cursor last_ack; reconnect cere evenimente lipsă sau snapshot versionat. Nicio scriere directă într-un SQLite comun prin share de rețea.

Chat append-only cu editări/tombstones versionate; planurile sunt imuabile per versiune/hash; note/fișiere folosesc compare-and-swap pe hash de bază. Dacă baza diverge, păstrează ambele versiuni și deschide conflict. Nu aplica last-write-wins pentru cod, vault sau aprobări. Private notes/messages nu intră în jurnalul gazdei fără partajare explicită.

Operațiile offline sunt propuneri locale, nu aprobări comune definitive. La reconnect, reverifică membership, versiune plan și permisiuni înainte de acceptarea outboxului. Revocarea nu poate șterge garantat copiile deja descărcate. Backupul comun se face pe gazdă, separat de copiile locale; restore conservă identitatea/sequence sau forțează reîmperechere și reconciliere explicită.

## Sarcini și două PC-uri

Fiecare device declară capabilități, versiuni, modele locale și integrarea Studio/Blender disponibilă. Aceste declarații sunt date neîncrezute, nu autorizare. Workerul acceptă numai tipuri de task cunoscute și propriul scope local.

Consent separat „Permite sarcini aprobate pe acest PC”, cu folder roots, limite resurse, program, timeout și provider/buget propriu. Concurență implicită 1 per device; două sarcini independente pot rula pe două PC-uri. Plannerul nu poate impune consent sau o cheie API.

Task assignment: project_id, task_id, plan_hash, input hashes, device_id, allowed tools, base revision, budget owner, expiry. Coordinatorul acordă lease cu fencing token monoton; heartbeat îl reînnoiește. Reatribuirea se permite numai după oprirea/reconcilierea execuției anterioare, mai ales pentru efecte externe. Lease expirat oprește noi efecte; un rezultat întârziat nu suprascrie versiunea curentă.

Outputul este un patch/artefact cu hash și test report, inițial propus. Gazda verifică scope, snapshot, token, tip/size și rezultate înainte de integrare. Nu execută cod host arbitrar primit de la coleg. Timeout după upload/publicare intră în recovery_required, fără retry orb.

Planul comun este aprobat de owner; taskurile pe alt PC necesită și grantul local deja valid pentru acel scop. Orice folosire API pe contul colegului cere buget aprobat de el. Politica efectivă este intersecția planului comun, rolului și politicii device-ului. Oprirea locală are prioritate.

## Git și Studio

GitHub rămâne versionare opțională pentru cod, nu transport obligatoriu de chat. Checkout/branch per participant, patches și diff înainte de integrare; nu partaja directorul .git live. Modificările conflictuale se păstrează. Indicatorul „lucrează la fișier” este informativ, nu protecție suficientă împotriva conflictelor.

Operațiile Studio/Blender au resource owner, device_id și instance_id, cu un writer activ per țintă. Rojo filesystem ownership rămâne în vigoare. Acest protocol nu înlocuiește Roblox Team Create și nu promite editarea simultană a aceluiași place. Un PC poate construi cod, altul poate cerceta sau testa o copie izolată.

## Acceptanță

Două PC-uri Windows conectate pe VPN: pairing, host accept, chat, transfer cu resume, roluri, task fake pe fiecare, rezultat verificat, disconnect/reconnect și conflict. Host down: date cache disponibile, fără stare falsă online. Revoacă colegul în timp ce descarcă și verifică blocarea accesului ulterior. Nu lansa shell/MCP/Ollama remote direct.
