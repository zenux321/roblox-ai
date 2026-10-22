import { ChevronDown, Plus } from 'lucide-react';
import { useState } from 'react';

import { usePopover } from '../app/usePopover';
import type { ContextItem, ProviderStatus, Visibility } from '../domain/types';
import { t } from '../i18n';
import styles from './Composer.module.css';

interface ComposerProps {
  visibility: Visibility;
  peerName: string;
  provider: ProviderStatus;
  busy: boolean;
  context: ContextItem[];
  onToggleContext: (id: string) => void;
  onSend: (text: string) => Promise<boolean>;
  onStop: () => void;
}

export function rowsFor(text: string): number {
  return Math.min(8, Math.max(2, text.split('\n').length));
}

export function Composer(props: ComposerProps) {
  const [draft, setDraft] = useState('');
  const [claudeTried, setClaudeTried] = useState(false);
  const { open: modelOpen, toggle: modelToggle, anchor: modelAnchor } = usePopover();
  const { open: ctxOpen, toggle: ctxToggle, anchor: ctxAnchor } = usePopover();
  const missing = props.provider.kind === 'missing';
  const canSend = !missing && !props.busy && draft.trim().length > 0;
  const selected = props.context.filter((c) => c.selected).length;

  const submit = async () => {
    if (!canSend) {
      return;
    }
    const text = draft;
    setDraft('');
    const accepted = await props.onSend(text);
    if (!accepted) {
      setDraft(text);
    }
  };

  return (
    <div className={styles.wrap}>
      {missing && (
        <div className={styles.banner} role="alert">
          <span className={styles.bannerTitle}>{t('composer.missingTitle')}</span>
          <span className={styles.bannerText}>{t('composer.missingText')}</span>
        </div>
      )}
      <div className={styles.box}>
        <textarea
          className={styles.input}
          value={draft}
          rows={rowsFor(draft)}
          disabled={missing}
          aria-label={t('composer.placeholder')}
          placeholder={missing ? t('composer.placeholderMissing') : t('composer.placeholder')}
          onChange={(e) => setDraft(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === 'Enter' && !e.shiftKey && !e.nativeEvent.isComposing) {
              e.preventDefault();
              void submit();
            }
          }}
        />
        <div className={styles.toolbar}>
          <button type="button" className={styles.tool} disabled title={t('composer.attachLater')}>
            <Plus size={15} aria-hidden="true" />
            {t('composer.attach')}
          </button>

          <div className={styles.popWrap} ref={modelAnchor}>
            <button type="button" className={styles.tool} aria-expanded={modelOpen} onClick={modelToggle}>
              <span className={`${styles.dot} ${missing ? styles.dotDanger : styles.dotOk}`} />
              {props.provider.label}
              <ChevronDown size={12} className={styles.caret} aria-hidden="true" />
            </button>
            {modelOpen && (
              <div className={`${styles.popover} ${styles.modelPopover}`}>
                <div className={`${styles.option} ${missing ? '' : styles.optionActive}`}>
                  <span className={styles.optionTitle}>{t('composer.model.demo')}</span>
                  <span className={missing ? styles.optionSub : styles.optionSubOk}>
                    {t('composer.model.demoSub')}
                  </span>
                </div>
                <div className={styles.option} aria-disabled="true">
                  <span className={`${styles.optionTitle} ${styles.muted}`}>
                    {t('composer.model.ollama')}
                  </span>
                  <span className={styles.optionSub}>{t('composer.model.ollamaSub')}</span>
                </div>
                <button type="button" className={styles.option} onClick={() => setClaudeTried(true)}>
                  <span className={`${styles.optionTitle} ${styles.muted}`}>
                    {t('composer.model.claude')}
                  </span>
                  <span className={styles.optionSub}>{t('composer.model.claudeSub')}</span>
                </button>
                {claudeTried && (
                  <div className={styles.warn} role="alert">
                    {t('composer.model.claudeWarn')}
                  </div>
                )}
              </div>
            )}
          </div>

          <div className={styles.popWrap} ref={ctxAnchor}>
            <button type="button" className={styles.tool} aria-expanded={ctxOpen} onClick={ctxToggle}>
              {t('composer.context', { count: selected })}
              <ChevronDown size={12} className={styles.caret} aria-hidden="true" />
            </button>
            {ctxOpen && (
              <div className={`${styles.popover} ${styles.contextPopover}`}>
                <span className={styles.contextTitle}>{t('composer.contextTitle')}</span>
                {props.context.map((item) => (
                  <label key={item.id} className={styles.check}>
                    <input
                      type="checkbox"
                      checked={item.selected}
                      onChange={() => props.onToggleContext(item.id)}
                    />
                    <span className={styles.checkText}>
                      <span>{item.title}</span>
                      <span className={styles.checkGroup}>{item.group}</span>
                    </span>
                  </label>
                ))}
                <span className={styles.contextNote}>
                  {t('composer.contextNote', { peer: props.peerName })}
                </span>
              </div>
            )}
          </div>

          <span className={styles.flex} />
          {props.busy ? (
            <button type="button" className={styles.stop} onClick={props.onStop}>
              {t('composer.stop')}
            </button>
          ) : (
            <button
              type="button"
              className={`${styles.send} ${canSend ? styles.sendReady : ''}`}
              disabled={!canSend}
              onClick={() => void submit()}
            >
              {t('composer.send')}
            </button>
          )}
        </div>
      </div>
      <div className={styles.hint}>
        <span>
          {props.visibility === 'shared'
            ? t('composer.visShared', { peer: props.peerName })
            : t('composer.visPrivate')}
        </span>
        <span>{t('composer.shortcuts')}</span>
      </div>
    </div>
  );
}
