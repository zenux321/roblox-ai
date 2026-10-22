import { Minus, Square, X } from 'lucide-react';

import { usePopover } from '../app/usePopover';
import type { MemberCard, StatusSnapshot } from '../domain/types';
import { formatUsd, t } from '../i18n';
import type { RuntimeInfo } from '../platform/tauri';
import { windowControls } from '../platform/tauri';
import { PersonAvatar } from './Avatar';
import styles from './TitleBar.module.css';

interface TitleBarProps {
  crumb: string;
  status: StatusSnapshot;
  members: MemberCard[];
  runtime: RuntimeInfo | null;
  desktop: boolean;
}

export function TitleBar({ crumb, status, members, runtime, desktop }: TitleBarProps) {
  const { open: demoOpen, toggle: demoToggle, anchor: demoAnchor } = usePopover();
  const peers = members.filter((m) => m.palette === 'peer');

  return (
    <header className={styles.bar} data-tauri-drag-region>
      <span className={styles.logo} aria-hidden="true">
        AI
      </span>
      <span className={styles.name}>{t('app.name')}</span>
      <span className={styles.crumb}>/ {crumb}</span>
      <div className={styles.spacer} data-tauri-drag-region />

      <div className={styles.pills}>
        <div ref={demoAnchor} className={styles.demoWrap}>
          <button
            type="button"
            className={`${styles.pill} ${styles.demoPill}`}
            aria-expanded={demoOpen}
            onClick={demoToggle}
          >
            <span className={styles.demoDot} />
            <span className={styles.strong}>{t('app.demoBadge')}</span>
            <span className={styles.sub}>{t('app.demoBadgeSub')}</span>
          </button>
          {demoOpen && (
            <div className={styles.demoPopover} role="dialog" aria-label={t('app.demoTitle')}>
              <span className={styles.popTitle}>{t('app.demoTitle')}</span>
              <span className={styles.popText}>{t('app.demoText')}</span>
              <span className={styles.popTitle}>{t('app.coreTitle')}</span>
              {runtime ? (
                <>
                  <span className={styles.popText}>
                    {t('app.coreVersion', { app: runtime.app_version, core: runtime.core_version })}
                  </span>
                  <ul className={styles.verdicts}>
                    {runtime.example_configs.map((c) => (
                      <li key={c.name}>
                        <span className={styles.mono}>{c.name}</span>:{' '}
                        {c.contract_valid ? t('app.configContractOk') : t('app.configContractBad')} ·{' '}
                        {c.execution_ready ? t('app.configExecReady') : t('app.configExecBlocked')}
                        {c.execution_issue_codes.length > 0 && (
                          <span className={styles.codes}> ({c.execution_issue_codes.join(', ')})</span>
                        )}
                      </li>
                    ))}
                  </ul>
                </>
              ) : (
                <span className={styles.popText}>{t('app.coreBrowser')}</span>
              )}
            </div>
          )}
        </div>

        <div className={styles.pill} title={t('titleBar.connectionTitle')}>
          <span className={styles.dot} />
          <span className={styles.strong}>{t('titleBar.solo')}</span>
          <span className={styles.sub}>{t('titleBar.soloSub')}</span>
        </div>

        <div className={styles.pill}>
          <span className={styles.dot} />
          <span className={styles.strong}>{t('titleBar.studio')}</span>
          <span className={styles.sub}>{t('titleBar.studioNotConfigured')}</span>
        </div>

        <div className={styles.pill}>
          <span className={styles.sub}>{t('titleBar.cost')}</span>
          <span className={styles.money}>{formatUsd(status.cost.spentUsd)}</span>
        </div>

        <div className={styles.people}>
          <PersonAvatar initial="A" palette="self" size={20} />
          {peers.map((p) => (
            <PersonAvatar key={p.name} initial={p.initial} palette="peer" size={20} online={p.online} />
          ))}
          <button type="button" className={styles.invite} disabled title={t('titleBar.inviteLater')}>
            {t('titleBar.invite')}
          </button>
        </div>
      </div>

      {desktop && (
        <div className={styles.windowControls}>
          <button
            type="button"
            className={styles.winButton}
            aria-label={t('titleBar.minimize')}
            onClick={() => void windowControls.minimize()}
          >
            <Minus size={14} />
          </button>
          <button
            type="button"
            className={styles.winButton}
            aria-label={t('titleBar.maximize')}
            onClick={() => void windowControls.toggleMaximize()}
          >
            <Square size={12} />
          </button>
          <button
            type="button"
            className={`${styles.winButton} ${styles.winClose}`}
            aria-label={t('titleBar.close')}
            onClick={() => void windowControls.close()}
          >
            <X size={15} />
          </button>
        </div>
      )}
    </header>
  );
}
