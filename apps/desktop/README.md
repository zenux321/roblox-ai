# apps/desktop — interfața Roblox AI Builder

React 19 + TypeScript în shell Tauri 2.

- **În aplicația desktop** (`npm run tauri dev`), interfața folosește `TauriWorkspaceGateway`. Conversațiile, planurile, aprobările și rulările sunt salvate de serviciul Rust (`crates/app`) în SQLite, în `%LOCALAPPDATA%/RobloxAIBuilder`.
- **În browser** (`npm run dev`), folosește gateway-ul demonstrativ din `src/mock/`, cu meniul „Demo: stări”.
- În ambele moduri nu există model real, Studio, VPN sau apeluri plătite.

## Comenzi

```bash
npm ci                 # dependențe exacte din package-lock.json
npm run dev            # preview în browser: http://localhost:5173
npm run check          # prettier + eslint + tsc + vitest + build
npm run test:visual    # Playwright: capturi în test-results/visual/
npm run tauri dev      # aplicația desktop (Windows: WebView2 + Rust 1.97)
npm run tauri build    # installer NSIS per-user (neverificat încă)
```

Pe Windows, la primul `npm run test:visual`, rulează `npx playwright install chromium` (descarcă browserul pentru Playwright 1.56.1).

## Structură

- `src/domain/types.ts` — contractele UI ↔ nucleu (`WorkspaceGateway`, `PlanGateway`), verificate de `src/domain/contract.test.ts` față de `tests/fixtures/contract/dto-samples.json` (generat din Rust).
- `src/platform/tauriGateway.ts` — comenzile Tauri și evenimentele `rab://run-event` / `rab://reply`.
- `src/components/PlanScreen.tsx` — Plan, Execuție, Rezultate și fereastra de recuperare.
- `src/mock/` — gateway-ul demonstrativ și datele din prototipul v5, reconciliate.
- `src/components/` — componentele recreate după `design/claude-export/Roblox AI Builder v5.dc.html`.
- `src/i18n/ro.json` — textele UI în română.
- `src/theme/tokens.css` — tokenurile din `docs/DESIGN_HANDOFF.md`.
- `src-tauri/` — shellul Rust: comenzi subțiri peste `crates/app`, CSP strictă, capabilities minime (fereastră + `listen`).
