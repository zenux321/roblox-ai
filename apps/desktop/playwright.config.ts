import { defineConfig } from '@playwright/test';

/**
 * Verificări vizuale pe build-ul de producție servit local (cu CSP-ul real).
 * Capturile ajung în test-results/visual/ (ignorat de Git).
 */
export default defineConfig({
  testDir: './e2e',
  outputDir: './test-results/playwright',
  reporter: [['list']],
  workers: 1,
  use: {
    baseURL: 'http://127.0.0.1:4173',
    colorScheme: 'dark',
    browserName: 'chromium',
    locale: 'ro-RO',
  },
  webServer: {
    command: 'npm run build && npm run preview -- --host 127.0.0.1',
    url: 'http://127.0.0.1:4173',
    reuseExistingServer: false,
    timeout: 120_000,
  },
});
