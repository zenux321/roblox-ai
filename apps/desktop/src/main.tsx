import './theme/global.css';

import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';

import { App } from './app/App';
import { MockWorkspaceGateway } from './mock/mockGateway';
import { inDesktopShell } from './platform/tauri';
import { TauriWorkspaceGateway } from './platform/tauriGateway';

const root = document.getElementById('root');
if (root) {
  // În aplicația desktop: serviciul Rust (SQLite). În browser: date demonstrative.
  const app = inDesktopShell() ? (
    <App gateway={new TauriWorkspaceGateway()} />
  ) : (
    (() => {
      const mock = new MockWorkspaceGateway();
      return <App gateway={mock} demo={mock} />;
    })()
  );
  createRoot(root).render(<StrictMode>{app}</StrictMode>);
}
