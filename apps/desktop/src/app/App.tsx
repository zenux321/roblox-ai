import { useCallback, useEffect, useState } from 'react';

import { ChatView } from '../components/ChatView';
import { DemoMenu, type DemoAction } from '../components/DemoMenu';
import { PlanScreen, RecoveryDialog, type PlanTab } from '../components/PlanScreen';
import { RightPanel } from '../components/RightPanel';
import { Sidebar } from '../components/Sidebar';
import { TitleBar } from '../components/TitleBar';
import type { DemoControls, WorkspaceGateway } from '../domain/types';
import { t } from '../i18n';
import { OWNER_NAME } from '../mock/demoData';
import { fetchRuntimeInfo, inDesktopShell, type RuntimeInfo } from '../platform/tauri';
import styles from './App.module.css';
import { usePlanRun } from './usePlanRun';
import { useWorkspace } from './useWorkspace';

interface AppProps {
  gateway: WorkspaceGateway;
  /** Prezent doar în previzualizarea din browser (gateway demonstrativ). */
  demo?: DemoControls;
}

export function App({ gateway, demo }: AppProps) {
  const ws = useWorkspace(gateway);
  const planRun = usePlanRun(gateway);
  const { state } = ws;
  const [leftOpen, setLeftOpen] = useState(true);
  const [rightOpen, setRightOpen] = useState(true);
  const [view, setView] = useState<'chat' | 'plan'>('chat');
  const [planTab, setPlanTab] = useState<PlanTab>('plan');
  const [recoveryDismissed, setRecoveryDismissed] = useState<string | null>(null);
  const [context, setContext] = useState<Record<string, boolean>>({});
  const [runtime, setRuntime] = useState<RuntimeInfo | null>(null);
  const desktop = inDesktopShell();
  const { plan, run } = planRun.state;

  useEffect(() => {
    fetchRuntimeInfo()
      .then(setRuntime)
      .catch(() => setRuntime(null));
  }, []);

  const openPlan = useCallback(() => {
    setView('plan');
    setPlanTab(run && !['succeeded', 'failed', 'cancelled'].includes(run.status) ? 'execution' : 'plan');
  }, [run]);

  const onDemo = useCallback(
    (action: DemoAction) => {
      if (!demo) {
        return;
      }
      const activeId = state.activeId;
      switch (action) {
        case 'normal':
          demo.setSlowLoading(false);
          ws.applyStatus(demo.setProviderAvailable(true));
          break;
        case 'loading':
          demo.setSlowLoading(true);
          if (activeId) {
            void ws.reloadConversation(activeId).finally(() => demo.setSlowLoading(false));
          }
          break;
        case 'empty':
          setView('chat');
          void ws.createConversation('project');
          break;
        case 'model_missing':
          ws.stop();
          ws.applyStatus(demo.setProviderAvailable(state.snapshot?.status.provider.kind === 'missing'));
          break;
        case 'restart':
          if (demo.simulateRestart()) {
            setRecoveryDismissed(null);
            void planRun.refresh();
          }
          break;
      }
    },
    [demo, ws, planRun, state.activeId, state.snapshot?.status.provider.kind],
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
  const crumb =
    view === 'plan'
      ? t('plan.screenTitle')
      : conversation?.scope === 'general'
        ? t('app.generalChat')
        : snapshot.project.name;
  const planDot =
    run && ['queued', 'running'].includes(run.status)
      ? 'accent'
      : plan?.status === 'awaiting_approval' || run?.status === 'recovery_required'
        ? 'warning'
        : null;
  const showRecovery = run?.status === 'recovery_required' && recoveryDismissed !== run.id;
  const demoMenu = demo ? (
    <DemoMenu onAction={onDemo} providerMissing={snapshot.status.provider.kind === 'missing'} />
  ) : null;

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
          activeId={view === 'chat' ? (conversation?.id ?? null) : null}
          query={state.query}
          hits={state.hits}
          onToggle={() => setLeftOpen((v) => !v)}
          onSearch={(q) => void ws.search(q)}
          onOpen={(id) => {
            setView('chat');
            void ws.openConversation(id);
          }}
          onCreate={(scope) => {
            setView('chat');
            void ws.createConversation(scope);
          }}
          planActive={view === 'plan'}
          planDot={planDot}
          onOpenPlan={openPlan}
        />
        {view === 'plan' ? (
          <PlanScreen planRun={planRun} tab={planTab} onTab={setPlanTab} onBack={() => setView('chat')} />
        ) : (
          conversation && (
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
              onPreparePlan={() =>
                void ws
                  .preparePlan(conversation.id)
                  .then(planRun.adoptPlan)
                  .catch(() => undefined)
              }
              onOpenPlan={openPlan}
              demoMenu={demoMenu}
            />
          )
        )}
        {rightOpen ? (
          <RightPanel
            snapshot={snapshot}
            canDiscuss={ws.providerReady && !ws.busy}
            onDiscuss={() => {
              setView('chat');
              void ws.send(t('panel.discussPrompt'));
            }}
            plan={plan}
            run={run}
            onOpenPlan={openPlan}
          />
        ) : (
          <div className={styles.rightPlaceholder} />
        )}
        {showRecovery && (
          <RecoveryDialog
            pending={planRun.state.pending}
            onRecover={() => {
              setView('plan');
              setPlanTab('execution');
              void planRun.recover();
            }}
            onLater={() => setRecoveryDismissed(run.id)}
          />
        )}
      </div>
    </div>
  );
}
