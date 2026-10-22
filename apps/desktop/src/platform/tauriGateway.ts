/**
 * Gateway-ul real: comenzi Tauri tipizate către serviciul Rust (crates/app).
 * Conversațiile, planurile, aprobările și rulările sunt persistente în SQLite.
 * Panourile Agenți/Echipă/Calculatoare rămân date demonstrative până la C1–C3.
 */
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

import type {
  AssistantMessage,
  ConversationScope,
  ConversationSummary,
  Message,
  PersonMessage,
  PlanView,
  ReplyEvent,
  ReplyRequest,
  RunEventView,
  RunResults,
  RunView,
  Visibility,
  WorkspaceGateway,
  WorkspaceSnapshot,
} from '../domain/types';
import { t } from '../i18n';
import { AGENTS, COMPUTERS, CONTEXT, MEMBERS, PEOPLE, PROJECT, TASKS } from '../mock/demoData';

interface WorkspaceInfo {
  projectId: string;
  projectName: string;
  projectRoot: string;
  dataDir: string;
  identity: string;
}

type ReplyNoticeEvent =
  ReplyEvent | { type: 'done'; message: AssistantMessage } | { type: 'error'; message: string };

interface ReplyNotice {
  requestId: string;
  event: ReplyNoticeEvent;
}

/** Erorile serviciului vin ca `{ code, message }`. */
function toError(error: unknown): Error {
  if (error && typeof error === 'object' && 'message' in error) {
    return new Error(String(error.message));
  }
  return new Error(String(error));
}

async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    throw toError(error);
  }
}

export class TauriWorkspaceGateway implements WorkspaceGateway {
  readonly isDemo = false;
  private readonly replyHandlers = new Map<string, (event: ReplyNoticeEvent) => void>();
  private replyListener: Promise<UnlistenFn> | null = null;

  private ensureReplyListener(): Promise<UnlistenFn> {
    this.replyListener ??= listen<ReplyNotice>('rab://reply', (event) => {
      this.replyHandlers.get(event.payload.requestId)?.(event.payload.event);
    });
    return this.replyListener;
  }

  async loadWorkspace(): Promise<WorkspaceSnapshot> {
    const [conversations, info] = await Promise.all([
      call<ConversationSummary[]>('list_conversations'),
      call<WorkspaceInfo>('workspace_info'),
    ]);
    return {
      project: {
        ...structuredClone(PROJECT),
        name: info.projectName,
        subtitle: 'Proiect demonstrativ local',
        kindLabel: 'Copie locală a proiectului demonstrativ · conexiunile reale vin în M3–M4',
        connections: [
          { label: 'Folder proiect', value: info.projectRoot, state: 'connected' },
          { label: 'Roblox Studio', value: 'Nicio instanță aleasă', state: 'not_configured' },
          { label: 'Vault · memorie comună', value: 'Neconfigurat', state: 'not_configured' },
          { label: 'GitHub', value: 'Neconectat', state: 'not_configured' },
        ],
      },
      recentProjects: [{ name: info.projectName, connections: 'Proiectul curent', current: true }],
      people: structuredClone(PEOPLE),
      conversations,
      agents: structuredClone(AGENTS),
      tasks: structuredClone(TASKS),
      members: structuredClone(MEMBERS),
      computers: structuredClone(COMPUTERS),
      context: structuredClone(CONTEXT),
      status: {
        connection: { kind: 'solo' },
        studio: { state: 'not_configured' },
        cost: { spentUsd: 0, limitUsd: 0 },
        provider: { kind: 'demo', label: t('composer.model.demo') },
        limits: { maxAgentSteps: 12, maxRetries: 2, taskTimeoutMinutes: 10 },
      },
    };
  }

  loadMessages(conversationId: string): Promise<Message[]> {
    return call('load_messages', { conversationId });
  }

  searchConversations(query: string): Promise<string[]> {
    return call('search_conversations', { query });
  }

  createConversation(scope: ConversationScope): Promise<ConversationSummary> {
    return call('create_conversation', { scope });
  }

  renameConversation(id: string, title: string): Promise<ConversationSummary> {
    return call('rename_conversation', { id, title });
  }

  setVisibility(id: string, visibility: Visibility): Promise<ConversationSummary> {
    return call('set_visibility', { id, visibility });
  }

  sendMessage(
    conversationId: string,
    text: string,
  ): Promise<{ message: PersonMessage; conversation: ConversationSummary }> {
    return call('send_message', { conversationId, text });
  }

  async streamReply(
    request: ReplyRequest,
    signal: AbortSignal,
    onEvent: (event: ReplyEvent) => void,
  ): Promise<AssistantMessage> {
    await this.ensureReplyListener();
    const requestId = crypto.randomUUID();
    return new Promise<AssistantMessage>((resolve, reject) => {
      const cleanup = () => this.replyHandlers.delete(requestId);
      this.replyHandlers.set(requestId, (event) => {
        if (event.type === 'done') {
          cleanup();
          resolve(event.message);
        } else if (event.type === 'error') {
          cleanup();
          reject(new Error(event.message));
        } else {
          onEvent(event);
        }
      });
      signal.addEventListener(
        'abort',
        () => void call('cancel_reply', { requestId }).catch(() => undefined),
        {
          once: true,
        },
      );
      call('start_reply', {
        conversationId: request.conversationId,
        promptMessageId: request.promptMessageId,
        requestId,
      }).catch((error: unknown) => {
        cleanup();
        reject(toError(error));
      });
    });
  }

  preparePlan(conversationId: string): Promise<{ message: AssistantMessage; plan: PlanView }> {
    return call('prepare_plan', { conversationId });
  }

  currentPlan(): Promise<PlanView | null> {
    return call('current_plan');
  }

  approvePlan(planId: string, version: number, sha256: string): Promise<PlanView> {
    return call('approve_plan', { planId, version, sha256 });
  }

  revokeApproval(planId: string, version: number): Promise<PlanView> {
    return call('revoke_approval', { planId, version });
  }

  startRun(planId: string, version: number): Promise<RunView> {
    return call('start_run', { planId, version });
  }

  pauseRun(runId: string): Promise<RunView> {
    return call('pause_run', { runId });
  }

  resumeRun(runId: string): Promise<RunView> {
    return call('resume_run', { runId });
  }

  cancelRun(runId: string): Promise<RunView> {
    return call('cancel_run', { runId });
  }

  getRun(runId: string): Promise<RunView> {
    return call('get_run', { runId });
  }

  runEvents(runId: string, afterSeq: number): Promise<RunEventView[]> {
    return call('run_events', { runId, afterSeq });
  }

  runResults(runId: string): Promise<RunResults | null> {
    return call('run_results', { runId });
  }

  resolveRecovery(runId: string): Promise<RunView> {
    return call('resolve_recovery', { runId });
  }

  onRunEvent(listener: (runId: string, seq: number) => void): () => void {
    const unlisten = listen<{ runId: string; seq: number }>('rab://run-event', (event) =>
      listener(event.payload.runId, event.payload.seq),
    );
    return () => void unlisten.then((stop) => stop());
  }
}
