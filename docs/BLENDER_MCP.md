# Integrare opțională Blender MCP

Scop: creare/modificare asseturi 3D pentru joc, cu imagini de referință, previzualizare, optimizare și export. Blender rulează pe device-ul ales, iar MCP este accesat numai de workerul local. Sarcinile pot veni prin protocolul de colaborare, după aprobarea planului și grantul proprietarului PC-ului.

## Alegerea adaptorului — gate obligatoriu

Niciun server MCP Blender concret nu a fost ales sau verificat în această livrare. Nu presupune că există un server oficial Blender, un port standard sau anumite tooluri. Înainte de instalare verifică repository-ul furnizorului, licența, mentenanța, versiunea/commitul, transportul, dependențele, capabilitățile, accesul filesystem/rețea și compatibilitatea Blender. Documentează ADR și fixează versiunea; nu instala automat un pachet doar fiindcă numele conține „blender-mcp”.

Contract intern propus: health, list_instances, inspect_scene, apply_reviewed_operation, render_preview, validate_asset, export_asset. Capabilitățile reale se negociază și sunt mapate explicit. Funcțiile indisponibile sunt dezactivate cu explicație.

## Flux

Cerere în chat → referințe selectate → plan cu device/scene/limite → copie/backup .blend → operații aprobate → preview în chat → verificări → export într-un folder autorizat → import Studio separat → verificare vizuală și tehnică.

UI: Blender conectat/deconectat, versiune, device, scenă activă, progres, preview, variante, validări și fișiere rezultate. Nu promite mesh rigged gata de joc din orice imagine. Modelele, materiale/texturi și animațiile pot necesita intervenție manuală.

## Protecții

- Blender și add-onurile pot executa Python; MCP nu este sandbox. Codul arbitrar generat rămâne interzis implicit. MVP folosește operații/șabloane revizuite și parametri validați.
- Dacă adaptorul oferă numai execuție Python nerestricționată, integrarea rămâne blocată până la implementarea unui boundary sigur/izolare și aprobarea designului de securitate. Un prompt nu este protecție suficientă.
- Copie a scenei, un writer per instanță, timeout, limite de geometrie/render/memorie, anulare și salvare înainte de modificări.
- .blend extern este neîncrezut: auto-run scripts dezactivat și fără instalare de add-onuri din proiect. Procesul nu primește chei API; egress este restrâns.
- Exporturile și texturile rămân în root aprobat; fără resurse externe sau upload implicit. Se păstrează proveniența/licența.

## Verificări și livrabile

Verifică scară/unități, orientare, geometrie, normale, UV, materiale/texturi, buget poligoane al proiectului, pivot și nume. Pentru rigging/animație, teste dedicate dacă sunt în scope. Formatul de export și limitele Roblox se verifică în documentația oficială la implementare; nu fixa aici valori neverificate.

Acceptanță: asset simplu low-poly în scenă de test, preview real, raport, export și import verificat în Studio; aceeași operație prin workerul colegului cu țintă explicită. Teste pentru lipsă Blender, MCP incompatibil, script malițios, anulare, crash și export în afara rootului. Până atunci funcția este opțională și dezactivată.
