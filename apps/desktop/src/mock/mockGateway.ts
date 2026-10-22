/**
 * Gateway demonstrativ, în memorie. Simulează latența și scrierea progresivă,
 * dar logica de oprire, reîncercare, căutare și vizibilitate este cea reală a
 * contractului. Datele se pierd la restart până la persistența din M1.
 */
import type {
  AssistantMessage,
  ConversationScope,
  ConversationSummary,
  DemoControls,
  Message,
  PersonMessage,
  ReplyEvent,
  ReplyRequest,
  StatusSnapshot,
  Visibility,
  WorkspaceGateway,
  WorkspaceSnapshot,
} from '../domain/types';
import { t } from '../i18n';
import {
  AGENTS,
  COMPUTERS,
  CONTEXT,
  CONVERSATIONS,
  DEMO_MODEL_LABEL,
  MEMBERS,
  OWNER_NAME,
  PEOPLE,
  PROJECT,
  RECENT_PROJECTS,
  TASKS,
  THREADS,
} from './demoData';
import { demoReplyFor, plainText } from './replies';

export interface MockTiming {
  loadMs: number;
  slowLoadMs: number;
  thinkingMs: number;
  tickMs: number;
  charsPerTick: number;
}

export const DEFAULT_TIMING: MockTiming = {
  loadMs: 450,
  slowLoadMs: 3000,
  thinkingMs: 900,
  tickMs: 40,
  charsPerTick: 5,
};

export class ProviderMissingError extends Error {
  constructor() {
    super('provider_missing');
  }
}

function abortError(): DOMException {
  return new DOMException('Aborted', 'AbortError');
}

function sleep(ms: number, signal?: AbortSignal): Promise<void> {
  return new Promise((resolve, reject) => {
    if (signal?.aborted) {
      reject(abortError());
      return;
    }
    const timer = setTimeout(() => {
      signal?.removeEventListener('abort', onAbort);
      resolve();
    }, ms);
    const onAbort = () => {
      clearTimeout(timer);
      reject(abortError());
    };
    signal?.addEventListener('abort', onAbort, { once: true });
  });
}

function nowLabel(): string {
  return 'acum';
}

function messageText(message: Message): string {
  switch (message.kind) {
    case 'person':
      return message.text;
    case 'assistant':
      return [...message.paragraphs, ...(message.list ?? [])].join(' ');
    case 'agent':
      return `${message.agent} ${message.title}`;
  }
}

export class MockWorkspaceGateway implements WorkspaceGateway, DemoControls {
  readonly isDemo = true;
  private conversations: ConversationSummary[];
  private readonly threads: Map<string, Message[]>;
  private status: StatusSnapshot;
  private slowLoading = false;
  private seq = 100;

  constructor(private readonly timing: MockTiming = DEFAULT_TIMING) {
    this.conversations = structuredClone(CONVERSATIONS);
    this.threads = new Map(Object.entries(structuredClone(THREADS)));
    this.status = {
      connection: { kind: 'solo' },
      studio: { state: 'not_configured' },
      cost: { spentUsd: 0, limitUsd: 0 },
      provider: { kind: 'demo', label: t('composer.model.demo') },
      limits: { maxAgentSteps: 12, maxRetries: 2, taskTimeoutMinutes: 10 },
    };
  }

  private nextId(prefix: string): string {
    this.seq += 1;
    return `${prefix}-${this.seq}`;
  }

  private thread(id: string): Message[] {
    const thread = this.threads.get(id);
    if (!thread) {
      throw new Error(`Conversație necunoscută: ${id}`);
    }
    return thread;
  }

  private summary(id: string): ConversationSummary {
    const conversation = this.conversations.find((c) => c.id === id);
    if (!conversation) {
      throw new Error(`Conversație necunoscută: ${id}`);
    }
    return conversation;
  }

  private update(id: string, patch: Partial<ConversationSummary>): ConversationSummary {
    const updated = { ...this.summary(id), ...patch };
    this.conversations = this.conversations.map((c) => (c.id === id ? updated : c));
    return { ...updated };
  }

  async loadWorkspace(): Promise<WorkspaceSnapshot> {
    await sleep(0);
    return {
      project: structuredClone(PROJECT),
      recentProjects: structuredClone(RECENT_PROJECTS),
      people: structuredClone(PEOPLE),
      conversations: structuredClone(this.conversations),
      agents: structuredClone(AGENTS),
      tasks: structuredClone(TASKS),
      members: structuredClone(MEMBERS),
      computers: structuredClone(COMPUTERS),
      context: structuredClone(CONTEXT),
      status: structuredClone(this.status),
    };
  }

  async loadMessages(conversationId: string): Promise<Message[]> {
    await sleep(this.slowLoading ? this.timing.slowLoadMs : this.timing.loadMs);
    return structuredClone(this.thread(conversationId));
  }

  async searchConversations(query: string): Promise<string[]> {
    await sleep(0);
    const needle = query.trim().toLocaleLowerCase('ro');
    if (!needle) {
      return this.conversations.map((c) => c.id);
    }
    return this.conversations
      .filter(
        (c) =>
          c.title.toLocaleLowerCase('ro').includes(needle) ||
          (this.threads.get(c.id) ?? []).some((m) => messageText(m).toLocaleLowerCase('ro').includes(needle)),
      )
      .map((c) => c.id);
  }

  async createConversation(scope: ConversationScope): Promise<ConversationSummary> {
    await sleep(0);
    const conversation: ConversationSummary = {
      id: this.nextId('conv'),
      title: t('sidebar.newConversationTitle'),
      visibility: 'private',
      scope,
      lastActivity: nowLabel(),
      isNew: true,
    };
    this.conversations = [conversation, ...this.conversations];
    this.threads.set(conversation.id, []);
    return { ...conversation };
  }

  async renameConversation(id: string, title: string): Promise<ConversationSummary> {
    await sleep(0);
    const trimmed = title.trim();
    return trimmed ? this.update(id, { title: trimmed.slice(0, 120) }) : { ...this.summary(id) };
  }

  async setVisibility(id: string, visibility: Visibility): Promise<ConversationSummary> {
    await sleep(0);
    const conversation = this.summary(id);
    // Partajarea unei conversații existente cere alegere explicită a mesajelor (C0/C2).
    if (!conversation.isNew || conversation.scope !== 'project' || this.thread(id).length > 0) {
      throw new Error('Vizibilitatea se alege doar pentru o conversație nouă de proiect.');
    }
    return this.update(id, { visibility });
  }

  async sendMessage(
    conversationId: string,
    text: string,
  ): Promise<{ message: PersonMessage; conversation: ConversationSummary }> {
    await sleep(0);
    const body = text.trim();
    if (!body) {
      throw new Error('Mesaj gol');
    }
    const message: PersonMessage = {
      kind: 'person',
      id: this.nextId('msg'),
      authorId: 'andrei',
      time: nowLabel(),
      text: body,
      attachments: [],
    };
    this.thread(conversationId).push(message);
    const current = this.summary(conversationId);
    const conversation = this.update(conversationId, {
      lastActivity: nowLabel(),
      isNew: false,
      title:
        current.isNew && current.title === t('sidebar.newConversationTitle')
          ? body.slice(0, 40)
          : current.title,
    });
    return { message: structuredClone(message), conversation };
  }

  async streamReply(
    request: ReplyRequest,
    signal: AbortSignal,
    onEvent: (event: ReplyEvent) => void,
  ): Promise<AssistantMessage> {
    if (this.status.provider.kind !== 'demo') {
      throw new ProviderMissingError();
    }
    const thread = this.thread(request.conversationId);
    const prompt = thread.find(
      (m): m is PersonMessage => m.kind === 'person' && m.id === request.promptMessageId,
    );
    if (!prompt) {
      throw new Error('Mesajul la care se răspunde nu există.');
    }
    const attempt =
      thread.filter((m) => m.kind === 'assistant' && m.replyTo === request.promptMessageId).length + 1;
    const reply = demoReplyFor(prompt.text);
    const full = plainText(reply);
    let shown = 0;

    const finish = (stopped: boolean): AssistantMessage => {
      const partial = full.slice(0, shown);
      const message: AssistantMessage = stopped
        ? {
            kind: 'assistant',
            id: this.nextId('msg'),
            time: nowLabel(),
            model: DEMO_MODEL_LABEL,
            tag: t('chat.stopped'),
            paragraphs: [partial ? `${partial}…` : t('chat.stoppedEmpty')],
            extras: [],
            replyTo: request.promptMessageId,
            attempt,
            stopped: true,
          }
        : {
            kind: 'assistant',
            id: this.nextId('msg'),
            time: nowLabel(),
            model: DEMO_MODEL_LABEL,
            paragraphs: reply.paragraphs,
            extras: reply.extras,
            replyTo: request.promptMessageId,
            attempt,
            stopped: false,
            ...(reply.list ? { list: reply.list } : {}),
            ...(reply.tag ? { tag: reply.tag } : {}),
          };
      thread.push(message);
      this.update(request.conversationId, { lastActivity: nowLabel() });
      return structuredClone(message);
    };

    try {
      onEvent({ type: 'thinking' });
      await sleep(this.timing.thinkingMs, signal);
      while (shown < full.length) {
        shown = Math.min(full.length, shown + this.timing.charsPerTick);
        onEvent({ type: 'streaming', text: full.slice(0, shown) });
        await sleep(this.timing.tickMs, signal);
      }
      return finish(false);
    } catch (error) {
      if (error instanceof DOMException && error.name === 'AbortError') {
        return finish(true);
      }
      throw error;
    }
  }

  async preparePlan(conversationId: string): Promise<AssistantMessage> {
    await sleep(0);
    const message: AssistantMessage = {
      kind: 'assistant',
      id: this.nextId('msg'),
      time: nowLabel(),
      model: DEMO_MODEL_LABEL,
      paragraphs: ['Am pregătit planul. Nimic nu se schimbă până nu îl aprobă proprietarul.'],
      extras: [
        {
          type: 'plan_card',
          title: 'Plan v3 · Cronometru pe server',
          summary:
            '2 pași · Constructor pe PC-ul meu · 2 teste · cost maxim 0,00 $ · date trimise extern: niciuna',
          approver: `${OWNER_NAME} (proprietar)`,
        },
      ],
      attempt: 1,
      stopped: false,
    };
    this.thread(conversationId).push(message);
    return structuredClone(message);
  }

  setProviderAvailable(available: boolean): StatusSnapshot {
    this.status = {
      ...this.status,
      provider: available
        ? { kind: 'demo', label: t('composer.model.demo') }
        : { kind: 'missing', label: t('composer.model.missing') },
    };
    return structuredClone(this.status);
  }

  setSlowLoading(slow: boolean): void {
    this.slowLoading = slow;
  }
}
