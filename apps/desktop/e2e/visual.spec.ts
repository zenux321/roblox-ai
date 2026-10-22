import { expect, type Page, test } from '@playwright/test';

const SHOTS = 'test-results/visual';
const VIEWPORTS: [number, number][] = [
  [1920, 1080],
  [1440, 900],
  [1280, 720],
];

declare global {
  interface Window {
    __cspViolations?: string[];
  }
}

const external: string[] = [];

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    window.__cspViolations = [];
    document.addEventListener('securitypolicyviolation', (e) => {
      window.__cspViolations?.push(`${e.violatedDirective} ${e.blockedURI}`);
    });
  });
  page.on('request', (req) => {
    const url = new URL(req.url());
    if (url.hostname !== '127.0.0.1' && url.protocol !== 'data:') {
      external.push(req.url());
    }
  });
});

test.afterEach(async ({ page }) => {
  const violations = await page.evaluate(() => window.__cspViolations ?? []);
  expect(violations, 'încălcări CSP').toEqual([]);
  expect(external, 'cereri către alte domenii').toEqual([]);
});

async function open(page: Page, width: number, height: number) {
  await page.setViewportSize({ width, height });
  await page.goto('/');
  await expect(page.getByRole('heading', { name: 'Thumbnail pentru joc' })).toBeVisible();
  await expect(page.getByText(/Am făcut două capturi/)).toBeVisible();
  await page.evaluate(() => document.fonts.ready);
}

async function checkLayout(page: Page, width: number, height: number) {
  const fonts = await page.evaluate(() => ({
    onest: document.fonts.check('15px Onest', 'ășțîâ'),
    mono: document.fonts.check('13px "JetBrains Mono"'),
  }));
  expect(fonts).toEqual({ onest: true, mono: true });
  const overflow = await page.evaluate(() => document.documentElement.scrollWidth);
  expect(overflow).toBeLessThanOrEqual(width);
  const send = await page.getByRole('button', { name: 'Trimite' }).boundingBox();
  expect(send).not.toBeNull();
  expect((send?.y ?? 0) + (send?.height ?? 0)).toBeLessThanOrEqual(height);
}

for (const [width, height] of VIEWPORTS) {
  test(`spațiul de lucru ${width}×${height}`, async ({ page }) => {
    await open(page, width, height);
    await checkLayout(page, width, height);
    await page.screenshot({ path: `${SHOTS}/workspace-${width}x${height}.png` });
  });
}

test('panouri restrânse și conversație cu cod 1280×720', async ({ page }) => {
  await open(page, 1280, 720);
  await page.getByRole('button', { name: /De ce checkpointul e pe server/ }).click();
  await expect(page.getByText('CheckpointService.server.luau')).toBeVisible();
  await page.getByRole('button', { name: 'Restrânge' }).click();
  await page.getByRole('button', { name: 'Ascunde panoul' }).click();
  await page.waitForTimeout(300);
  await checkLayout(page, 1280, 720);
  await page.screenshot({ path: `${SHOTS}/collapsed-1280x720.png` });
});

test('text lung în română, scroll fără tăierea composerului 1280×720', async ({ page }) => {
  await open(page, 1280, 720);
  const long =
    'Aș vrea ca nivelul următor să aibă platforme de gheață care se topesc încet, ' +
    'un checkpoint încălzit și o cascadă de lavă înghețată; ' +
    'ține cont că jucătorii mai mici folosesc telefonul și trebuie să vadă bine butoanele. '.repeat(3);
  const composer = page.getByRole('textbox', { name: 'Scrie un mesaj…' });
  await composer.fill(long);
  await composer.press('Enter');
  await expect(page.getByText(/Serverul e sursa de adevăr/).last()).toBeVisible({ timeout: 10_000 });
  await expect(page.getByRole('button', { name: 'Trimite' })).toBeVisible();
  await checkLayout(page, 1280, 720);
  await page.screenshot({ path: `${SHOTS}/long-text-1280x720.png` });
});

test('navigare cu tastatura până la composer', async ({ page }) => {
  await open(page, 1440, 900);
  let reached = false;
  for (let i = 0; i < 60 && !reached; i++) {
    await page.keyboard.press('Tab');
    reached = await page.evaluate(() => document.activeElement?.tagName === 'TEXTAREA');
  }
  expect(reached).toBe(true);
  const outline = await page.evaluate(() => getComputedStyle(document.activeElement as Element).outlineStyle);
  expect(outline).not.toBe('');
  await page.screenshot({ path: `${SHOTS}/keyboard-focus-1440x900.png` });
});

test('stări: răspuns în curs, model lipsă, conversație goală', async ({ page }) => {
  await open(page, 1440, 900);
  const composer = page.getByRole('textbox', { name: 'Scrie un mesaj…' });
  await composer.fill('Explică-mi diferențele');
  await composer.press('Enter');
  await expect(page.getByText('Citesc contextul din vault…')).toBeVisible();
  await page.screenshot({ path: `${SHOTS}/state-thinking-1440x900.png` });
  await expect(page.getByRole('button', { name: 'Oprește răspunsul' })).toBeVisible();
  await page.getByRole('button', { name: 'Oprește răspunsul' }).click();
  await expect(page.getByText('Răspuns oprit')).toBeVisible();

  await page.getByRole('button', { name: 'Demo: stări' }).click();
  await page.getByRole('menuitem', { name: 'Model lipsă' }).click();
  await expect(page.getByText('Modelul local lipsește')).toBeVisible();
  await page.screenshot({ path: `${SHOTS}/state-model-missing-1440x900.png` });

  await page.getByRole('button', { name: 'Demo: stări' }).click();
  await page.getByRole('menuitem', { name: 'Model demo disponibil' }).click();
  await page.getByRole('button', { name: '+ Nouă' }).click();
  await expect(page.getByText('Despre ce vrei să vorbim?')).toBeVisible();
  await page.screenshot({ path: `${SHOTS}/state-empty-1440x900.png` });

  await page.getByRole('tab', { name: 'Agenți' }).click();
  await page.screenshot({ path: `${SHOTS}/panel-agents-1440x900.png` });
});
