/**
 * Randează prototipul v5 doar pentru comparație vizuală, izolat în browserul
 * de test (niciodată în Tauri). Bibliotecile CDN se servesc din fișiere locale
 * indicate de PROTOTYPE_LIBS (pachetele npm react@18.3.1, react-dom@18.3.1,
 * @babel/standalone@7.29.0, identice cu cele de pe unpkg); fără ele testul e sărit.
 */
import { readFileSync } from 'node:fs';
import { join, resolve } from 'node:path';

import { expect, test } from '@playwright/test';

const LIBS = process.env.PROTOTYPE_LIBS;
const DESIGN = resolve(import.meta.dirname, '../../../design/claude-export');
const FONTSOURCE = resolve(import.meta.dirname, '../node_modules/@fontsource');
const ORIGIN = 'http://prototype.local';

function fontCss(): string {
  const files = [
    ['onest', ['400', '500', '600', '700']],
    ['jetbrains-mono', ['400', '500']],
  ] as const;
  return files
    .flatMap(([pkg, weights]) =>
      weights.flatMap((w) =>
        ['latin', 'latin-ext'].map((subset) =>
          readFileSync(join(FONTSOURCE, pkg, `${subset}-${w}.css`), 'utf8').replaceAll(
            'url(./files/',
            `url(${ORIGIN}/fontsource/${pkg}/files/`,
          ),
        ),
      ),
    )
    .join('\n');
}

test.skip(!LIBS, 'PROTOTYPE_LIBS nu este setat');

test('prototipul v5 (referință) 1440×900', async ({ page }) => {
  const libs = LIBS ?? '';
  const scripts: Record<string, string> = {
    'https://unpkg.com/react@18.3.1/umd/react.production.min.js': join(
      libs,
      'react-18.3.1/package/umd/react.production.min.js',
    ),
    'https://unpkg.com/react-dom@18.3.1/umd/react-dom.production.min.js': join(
      libs,
      'react-dom-18.3.1/package/umd/react-dom.production.min.js',
    ),
    'https://unpkg.com/@babel/standalone@7.29.0/babel.min.js': join(
      libs,
      'babel-standalone-7.29.0/package/babel.min.js',
    ),
  };
  await page.route('**/*', async (route) => {
    const url = route.request().url();
    const cors = { 'access-control-allow-origin': '*' };
    if (scripts[url]) {
      return route.fulfill({
        body: readFileSync(scripts[url]),
        contentType: 'text/javascript',
        headers: cors,
      });
    }
    if (url.startsWith('https://fonts.googleapis.com/css2')) {
      return route.fulfill({ body: fontCss(), contentType: 'text/css', headers: cors });
    }
    if (url.startsWith(`${ORIGIN}/fontsource/`)) {
      const rel = decodeURIComponent(url.slice(`${ORIGIN}/fontsource/`.length));
      return route.fulfill({
        body: readFileSync(join(FONTSOURCE, rel)),
        contentType: 'font/woff2',
        headers: cors,
      });
    }
    if (url.startsWith(`${ORIGIN}/`)) {
      const rel = decodeURIComponent(url.slice(ORIGIN.length + 1));
      const type = rel.endsWith('.js') ? 'text/javascript' : 'text/html';
      return route.fulfill({ body: readFileSync(join(DESIGN, rel)), contentType: type });
    }
    return route.abort();
  });
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto(`${ORIGIN}/${encodeURIComponent('Roblox AI Builder v5.dc.html')}`);
  await expect(page.getByText('Thumbnail pentru joc').first()).toBeVisible({ timeout: 20_000 });
  await page.evaluate(() => document.fonts.ready);
  await page.screenshot({ path: 'test-results/visual/prototype-v5-1440x900.png' });
});
