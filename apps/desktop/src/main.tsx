import './theme/global.css';

import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';

import { App } from './app/App';
import { MockWorkspaceGateway } from './mock/mockGateway';

const root = document.getElementById('root');
if (root) {
  // Etapa 1: numai date demonstrative. Gateway-ul real (worker) vine în M1/M2.
  createRoot(root).render(
    <StrictMode>
      <App gateway={new MockWorkspaceGateway()} />
    </StrictMode>,
  );
}
