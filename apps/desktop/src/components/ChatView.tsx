import { useEffect, useRef, useState } from 'react';

import { useCopy } from '../app/useCopy';
import type { ReplyProgress } from '../app/useWorkspace';
import type {
  AssistantMessage,
  ContextItem,
  ConversationSummary,
  Message,
  Person,
  ProviderStatus,
  Visibility,
} from '../domain/types';
import { chatSuggestions, t } from '../i18n';
import { Composer } from './Composer';
import styles from './ChatView.module.css';
import {
  AgentActivityView,
  AssistantMessageView,
  HistorySkeleton,
  PersonMessageView,
  StreamingView,
  ThinkingView,
} from './Messages';

interface ChatViewProps {
  conversation: ConversationSummary;
  messages: Message[] | undefined;
  loading: boolean;
  reply: ReplyProgress | null;
  people: Person[];
  projectName: string;
  owner: string;
  limitUsd: number;
  provider: ProviderStatus;
  busy: boolean;
  context: ContextItem[];
  rightOpen: boolean;
  onToggleRight: () => void;
  onToggleContext: (id: string) => void;
  onRename: (title: string) => void;
  onSetVisibility: (visibility: Visibility) => void;
  onSend: (text: string) => Promise<boolean>;
  onStop: () => void;
  onRetry: (message: AssistantMessage) => void;
  onPreparePlan: () => void;
  onOpenPlan: () => void;
  demoMenu: React.ReactNode;
}

function attachmentCount(messages: Message[] | undefined): number {
  return (messages ?? []).reduce((n, m) => n + (m.kind === 'person' ? m.attachments.length : 0), 0);
}

function ConversationHeader(props: ChatViewProps & { peerName: string }) {
  const { conversation } = props;
  const [renaming, setRenaming] = useState(false);
  const [value, setValue] = useState(conversation.title);
  const shared = conversation.visibility === 'shared';
  const selected = props.context.filter((c) => c.selected).length;

  const startRename = () => {
    setValue(conversation.title);
    setRenaming(true);
  };
  const commit = () => {
    setRenaming(false);
    if (value.trim() && value.trim() !== conversation.title) {
      props.onRename(value);
    }
  };

  return (
    <div className={styles.header}>
      <div className={styles.headerText}>
        {renaming ? (
          <input
            className={styles.renameInput}
            value={value}
            autoFocus
            aria-label={t('chat.rename')}
            onChange={(e) => setValue(e.target.value)}
            onBlur={commit}
            onKeyDown={(e) => {
              if (e.key === 'Enter') {
                commit();
              } else if (e.key === 'Escape') {
                setRenaming(false);
              }
            }}
          />
        ) : (
          <div className={styles.titleRow}>
            <h1 className={styles.title} title={t('chat.rename')} onDoubleClick={startRename}>
              {conversation.title}
            </h1>
            <button type="button" className={styles.renameLink} onClick={startRename}>
              {t('chat.rename')}
            </button>
          </div>
        )}
        <div className={styles.subRow}>
          <span className={`${styles.visibility} ${shared ? styles.visShared : ''}`}>
            {shared ? t('chat.visShared', { peer: props.peerName }) : t('chat.visPrivate')}
          </span>
          <span className={styles.sub}>
            {conversation.scope === 'general'
              ? t('chat.subGeneral')
              : t('chat.subProject', { project: props.projectName, count: selected })}
          </span>
        </div>
      </div>
      {props.demoMenu}
      <button type="button" className={styles.headerButton} disabled title={t('composer.attachLater')}>
        {t('chat.attachments')}
        <span className={styles.count}>{attachmentCount(props.messages)}</span>
      </button>
      <button
        type="button"
        className={`${styles.headerButton} ${styles.muted}`}
        onClick={props.onToggleRight}
      >
        {props.rightOpen ? t('chat.hidePanel') : t('chat.showPanel')}
      </button>
    </div>
  );
}

function EmptyConversation(props: ChatViewProps & { peerName: string }) {
  const { conversation } = props;
  const canPick = conversation.isNew && conversation.scope === 'project';
  const shared = conversation.visibility === 'shared';
  return (
    <div className={styles.empty}>
      <h2 className={styles.emptyTitle}>{t('chat.emptyTitle')}</h2>
      <p className={styles.emptyText}>{t('chat.emptyText')}</p>
      {canPick && (
        <div className={styles.segmented} role="radiogroup" aria-label={t('chat.visPrivate')}>
          <button
            type="button"
            role="radio"
            aria-checked={!shared}
            className={`${styles.segment} ${!shared ? styles.segmentPrivate : ''}`}
            onClick={() => props.onSetVisibility('private')}
          >
            {t('chat.pickPrivate')}
          </button>
          <button
            type="button"
            role="radio"
            aria-checked={shared}
            className={`${styles.segment} ${shared ? styles.segmentShared : ''}`}
            onClick={() => props.onSetVisibility('shared')}
          >
            {t('chat.pickShared', { peer: props.peerName })}
          </button>
        </div>
      )}
      <div className={styles.suggestions}>
        {chatSuggestions.map((s) => (
          <button
            key={s}
            type="button"
            className={styles.suggestion}
            disabled={props.busy || props.provider.kind === 'missing'}
            onClick={() => void props.onSend(s)}
          >
            {s}
          </button>
        ))}
      </div>
    </div>
  );
}

export function ChatView(props: ChatViewProps) {
  const { conversation, messages, reply } = props;
  const copier = useCopy();
  const scrollRef = useRef<HTMLDivElement>(null);
  const peer = props.people.find((p) => !p.isSelf);
  const peerName = peer?.name ?? '';
  const replyHere = reply?.conversationId === conversation.id ? reply : null;
  const providerReady = props.provider.kind !== 'missing';

  useEffect(() => {
    const el = scrollRef.current;
    if (el && typeof el.scrollTo === 'function') {
      el.scrollTo({ top: el.scrollHeight });
    }
  }, [messages?.length, replyHere?.text, replyHere?.phase, conversation.id]);

  const thread = messages ?? [];
  const isEmpty = !props.loading && messages !== undefined && thread.length === 0 && !replyHere;
  const planIndex = thread.findIndex(
    (m) => m.kind === 'assistant' && m.extras.some((e) => e.type === 'plan_card'),
  );

  return (
    <section className={styles.chat} aria-label={conversation.title}>
      <ConversationHeader {...props} peerName={peerName} />
      <div className={styles.scroll} ref={scrollRef}>
        <div className={styles.column}>
          {props.loading && <HistorySkeleton />}
          {isEmpty && <EmptyConversation {...props} peerName={peerName} />}
          {!props.loading &&
            thread.map((m, i) => {
              switch (m.kind) {
                case 'person':
                  return (
                    <PersonMessageView
                      key={m.id}
                      message={m}
                      author={props.people.find((p) => p.id === m.authorId)}
                      visibility={conversation.visibility}
                      peerName={peerName}
                    />
                  );
                case 'agent':
                  return <AgentActivityView key={m.id} message={m} />;
                case 'assistant':
                  return (
                    <AssistantMessageView
                      key={m.id}
                      message={m}
                      copier={copier}
                      owner={props.owner}
                      limitUsd={props.limitUsd}
                      planPrepared={planIndex > i}
                      canRetry={!props.busy && providerReady}
                      onRetry={props.onRetry}
                      onPreparePlan={props.onPreparePlan}
                      onOpenPlan={props.onOpenPlan}
                    />
                  );
              }
            })}
          {replyHere?.phase === 'thinking' && <ThinkingView />}
          {replyHere?.phase === 'streaming' && <StreamingView text={replyHere.text} model="model simulat" />}
        </div>
      </div>
      <div className={styles.composerArea}>
        <Composer
          key={conversation.id}
          visibility={conversation.visibility}
          peerName={peerName}
          provider={props.provider}
          busy={props.busy}
          context={props.context}
          onToggleContext={props.onToggleContext}
          onSend={props.onSend}
          onStop={props.onStop}
        />
      </div>
    </section>
  );
}
