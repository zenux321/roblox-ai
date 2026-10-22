import { ChevronDown, ChevronUp, Play } from 'lucide-react';
import { useState } from 'react';

import type { useCopy } from '../app/useCopy';
import type {
  AgentActivityMessage,
  AssistantMessage,
  Attachment,
  CodeSnippet,
  Person,
  PersonMessage,
  Visibility,
} from '../domain/types';
import { formatUsd, formatWhen, t } from '../i18n';
import { AiAvatar, PersonAvatar } from './Avatar';
import styles from './Messages.module.css';

type Copier = ReturnType<typeof useCopy>;

const EXTENSION_LABEL: Record<Attachment['kind'], string> = {
  image: 'IMG',
  video: 'MP4',
  doc: 'PDF',
  code: 'LUA',
};

function AttachmentView({
  attachment,
  visibility,
  peerName,
}: {
  attachment: Attachment;
  visibility: Visibility;
  peerName: string;
}) {
  const analyzed = attachment.analysis === 'analyzed';
  return (
    <div className={`${styles.attachment} ${styles[`att_${attachment.kind}`]}`}>
      {attachment.kind === 'image' && (
        <div className={styles.imageTile} role="img" aria-label={attachment.name}>
          <span className={styles.fileChip}>{attachment.name}</span>
        </div>
      )}
      {attachment.kind === 'video' && (
        <div className={styles.videoTile} title={t('chat.videoDemo')}>
          <div className={styles.videoStage}>
            <span className={styles.playButton} aria-hidden="true">
              <Play size={14} />
            </span>
          </div>
          <div className={styles.videoBar}>
            <span>0:00</span>
            <span className={styles.videoTrack} />
            <span>{attachment.meta.split(' · ')[0]}</span>
          </div>
        </div>
      )}
      {(attachment.kind === 'doc' || attachment.kind === 'code') && (
        <div className={styles.fileCard}>
          <div className={styles.fileHead}>
            <span className={styles.ext}>{EXTENSION_LABEL[attachment.kind]}</span>
            <span className={styles.fileText}>
              <span className={styles.fileName}>{attachment.name}</span>
              <span className={styles.fileMeta}>{attachment.meta}</span>
            </span>
          </div>
          {attachment.preview && <pre className={styles.filePreview}>{attachment.preview}</pre>}
        </div>
      )}
      <span className={`${styles.analysis} ${analyzed ? styles.analyzed : ''}`}>
        <span className={styles.analysisDot} />
        {analyzed ? t('chat.analyzed') : t('chat.attachedOnly')}
      </span>
      <span className={styles.audience}>
        {visibility === 'shared'
          ? t('chat.attachmentAudienceShared', { peer: peerName })
          : t('chat.attachmentAudiencePrivate')}
      </span>
      {attachment.note && (
        <span className={styles.attNote}>
          {attachment.note}{' '}
          {attachment.alternative && (
            <button type="button" className={styles.inlineLink} disabled title={t('sidebar.nextStage')}>
              {attachment.alternative}
            </button>
          )}
        </span>
      )}
    </div>
  );
}

export function PersonMessageView({
  message,
  author,
  visibility,
  peerName,
}: {
  message: PersonMessage;
  author: Person | undefined;
  visibility: Visibility;
  peerName: string;
}) {
  const palette = author?.palette ?? 'peer';
  return (
    <article className={styles.row} aria-label={author?.name}>
      <PersonAvatar initial={author?.initial ?? '?'} palette={palette} />
      <div className={styles.body}>
        <div className={styles.head}>
          <span className={`${styles.author} ${styles[palette]}`}>{author?.name ?? '?'}</span>
          <span className={styles.time}>{formatWhen(message.time)}</span>
        </div>
        {message.text && <div className={styles.bubble}>{message.text}</div>}
        {message.attachments.length > 0 && (
          <div className={styles.attachments}>
            {message.attachments.map((a) => (
              <AttachmentView key={a.id} attachment={a} visibility={visibility} peerName={peerName} />
            ))}
          </div>
        )}
      </div>
    </article>
  );
}

export function AgentActivityView({ message }: { message: AgentActivityMessage }) {
  const [open, setOpen] = useState(false);
  return (
    <div className={styles.agentCard}>
      <button
        type="button"
        className={styles.agentHead}
        aria-expanded={open}
        onClick={() => setOpen((v) => !v)}
      >
        <AiAvatar size={20} soft />
        <span className={styles.agentName}>{message.agent}</span>
        <span className={styles.agentTitle}>{message.title}</span>
        <span className={styles.agentPc}>{message.computer}</span>
        {open ? <ChevronUp size={13} aria-hidden="true" /> : <ChevronDown size={13} aria-hidden="true" />}
      </button>
      {open && (
        <ul className={styles.agentDetails}>
          {message.details.map((d) => (
            <li key={d}>{d}</li>
          ))}
        </ul>
      )}
    </div>
  );
}

function CodeBlock({ code, copier, copyKey }: { code: CodeSnippet; copier: Copier; copyKey: string }) {
  return (
    <div className={styles.code}>
      <div className={styles.codeHead}>
        <span className={styles.codeLang}>{code.language}</span>
        <span className={styles.codeFile}>{code.file}</span>
        <button
          type="button"
          className={styles.codeCopy}
          onClick={() => void copier.copy(copyKey, code.code)}
        >
          {copier.label(copyKey)}
        </button>
      </div>
      <pre className={styles.codeBody}>{code.code}</pre>
    </div>
  );
}

function ImageGenerationCard({ owner, limitUsd }: { owner: string; limitUsd: number }) {
  return (
    <section className={styles.genCard} aria-label={t('chat.imageGen.title')}>
      <div className={styles.genHead}>
        <span className={styles.genTitle}>{t('chat.imageGen.title')}</span>
        <span className={styles.genState}>{t('chat.imageGen.unconfigured')}</span>
      </div>
      <div className={styles.genMetrics}>
        <div>
          <span>{t('chat.imageGen.variants')}</span>
          <strong>{t('chat.imageGen.variantsValue')}</strong>
        </div>
        <div>
          <span>{t('chat.imageGen.estimate')}</span>
          <strong className={styles.mono}>{t('chat.imageGen.estimateValue')}</strong>
        </div>
        <div>
          <span>{t('chat.imageGen.limit')}</span>
          <strong className={styles.mono}>{formatUsd(limitUsd)}</strong>
        </div>
        <div>
          <span>{t('chat.imageGen.payer')}</span>
          <strong>{t('chat.imageGen.payerValue', { owner })}</strong>
        </div>
      </div>
      <div className={styles.genBody}>
        <span className={styles.genRefs}>{t('chat.imageGen.references')}</span>
        <span className={styles.genText}>{t('chat.imageGen.explain')}</span>
        <div className={styles.buttonRow}>
          <button type="button" className={styles.buttonLight} disabled title={t('chat.imageGen.later')}>
            {t('chat.imageGen.configure')}
          </button>
          <button type="button" className={styles.buttonOutline} disabled title={t('chat.imageGen.later')}>
            {t('chat.imageGen.import')}
          </button>
        </div>
      </div>
    </section>
  );
}

export interface AssistantViewProps {
  message: AssistantMessage;
  copier: Copier;
  owner: string;
  limitUsd: number;
  /** Butonul „Pregătește un plan” dispare după ce planul există. */
  planPrepared: boolean;
  canRetry: boolean;
  onRetry: (message: AssistantMessage) => void;
  onPreparePlan: () => void;
  onOpenPlan: () => void;
}

export function AssistantMessageView(props: AssistantViewProps) {
  const { message, copier } = props;
  const text = [...message.paragraphs, ...(message.list ?? [])].join('\n');
  return (
    <article className={styles.row} aria-label={t('chat.assistant')}>
      <AiAvatar />
      <div className={styles.body}>
        <div className={styles.head}>
          <span className={styles.author}>{t('chat.assistant')}</span>
          <span className={styles.time}>
            {message.model} · {formatWhen(message.time)}
          </span>
          {message.tag && (
            <span className={`${styles.tag} ${message.stopped ? styles.tagWarning : ''}`}>{message.tag}</span>
          )}
          {message.attempt > 1 && (
            <span className={styles.tag}>{t('chat.attempt', { n: message.attempt })}</span>
          )}
        </div>
        {message.paragraphs.map((p, i) => (
          <p key={i} className={`${styles.paragraph} ${message.stopped ? styles.paragraphStopped : ''}`}>
            {p}
          </p>
        ))}
        {message.list && message.list.length > 0 && (
          <ul className={styles.list}>
            {message.list.map((item) => (
              <li key={item}>{item}</li>
            ))}
          </ul>
        )}
        {message.code && <CodeBlock code={message.code} copier={copier} copyKey={`${message.id}:code`} />}
        {message.extras.map((extra, i) => {
          switch (extra.type) {
            case 'image_generation_unconfigured':
              return <ImageGenerationCard key={i} owner={props.owner} limitUsd={props.limitUsd} />;
            case 'prepare_plan':
              return props.planPrepared ? null : (
                <button key={i} type="button" className={styles.planButton} onClick={props.onPreparePlan}>
                  {t('chat.preparePlan')}
                </button>
              );
            case 'plan_card':
              return (
                <section key={i} className={styles.planCard} aria-label={extra.title}>
                  <div className={styles.planHead}>
                    <span className={styles.planTitle}>{extra.title}</span>
                    <span className={styles.planState}>{t('chat.planAwaiting')}</span>
                  </div>
                  <span className={styles.planSummary}>{extra.summary}</span>
                  <div className={styles.planActions}>
                    <button type="button" className={styles.buttonAccent} onClick={props.onOpenPlan}>
                      {t('chat.openPlan')}
                    </button>
                    <span className={styles.planApprover}>
                      {t('chat.canApprove', { who: extra.approver })}
                    </span>
                  </div>
                </section>
              );
            case 'associate_project':
              return (
                <div key={i} className={styles.associate}>
                  <span>{t('chat.associateText')}</span>
                  <button type="button" className={styles.buttonDark} disabled title={t('sidebar.nextStage')}>
                    {t('chat.associate')}
                  </button>
                </div>
              );
          }
        })}
        <div className={styles.footer}>
          <button
            type="button"
            className={styles.footerLink}
            onClick={() => void copier.copy(message.id, text)}
          >
            {copier.label(message.id)}
          </button>
          <button
            type="button"
            className={styles.footerLink}
            disabled={!props.canRetry}
            onClick={() => props.onRetry(message)}
          >
            {t('chat.retry')}
          </button>
        </div>
      </div>
    </article>
  );
}

export function ThinkingView() {
  return (
    <div className={styles.row} role="status">
      <AiAvatar />
      <div className={styles.thinking}>
        <span className={styles.pulse}>{t('chat.thinking')}</span>
        <span className={styles.skeletonLine} style={{ width: '90%' }} />
        <span className={styles.skeletonLine} style={{ width: '55%' }} />
      </div>
    </div>
  );
}

export function StreamingView({ text, model }: { text: string; model: string }) {
  return (
    <div className={styles.row} role="status" aria-live="polite">
      <AiAvatar />
      <div className={styles.body}>
        <div className={styles.head}>
          <span className={styles.author}>{t('chat.assistant')}</span>
          <span className={styles.time}>
            {model} · {t('chat.writing')}
          </span>
        </div>
        <p className={`${styles.paragraph} ${styles.streaming}`}>
          {text}
          <span className={styles.cursor} aria-hidden="true" />
        </p>
      </div>
    </div>
  );
}

export function HistorySkeleton() {
  return (
    <div className={styles.skeleton} role="status">
      <span className={styles.pulse}>{t('chat.loadingHistory')}</span>
      <div className={styles.skeletonRow}>
        <span className={styles.skeletonAvatar} />
        <div className={styles.skeletonCol}>
          <span className={styles.skeletonLine} style={{ width: '30%' }} />
          <span className={styles.skeletonBlock} style={{ width: '70%' }} />
        </div>
      </div>
      <div className={styles.skeletonRow}>
        <span className={`${styles.skeletonAvatar} ${styles.skeletonSquare}`} />
        <div className={styles.skeletonCol}>
          <span className={styles.skeletonLine} style={{ width: '24%' }} />
          <span className={styles.skeletonLine} style={{ width: '90%' }} />
          <span className={styles.skeletonLine} style={{ width: '80%' }} />
        </div>
      </div>
    </div>
  );
}
