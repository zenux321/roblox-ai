import { useCallback, useEffect, useState } from 'react';

import { ChatView } from '../components/ChatView';
import { DemoMenu, type DemoAction } from '../components/DemoMenu';
import { RightPanel } from '../components/RightPanel';
import { Sidebar } from '../components/Sidebar';
import { TitleBar } from '../components/TitleBar';
import type { DemoControls, WorkspaceGateway } from '../domain/types';
import { t } from '../i18n';
import { OWNER_NAME } from '../mock/demoData';
import { fetchRuntimeInfo, inDesktopShell, type RuntimeInfo } from '../platform/tauri';
import styles from './App.module.css';
import { useWorkspace } from './useWorkspace';

interface AppProps {
  gateway: WorkspaceGateway & DemoControls;
}

export function App({ gateway }: AppProps) {
  const ws = useWorkspace(gateway);
  const { state } = ws;
  const [leftOpen, setLeftOpen] = useState(true);
  const [rightOpen, setRightOpen] = useState(true);
  const [context, setContext] = useState<Record<string, boolean>>({});
  const [runtime, setRuntime] = useState<RuntimeInfo | null>(null);
  const desktop = inDesktopShell();

  useEffect(() => {
    fetchRuntimeInfo()
      .then(setRuntime)
      .catch(() => setRuntime(null));
  }, []);

  const onDemo = useCallback(
    (action: DemoAction) => {
      const activeId = state.activeId;
      switch (action) {
        case 'normal':
          gateway.setSlowLoading(false);
          ws.applyStatus(gateway.setProviderAvailable(true));
          break;
        case 'loading':
          gateway.setSlowLoading(true);
          if (activeId) {
            void ws.reloadConversation(activeId).finally(() => gateway.setSlowLoading(false));
          }
          break;
        case 'empty':
          void ws.createConversation('project');
          break;
        case 'model_missing':
          ws.stop();
          ws.applyStatus(gateway.setProviderAvailable(state.snapshot?.status.provider.kind === 'missing'));
          break;
      }
    },
    [gateway, ws, state.activeId, state.snapshot?.status.provider.kind],
  );

  if (!state.snapshot) {
    return (
      <div className={styles.boot}>
        {state.loadError ? (
          <>
            <span>{t('app.loadError')}</span>
            <button type="button" className={styles.retry} onClick={() => void ws.retryLoad()}>
              {t('app.retry')}
            </button>
          </>
        ) : (
          <span className={styles.pulse}>{t('chat.loadingHistory')}</span>
        )}
      </div>
    );
  }

  const snapshot = state.snapshot;
  const conversation =
    snapshot.conversations.find((c) => c.id === state.activeId) ?? snapshot.conversations[0];
  const contextItems = snapshot.context.map((c) => ({ ...c, selected: context[c.id] ?? c.selected }));
  const crumb = conversation?.scope === 'general' ? t('app.generalChat') : snapshot.project.name;

  return (
    <div className={styles.app}>
      <TitleBar
        crumb={crumb}
        status={snapshot.status}
        members={snapshot.members}
        runtime={runtime}
        desktop={desktop}
      />
      <div
        className={`${styles.grid} ${leftOpen ? '' : styles.leftClosed} ${rightOpen ? '' : styles.rightClosed}`}
        data-testid="workspace-grid"
      >
        <Sidebar
          collapsed={!leftOpen}
          project={snapshot.project}
          recentProjects={snapshot.recentProjects}
          conversations={snapshot.conversations}
          activeId={conversation?.id ?? null}
          query={state.query}
          hits={state.hits}
          onToggle={() => setLeftOpen((v) => !v)}
          onSearch={(q) => void ws.search(q)}
          onOpen={(id) => void ws.openConversation(id)}
          onCreate={(scope) => void ws.createConversation(scope)}
        />
        {conversation && (
          <ChatView
            conversation={conversation}
            messages={state.threads[conversation.id]}
            loading={state.loadingId === conversation.id}
            reply={state.reply}
            people={snapshot.people}
            projectName={snapshot.project.name}
            owner={OWNER_NAME}
            limitUsd={snapshot.status.cost.limitUsd}
            provider={snapshot.status.provider}
            busy={ws.busy}
            context={contextItems}
            rightOpen={rightOpen}
            onToggleRight={() => setRightOpen((v) => !v)}
            onToggleContext={(id) =>
              setContext((c) => ({
                ...c,
                [id]: !(c[id] ?? snapshot.context.find((x) => x.id === id)?.selected ?? false),
              }))
            }
            onRename={(title) => void ws.rename(conversation.id, title)}
            onSetVisibility={(v) => void ws.setVisibility(conversation.id, v)}
            onSend={ws.send}
            onStop={ws.stop}
            onRetry={ws.retry}
            onPreparePlan={() => void ws.preparePlan(conversation.id)}
            demoMenu={
              <DemoMenu onAction={onDemo} providerMissing={snapshot.status.provider.kind === 'missing'} />
            }
          />
        )}
        {rightOpen ? (
          <RightPanel
            snapshot={snapshot}
            canDiscuss={ws.providerReady && !ws.busy}
            onDiscuss={() => void ws.send(t('panel.discussPrompt'))}
          />
        ) : (
          <div className={styles.rightPlaceholder} />
        )}
      </div>
    </div>
  );
}
