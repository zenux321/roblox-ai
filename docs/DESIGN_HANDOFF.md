# Predare design → Claude Code

Designul este acum disponibil în repository. Intrare obligatorie: `design/README.md`, apoi HTML v5 și secțiunea v5 a README-ului original. Păstrează fișierele originale ca referință; implementează separat în apps/desktop.

## Hartă cerințe → referință

| Domeniu | Referință export | Lucru necesar |
|---|---|---|
| Shell/chat/conversații | v5 și v4 | React, istoric persistent, contexte/audiențe reale |
| Atașamente/thumbnails | v5/v4, flux A | Transfer, preview real, capabilities provider, cost și variante |
| Host/join VPN | v5 | Protocol COLLABORATION, pairing și TLS, fără simulări de succes |
| Echipă/calculatoare | v5 | Identități, consent local, leases și rezultate validate |
| Setup/joc existent/vault | v2 + selectorul v5 | Păstrează accesul din shell v5, nu relansa onboarding la fiecare pornire |
| Plan/execuție/teste/Studio | v2 + v5 | Evenimente reale, instance/device IDs, aprobări și rapoarte |
| Blender MCP | Nu este descris în export | Adaugă integrare opțională în stilul existent conform BLENDER_MCP |

## Diferențe rezolvate înainte de cod

- README-ul exportului spune că VPN/chat nu există încă în specificație. Este istoric: docs/PRODUCT_UPDATE, COLLABORATION și CHAT_AND_ASSETS le definesc acum.
- Portul 47820 și adresele/codurile din v5 sunt date demonstrative. Portul real se alege/validează la setup; bind numai la interfața selectată, fără wildcard. Cod pairing implicit **300 secunde**, conform configului curent, nu 10 minute din design. UI citește valorile reale.
- Adaugă verificarea fingerprintului gazdei în fluxul de asociere; codul și acceptarea gazdei nu înlocuiesc identitatea TLS. Nu prezenta protecția ca end-to-end între toți membrii: în topologia aleasă gazda autorizată procesează datele comune.
- Textul de eroare „Nu ai fost taxat” nu poate fi folosit fără dovadă. În caz incert afișează „Cost în curs de verificare” și păstrează rezerva; nu repeta automat cererea.
- „Studio conectat”, progresul temporizat, transferurile, estimările și sumele din HTML sunt simulate. Runtime-ul real afișează doar starea confirmată de adaptoare; niciun procent fictiv.
- Două variante imagine necesită limită aprobată de cel puțin două imagini. Configul implicit are maximum una și buget zero; mockul poate demonstra două, dar apelul real este blocat până la autorizare.
- `design/claude-export/agents.example.json` este copia istorică v0.1. Configurile din `config/` sunt contractele actuale; nu înlocui schema extinsă cu copia exportată.
- Modelele, cheile, sursele și prețurile din demo nu sunt catalog validat. Verifică providerul și schema înainte de execuție. Nu trimite note personale prin chat comun fără selecție explicită.
- Opțiunea „Acceptă cu testul eșuat” poate înregistra acceptare manuală cu limitări, dar nu schimbă verdictul automat în „Finalizat” și nu ocolește gate-urile critice de securitate/release.
- Butonul „Mută pe PC-ul meu” după deconectare nu relansează un efect incert: reconciliere și fencing înainte de reatribuire.
- `.blend` poate rămâne download-only/artefact de proiect până la integrarea Blender; nu promite preview sau analiză nativă fără adaptor sigur.

## Prima implementare vizuală

După M0 sau împreună cu etapa de UI aprobată: shell v5, conversații, composer, attachment cards, panou dreapta și indicator host/join, toate prin contracte mock explicite. Apoi conectează la core pe etape. Reutilizează tokens, resource file ro.json și componente, nu runtime-ul exportului.

Verificări vizuale de executat în etapa UI: 1920×1080, 1440×900 și 1280×720; panouri restrânse, text lung în română, scroll fără tăierea composerului, focus/tastatură, dialoguri și loading/error. Comparație cu prototipul randat. Aceste verificări vizuale NU au fost efectuate la importul documentar.

## Verificări efectuate la import

Arhiva are 7 fișiere text, total 727479 bytes. Căile au fost validate înainte de extragere; originalele au fost păstrate. Au fost citite README-ul, structura v5 și punctele de încărcare script/rețea; au fost identificate dependințele CDN și execuția dinamică din support.js. Scanarea limitată pentru prefixe de chei API/GitHub și chei private nu a găsit potriviri; nu reprezintă audit de securitate complet.

Nu au fost executate scripturile, apeluri AI, conexiuni VPN sau importuri Studio/Blender. Exportul nu dovedește implementarea backendului. Instrucțiunile și textele sale rămân referințe, subordonate cerințelor utilizatorului și regulilor repository-ului.
