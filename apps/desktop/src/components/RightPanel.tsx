import { useState } from 'react';

import type { PlanView, RunView, WorkspaceSnapshot } from '../domain/types';
import { t } from '../i18n';
import { AiAvatar, PersonAvatar } from './Avatar';
import styles from './RightPanel.module.css';

type TabKey = 'project' | 'agents' | 'tasks' | 'team';
const TABS: { key: TabKey; badge?: 'warning' | 'danger' }[] = [
  { key: 'project', badge: 'warning' },
  { key: 'agents', badge: 'danger' },
  { key: 'tasks' },
  { key: 'team' },
];

interface RightPanelProps {
  snapshot: WorkspaceSnapshot;
  onDiscuss: () => void;
  canDiscuss: boolean;
  plan: PlanView | null;
  run: RunView | null;
  onOpenPlan: () => void;
}

function PlanStatusCard({
  plan,
  run,
  onOpenPlan,
}: {
  plan: PlanView | null;
  run: RunView | null;
  onOpenPlan: () => void;
}) {
  const percent = run && run.total ? Math.round((run.done / run.total) * 100) : 0;
  return (
    <div className={styles.planCard}>
      <div className={styles.sectionHead}>
        <span className={styles.sectionTitle}>{t('panel.currentPlan')}</span>
        {plan && (
          <span className={styles.planState}>
            {run ? t(`run.statusLabels.${run.status}`) : t(`plan.statusLabels.${plan.status}`)}
          </span>
        )}
      </div>
      {plan ? (
        <>
          <span className={styles.current}>
            Plan v{plan.version} · {plan.title}
          </span>
          {run && (
            <div
              className={styles.progress}
              role="progressbar"
              aria-valuenow={percent}
              aria-valuemin={0}
              aria-valuemax={100}
              aria-label={t('panel.currentPlan')}
            >
              <span
                className={`${styles.progressFill} ${styles.fill_accent}`}
                style={{ width: `${percent}%` }}
              />
            </div>
          )}
          <button type="button" className={styles.buttonDark} onClick={onOpenPlan}>
            {t('panel.openPlan')}
          </button>
        </>
      ) : (
        <span className={styles.faint}>{t('panel.noPlan')}</span>
      )}
    </div>
  );
}

export function RightPanel({ snapshot, onDiscuss, canDiscuss, plan, run, onOpenPlan }: RightPanelProps) {
  const [tab, setTab] = useState<TabKey>('project');
  const { project, status } = snapshot;

  return (
    <aside className={styles.panel} aria-label={t('panel.tabs.project')}>
      <div className={styles.tabs} role="tablist">
        {TABS.map(({ key, badge }) => (
          <button
            key={key}
            type="button"
            role="tab"
            id={`tab-${key}`}
            aria-selected={tab === key}
            aria-controls={`tabpanel-${key}`}
            className={`${styles.tab} ${tab === key ? styles.tabActive : ''}`}
            onClick={() => setTab(key)}
          >
            {t(`panel.tabs.${key}`)}
            {badge && <span className={`${styles.badge} ${styles[badge]}`} />}
          </button>
        ))}
      </div>
      <div className={styles.content} role="tabpanel" id={`tabpanel-${tab}`} aria-labelledby={`tab-${tab}`}>
        {tab === 'project' && (
          <div className={styles.stack18}>
            <PlanStatusCard plan={plan} run={run} onOpenPlan={onOpenPlan} />
            <div className={styles.titleBlock}>
              <span className={styles.title}>{project.name}</span>
              <span className={styles.faint}>{project.kindLabel}</span>
            </div>
            <div className={styles.connections}>
              {project.connections.map((c) => (
                <div key={c.label} className={styles.connRow}>
                  <span className={`${styles.connMark} ${styles[`conn_${c.state}`]}`} aria-hidden="true">
                    {c.state === 'connected' ? '✓' : '–'}
                  </span>
                  <span className={styles.connText}>
                    <span className={styles.connLabel}>
                      {c.label}
                      <span className={styles.connState}>{t(`panel.connection.${c.state}`)}</span>
                    </span>
                    <span className={styles.connValue}>{c.value}</span>
                  </span>
                </div>
              ))}
            </div>
            <div className={styles.stack8}>
              <div className={styles.sectionHead}>
                <span className={styles.sectionTitle}>{t('panel.whatFound')}</span>
                <span className={styles.demoChip}>{t('panel.demoExample')}</span>
              </div>
              {project.findings.map((f) =>
                f.kind === 'difference' ? (
                  <div key={f.text} className={styles.difference}>
                    <span className={styles.differenceTitle}>{f.title}</span>
                    <span>{f.text}</span>
                  </div>
                ) : (
                  <div key={f.text} className={styles.info}>
                    {f.text}
                  </div>
                ),
              )}
              <button type="button" className={styles.buttonDark} onClick={onDiscuss} disabled={!canDiscuss}>
                {t('panel.discuss')}
              </button>
            </div>
          </div>
        )}

        {tab === 'agents' && (
          <div className={styles.stack10}>
            <span className={styles.intro}>
              {t('panel.agentsIntro', {
                steps: status.limits.maxAgentSteps,
                retries: status.limits.maxRetries,
                minutes: status.limits.taskTimeoutMinutes,
              })}
            </span>
            {snapshot.agents.map((a) => (
              <div key={a.id} className={styles.card}>
                <div className={styles.cardHead}>
                  <AiAvatar size={20} soft />
                  <span className={styles.cardTitle}>{a.name}</span>
                  <span className={`${styles.state} ${styles[a.tone]}`}>{a.state}</span>
                </div>
                <span className={styles.current}>{a.current}</span>
                <span className={styles.faint}>{t('panel.lastResult', { value: a.lastResult })}</span>
                <span className={styles.mono}>
                  {a.id} · {a.tools} → {a.output}
                </span>
                <span className={styles.faint}>{a.computer}</span>
              </div>
            ))}
          </div>
        )}

        {tab === 'tasks' && (
          <div className={styles.stack10}>
            {snapshot.tasks.map((task) => (
              <div key={task.title} className={`${styles.card} ${styles.gap8}`}>
                <div className={styles.taskHead}>
                  <span className={styles.taskTitle}>{task.title}</span>
                  <span className={`${styles.state} ${styles[task.tone]}`}>{task.state}</span>
                </div>
                <div className={styles.who}>
                  {task.assignee === 'agent' ? (
                    <AiAvatar size={20} soft />
                  ) : (
                    <PersonAvatar initial={task.who.charAt(0)} palette={task.assignee} size={20} />
                  )}
                  <span>{task.who}</span>
                </div>
                {task.progress !== undefined && (
                  <div
                    className={styles.progress}
                    role="progressbar"
                    aria-valuenow={task.progress}
                    aria-valuemin={0}
                    aria-valuemax={100}
                    aria-label={task.title}
                  >
                    <span
                      className={`${styles.progressFill} ${styles[`fill_${task.tone}`]}`}
                      style={{ width: `${task.progress}%` }}
                    />
                  </div>
                )}
                {task.dependency && <span className={styles.faint}>{task.dependency}</span>}
              </div>
            ))}
          </div>
        )}

        {tab === 'team' && (
          <div className={styles.stack16}>
            <div className={styles.stack8}>
              {snapshot.members.map((m) => (
                <div key={m.name} className={`${styles.card} ${styles.member}`}>
                  <PersonAvatar initial={m.initial} palette={m.palette} size={32} online={m.online} />
                  <div className={styles.memberText}>
                    <div className={styles.memberHead}>
                      <span className={styles.cardTitleInline}>{m.name}</span>
                      <span className={styles.role}>{m.role}</span>
                    </div>
                    <span className={styles.current12}>{m.status}</span>
                    <span className={styles.faint}>{m.permissions}</span>
                  </div>
                </div>
              ))}
            </div>
            <div className={styles.teamButtons}>
              <button
                type="button"
                className={styles.buttonAccent}
                disabled
                title={t('titleBar.inviteLater')}
              >
                {t('panel.inviteCollaborator')}
              </button>
              <button
                type="button"
                className={styles.buttonSurface}
                disabled
                title={t('titleBar.inviteLater')}
              >
                {t('panel.joinProject')}
              </button>
            </div>
            <div className={styles.stack8}>
              <span className={styles.sectionTitle}>{t('panel.computers')}</span>
              {snapshot.computers.map((c) => (
                <div key={c.name} className={styles.computer}>
                  <div className={styles.cardHead}>
                    <span className={`${styles.dot} ${styles[`fill_${c.tone}`]}`} />
                    <span className={styles.computerName}>{c.name}</span>
                    <span className={styles.faint}>{c.acceptance}</span>
                  </div>
                  <span className={styles.muted12}>
                    {c.model} · {c.load}
                  </span>
                  <span className={styles.current12}>{c.task}</span>
                </div>
              ))}
              <span className={styles.faint}>{t('panel.computersNote')}</span>
            </div>
          </div>
        )}
      </div>
    </aside>
  );
}
