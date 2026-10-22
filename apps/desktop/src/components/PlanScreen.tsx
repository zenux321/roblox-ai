import { useState } from 'react';

import type { PlanRun } from '../app/usePlanRun';
import type { FileDiff, PlanView, RunEventView, RunResults, RunView } from '../domain/types';
import { formatDateTime, formatDuration, formatTime, formatUsd, t, verdictText } from '../i18n';
import styles from './PlanScreen.module.css';

export type PlanTab = 'plan' | 'execution' | 'results';

interface PlanScreenProps {
  planRun: PlanRun;
  tab: PlanTab;
  onTab: (tab: PlanTab) => void;
  onBack: () => void;
}

const TERMINAL = new Set(['succeeded', 'failed', 'cancelled']);

function EmptyCard({
  title,
  text,
  action,
  onAction,
}: {
  title: string;
  text: string;
  action: string;
  onAction: () => void;
}) {
  return (
    <div className={styles.empty}>
      <h2 className={styles.emptyTitle}>{title}</h2>
      <p className={styles.emptyText}>{text}</p>
      <button type="button" className={styles.buttonSecondary} onClick={onAction}>
        {action}
      </button>
    </div>
  );
}

function PlanTabView({
  plan,
  run,
  planRun,
  onBack,
}: {
  plan: PlanView;
  run: RunView | null;
  planRun: PlanRun;
  onBack: () => void;
}) {
  const { pending } = planRun.state;
  const awaiting = plan.status === 'awaiting_approval';
  const approval = plan.approval;
  const canStart = plan.status === 'approved' && approval?.valid && (!run || TERMINAL.has(run.status));
  return (
    <div className={styles.column}>
      <div className={styles.titleRow}>
        <div className={styles.titleText}>
          <h2 className={styles.planTitle}>
            {t('plan.title', { version: plan.version, title: plan.title })}
          </h2>
          <p className={styles.objective}>{plan.objective}</p>
        </div>
        <div className={styles.versions}>
          <span className={styles.versionsLabel}>{t('plan.versions')}</span>
          {plan.version > 1 && <span className={styles.versionChip}>v{plan.version - 1}</span>}
          <span className={`${styles.versionChip} ${styles.versionCurrent}`}>
            {t('plan.current', { version: plan.version })}
          </span>
        </div>
      </div>

      <div className={styles.metrics}>
        <div className={styles.metric}>
          <span>{t('plan.metricDuration')}</span>
          <strong>{plan.estimatedDuration}</strong>
        </div>
        <div className={styles.metric}>
          <span>{t('plan.metricCost')}</span>
          <strong>{formatUsd(plan.costEstimateUsd)}</strong>
        </div>
        <div className={styles.metric}>
          <span>{t('plan.metricLimit')}</span>
          <strong>{formatUsd(plan.costLimitUsd)}</strong>
        </div>
        <div className={styles.metric}>
          <span>{t('plan.metricRepairs')}</span>
          <strong>{t('plan.repairsValue', { n: plan.maxRetries })}</strong>
        </div>
      </div>

      <section className={styles.section}>
        <h3 className={styles.sectionTitle}>{t('plan.steps')}</h3>
        <div className={styles.table} role="table" aria-label={t('plan.steps')}>
          <div className={`${styles.tableRow} ${styles.tableHead}`} role="row">
            <span role="columnheader">{t('plan.colNumber')}</span>
            <span role="columnheader">{t('plan.colWhat')}</span>
            <span role="columnheader">{t('plan.colAgent')}</span>
            <span role="columnheader">{t('plan.colWhere')}</span>
            <span role="columnheader">{t('plan.colZone')}</span>
            <span role="columnheader">{t('plan.colAfter')}</span>
          </div>
          {plan.tasks.map((task) => (
            <div key={task.id} className={styles.tableRow} role="row">
              <span className={styles.stepNumber} role="cell">
                {task.number}
              </span>
              <span className={styles.stepText} role="cell">
                <span className={styles.stepTitle}>{task.title}</span>
                <span className={styles.faint}>{task.description}</span>
              </span>
              <span className={styles.cellText} role="cell">
                {task.agent}
              </span>
              <span className={styles.cellText} role="cell">
                {task.location}
              </span>
              <span className={styles.cellMono} role="cell">
                {task.zone}
              </span>
              <span className={styles.faint} role="cell">
                {task.after}
              </span>
            </div>
          ))}
        </div>
      </section>

      <div className={styles.twoColumns}>
        <section className={styles.section}>
          <h3 className={styles.sectionTitle}>{t('plan.tests')}</h3>
          <ul className={styles.listCard}>
            {plan.tests.map((test) => (
              <li key={test.id} className={styles.testRow}>
                <span className={styles.circle} aria-hidden="true">
                  ○
                </span>
                <span className={styles.flex}>{test.title}</span>
                <span className={`${styles.chip} ${test.kind === 'studio' ? styles.chipWarn : ''}`}>
                  {test.kind === 'static' ? t('plan.testStatic') : t('plan.testStudio')}
                </span>
              </li>
            ))}
          </ul>
          <p className={styles.note}>{t('plan.testsNote')}</p>
        </section>
        <section className={styles.section}>
          <h3 className={styles.sectionTitle}>{t('plan.permissions')}</h3>
          <dl className={styles.listCard}>
            {plan.permissions.map((row) => (
              <div key={row.label} className={styles.permissionRow}>
                <dt>{row.label}</dt>
                <dd>{row.value}</dd>
              </div>
            ))}
          </dl>
        </section>
      </div>

      <div className={styles.approvalRow}>
        {awaiting && (
          <>
            <button
              type="button"
              className={styles.buttonApprove}
              disabled={pending}
              onClick={() => void planRun.approve()}
            >
              {t('plan.approve', { version: plan.version })}
            </button>
            <button type="button" className={styles.buttonOutline} onClick={onBack}>
              {t('plan.requestChanges')}
            </button>
            <span className={styles.note}>
              {approval?.revokedAt
                ? t('plan.revokedAt', { when: formatDateTime(approval.revokedAt) })
                : t('plan.approveNote')}
            </span>
          </>
        )}
        {plan.status === 'approved' && approval && (
          <>
            <div className={styles.signature}>
              <span>
                {t('plan.approvedBy', {
                  who: approval.approvedBy,
                  when: formatDateTime(approval.approvedAt),
                })}
              </span>
              <span className={styles.signatureMono}>
                {t('plan.fingerprint', {
                  short: `${approval.planSha256.slice(0, 4)}…${approval.planSha256.slice(-4)}`,
                })}
              </span>
            </div>
            {canStart && (
              <button
                type="button"
                className={styles.buttonApprove}
                disabled={pending}
                onClick={() => void planRun.start()}
              >
                {t('plan.startRun')}
              </button>
            )}
            <button
              type="button"
              className={styles.buttonDanger}
              disabled={pending}
              onClick={() => void planRun.revoke()}
            >
              {t('plan.revoke')}
            </button>
          </>
        )}
      </div>
    </div>
  );
}

const MARKS: Record<string, string> = {
  pending: '○',
  running: '•',
  succeeded: '✓',
  failed: '✕',
  recovery_required: '!',
};

function ExecutionTabView({
  plan,
  run,
  events,
  planRun,
  onTab,
}: {
  plan: PlanView;
  run: RunView;
  events: RunEventView[];
  planRun: PlanRun;
  onTab: (tab: PlanTab) => void;
}) {
  const percent = run.total ? Math.round((run.done / run.total) * 100) : 0;
  const { pending } = planRun.state;
  const active = run.live && ['queued', 'running'].includes(run.status);
  return (
    <div className={styles.column}>
      <div className={styles.progressCard}>
        <div className={styles.progressMain}>
          <div className={styles.progressHead}>
            <span className={styles.faint13}>{t('run.job', { version: plan.version })}</span>
            <span className={`${styles.statusBadge} ${styles[`status_${run.status}`]}`}>
              {t(`run.statusLabels.${run.status}`)}
            </span>
          </div>
          <span className={styles.activity}>{run.activity}</span>
          <div
            className={styles.bar}
            role="progressbar"
            aria-valuemin={0}
            aria-valuemax={100}
            aria-valuenow={percent}
            aria-label={t('run.job', { version: plan.version })}
          >
            <span className={styles.barFill} style={{ width: `${percent}%` }} />
          </div>
          <div className={styles.progressMeta}>
            <span>{t('run.doneLabel', { done: run.done, total: run.total })}</span>
            <span>{t('run.startedAt', { time: formatTime(run.startedAt) })}</span>
            <span>{t('run.costSoFar')}</span>
          </div>
        </div>
        <span className={styles.percent}>{percent}%</span>
      </div>

      {run.status === 'recovery_required' && (
        <div className={styles.block} role="alert">
          <span className={styles.blockTitle}>{t('run.recoveryTitle')}</span>
          <span className={styles.blockText}>{t('run.recoveryText')}</span>
          <div className={styles.buttonRow}>
            <button
              type="button"
              className={styles.buttonWarn}
              disabled={pending}
              onClick={() => void planRun.recover()}
            >
              {t('run.recoveryGo')}
            </button>
            <button
              type="button"
              className={styles.buttonOutline}
              disabled={pending}
              onClick={() => void planRun.cancel()}
            >
              {t('run.stop')}
            </button>
          </div>
        </div>
      )}

      <div className={styles.buttonRow}>
        {active && (
          <button
            type="button"
            className={styles.buttonOutline}
            disabled={pending}
            onClick={() => void planRun.pause()}
          >
            {t('run.pause')}
          </button>
        )}
        {run.status === 'paused' && (
          <button
            type="button"
            className={styles.buttonApproveSmall}
            disabled={pending}
            onClick={() => void planRun.resume()}
          >
            {t('run.resume')}
          </button>
        )}
        {!TERMINAL.has(run.status) && run.status !== 'recovery_required' && (
          <button
            type="button"
            className={styles.buttonDanger}
            disabled={pending}
            onClick={() => void planRun.cancel()}
          >
            {t('run.stop')}
          </button>
        )}
        {run.hasResults && (
          <button type="button" className={styles.buttonApproveSmall} onClick={() => onTab('results')}>
            {t('run.viewResults')}
          </button>
        )}
      </div>

      <div className={styles.execGrid}>
        <section className={styles.section}>
          <h3 className={styles.sectionTitle}>{t('run.steps')}</h3>
          <ol className={styles.stepList}>
            {run.tasks.map((task) => (
              <li
                key={task.id}
                className={`${styles.stepItem} ${task.state === 'running' ? styles.stepActive : ''}`}
              >
                <span className={`${styles.stepDot} ${styles[`dot_${task.state}`]}`} aria-hidden="true">
                  {MARKS[task.state]}
                </span>
                <span className={styles.stepText}>
                  <span className={styles.stepTitle}>{task.title}</span>
                  <span className={styles.faint}>
                    {t(`run.taskStates.${task.state}`)}
                    {task.attempt > 0 && ` · ${t('run.attempt', { n: task.attempt })}`}
                  </span>
                </span>
              </li>
            ))}
          </ol>
        </section>
        <section className={styles.section}>
          <h3 className={styles.sectionTitle}>{t('run.log')}</h3>
          <ol className={styles.log} aria-live="polite">
            {events.map((event) => (
              <li key={event.seq} className={styles.logLine}>
                <span className={styles.logTime}>{formatTime(event.at)}</span>
                <span className={styles[`log_${event.level}`]}>{event.message}</span>
              </li>
            ))}
          </ol>
          <p className={styles.note}>{t('run.backgroundNote')}</p>
        </section>
      </div>
    </div>
  );
}

function DiffViewer({ changes }: { changes: FileDiff[] }) {
  const [selected, setSelected] = useState(0);
  const file = changes[selected] ?? changes[0];
  if (!file) {
    return <p className={styles.note}>{t('results.noChanges')}</p>;
  }
  return (
    <div className={styles.diff}>
      <div className={styles.diffTabs} role="tablist" aria-label={t('results.changes')}>
        {changes.map((c, i) => (
          <button
            key={c.path}
            type="button"
            role="tab"
            aria-selected={c.path === file.path}
            className={`${styles.diffTab} ${c.path === file.path ? styles.diffTabActive : ''}`}
            onClick={() => setSelected(i)}
          >
            <span className={styles.mono12}>{c.path.split('/').pop()}</span>
            <span className={styles.added}>+{c.added}</span>
            <span className={styles.removed}>−{c.removed}</span>
          </button>
        ))}
      </div>
      <div className={styles.diffPath}>
        {file.path}
        {file.isNew && <span className={styles.newChip}>{t('results.newFile')}</span>}
      </div>
      <div className={styles.diffBody} role="tabpanel">
        {file.lines.map((line, i) => (
          <div key={i} className={`${styles.diffLine} ${styles[`line_${line.kind}`]}`}>
            <span className={styles.lineNumber}>{line.number ?? ''}</span>
            <span className={styles.lineSign}>
              {line.kind === 'add' ? '+' : line.kind === 'del' ? '−' : ''}
            </span>
            <span className={styles.lineText}>{line.text}</span>
          </div>
        ))}
      </div>
    </div>
  );
}

function ResultsTabView({ results }: { results: RunResults }) {
  const [title, text] = verdictText(results.verdict);
  const marks = { passed: '✓', failed: '✕', not_run: '–' } as const;
  return (
    <div className={`${styles.column} ${styles.wide}`}>
      <div className={`${styles.verdict} ${styles[`verdict_${results.verdict}`]}`} role="status">
        <div className={styles.verdictText}>
          <span className={styles.verdictTitle}>{title}</span>
          <span>{text}</span>
        </div>
        <div className={styles.verdictStats}>
          <div>
            <span>{t('results.duration')}</span>
            <strong>{formatDuration(results.durationSeconds)}</strong>
          </div>
          <div>
            <span>{t('results.cost')}</span>
            <strong>{formatUsd(results.costUsd)}</strong>
          </div>
          <div>
            <span>{t('results.changes')}</span>
            <strong>{t('results.changesValue', { n: results.changes.length })}</strong>
          </div>
        </div>
      </div>

      <div className={styles.resultsGrid}>
        <div className={styles.column16}>
          <section className={styles.section}>
            <h3 className={styles.sectionTitle}>{t('results.tests')}</h3>
            <ul className={styles.listCard}>
              {results.tests.map((test) => (
                <li key={test.id} className={styles.resultRow}>
                  <span
                    className={`${styles.resultMark} ${styles[`mark_${test.status}`]}`}
                    aria-label={test.status}
                  >
                    {marks[test.status]}
                  </span>
                  <span className={styles.stepText}>
                    <span>{test.title}</span>
                    <span className={styles.faint}>{test.note}</span>
                  </span>
                </li>
              ))}
            </ul>
          </section>
          <section className={styles.section}>
            <h3 className={styles.sectionTitle}>{t('results.limitations')}</h3>
            <ul className={`${styles.listCard} ${styles.limitations}`}>
              {results.limitations.map((l) => (
                <li key={l}>{l}</li>
              ))}
            </ul>
          </section>
        </div>
        <section className={`${styles.section} ${styles.minw0}`}>
          <h3 className={styles.sectionTitle}>{t('results.changes')}</h3>
          <DiffViewer changes={results.changes} />
        </section>
      </div>

      <div className={styles.actionsRow}>
        {results.reportPath && (
          <span className={styles.mono12}>{t('results.report', { path: results.reportPath })}</span>
        )}
        <button type="button" className={styles.buttonSurface} disabled title={t('results.later')}>
          {t('results.openStudio')}
        </button>
        <button type="button" className={styles.buttonSurface} disabled title={t('results.later')}>
          {t('results.commit')}
        </button>
        <button type="button" className={styles.buttonSurface} disabled title={t('results.later')}>
          {t('results.pr')}
        </button>
        <span className={styles.flex} />
        <div className={styles.publish}>
          <button type="button" className={styles.buttonOutline} disabled title={t('results.later')}>
            {t('results.publish')}
          </button>
          <span className={styles.faint}>{t('results.publishNote')}</span>
        </div>
      </div>
    </div>
  );
}

export function PlanScreen({ planRun, tab, onTab, onBack }: PlanScreenProps) {
  const { plan, run, events, results, error } = planRun.state;
  const dots: Record<PlanTab, string | undefined> = {
    plan: plan?.status === 'awaiting_approval' ? styles.dotWarning : undefined,
    execution:
      run && !TERMINAL.has(run.status)
        ? run.status === 'recovery_required'
          ? styles.dotWarning
          : styles.dotAccent
        : undefined,
    results: results ? styles.dotSuccess : undefined,
  };
  return (
    <section className={styles.screen} aria-label={t('plan.screenTitle')}>
      <div className={styles.header}>
        <button type="button" className={styles.back} onClick={onBack}>
          {t('plan.back')}
        </button>
        <nav className={styles.tabs} role="tablist" aria-label={t('plan.screenTitle')}>
          {(['plan', 'execution', 'results'] as const).map((key) => (
            <button
              key={key}
              type="button"
              role="tab"
              aria-selected={tab === key}
              className={`${styles.tab} ${tab === key ? styles.tabActive : ''}`}
              onClick={() => onTab(key)}
            >
              {t(`plan.tabs.${key}`)}
              {dots[key] && <span className={`${styles.tabDot} ${dots[key]}`} />}
            </button>
          ))}
        </nav>
        {plan && <span className={styles.headerStatus}>{t(`plan.statusLabels.${plan.status}`)}</span>}
      </div>
      <div className={styles.scroll}>
        {error && (
          <div className={styles.error} role="alert">
            {t('plan.error', { message: error })}
          </div>
        )}
        {!plan && (
          <EmptyCard
            title={t('plan.emptyTitle')}
            text={t('plan.emptyText')}
            action={t('plan.backToChat')}
            onAction={onBack}
          />
        )}
        {plan && tab === 'plan' && <PlanTabView plan={plan} run={run} planRun={planRun} onBack={onBack} />}
        {plan &&
          tab === 'execution' &&
          (run ? (
            <ExecutionTabView plan={plan} run={run} events={events} planRun={planRun} onTab={onTab} />
          ) : (
            <EmptyCard
              title={t('run.emptyTitle')}
              text={t('run.emptyText')}
              action={t('run.openPlan')}
              onAction={() => onTab('plan')}
            />
          ))}
        {plan &&
          tab === 'results' &&
          (results ? (
            <ResultsTabView key={run?.id} results={results} />
          ) : (
            <EmptyCard
              title={t('results.emptyTitle')}
              text={t('results.emptyText')}
              action={t('results.viewExecution')}
              onAction={() => onTab('execution')}
            />
          ))}
      </div>
    </section>
  );
}

export function RecoveryDialog({
  onRecover,
  onLater,
  pending,
}: {
  onRecover: () => void;
  onLater: () => void;
  pending: boolean;
}) {
  return (
    <div className={styles.overlay}>
      <div className={styles.modal} role="dialog" aria-modal="true" aria-labelledby="recovery-title">
        <span className={styles.modalKicker}>{t('run.recoveryTitle')}</span>
        <h2 id="recovery-title" className={styles.modalTitle}>
          {t('run.recoveryHeadline')}
        </h2>
        <p className={styles.modalText}>{t('run.recoveryText')}</p>
        <div className={styles.buttonRow}>
          <button
            type="button"
            className={styles.buttonApprove}
            autoFocus
            disabled={pending}
            onClick={onRecover}
          >
            {t('run.recoveryGo')}
          </button>
          <button type="button" className={styles.buttonOutline} onClick={onLater}>
            {t('run.recoveryLater')}
          </button>
        </div>
      </div>
    </div>
  );
}
