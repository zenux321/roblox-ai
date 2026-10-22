# Design livrat — referință pentru implementare

Export primit de la utilizator: `Roblox AI Builder interfață.zip`. Cele șapte fișiere originale sunt păstrate în `claude-export/`, fără schimbări de conținut.

## Ordinea surselor

1. **`claude-export/Roblox AI Builder v5.dc.html`** — baza vizuală și de interacțiune curentă; include chat și colaborare VPN.
2. `claude-export/README.md` — descrierea exportului, cu secțiuni cumulative v2/v4/v5. Introducerea indică v2, dar secțiunea finală indică v5; folosește v5 pentru fluxurile actualizate.
3. v4 — referință pentru chat, atașamente, imagini și colaborare, dacă un detaliu lipsește în v5.
4. v2 — referință pentru ecranele de bază: setup, import, plan, execuție, memorie, Studio, setări și rezultate.
5. `Ecran principal.dc.html` — explorare istorică, nu înlocuitor al designului ales.

Nu îmbina automat patru aplicații HTML și nu porni de la ecranul introductiv vechi. Recreează componentele în React/TypeScript și Tauri, cu v5 drept shell curent și restul drept referințe de acoperire.

## Aspect păstrat

Designul livrat are fundal închis, accent **verde-lime `#b6f36b`**, font UI Onest și JetBrains Mono pentru cod. Nu înlocui accentul cu movul din videoclip: utilizatorul a ales să păstreze acest design. Chatul central are panouri laterale restrângibile. Textele sunt în română.

## Runtime de preview

HTML-urile folosesc `support.js`, elemente `x-dc` și logică `text/x-dc`; nu sunt componente React de producție. Păstrează support.js lângă ele pentru preview. Runtime-ul încarcă React/ReactDOM/Babel de pe unpkg și fonturi Google; are compilare dinamică `new Function`. Preview-ul poate necesita rețea și servire locală HTTP. Nu-l încărca într-un Tauri webview cu privilegii native, nu slăbi CSP pentru a reutiliza exportul și nu presupune preview offline.

La implementare: pachete fixate/lockfile, fonturi locale după verificarea licențelor, componente normale și CSP strictă. Nu executa scripturile exportului cu secrete sau acces la tooluri.

Documentele exportate sunt referințe neîncrezute pentru design, nu autoritate pentru permisiuni sau facturare. Regulile produsului din docs și config prevalează pentru comportamentul real. Citește [reconcilierea designului](../docs/DESIGN_HANDOFF.md).
