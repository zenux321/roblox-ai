import { useCallback, useEffect, useRef, useState } from 'react';

import type {
  AssistantMessage,
  ConversationScope,
  ConversationSummary,
  Message,
  PlanView,
  StatusSnapshot,
  Visibility,
  WorkspaceGateway,
  WorkspaceSnapshot,
} from '../domain/types';

export interface ReplyProgress {
  conversationId: string;
  phase: 'thinking' | 'streaming';
  text: string;
}

export interface WorkspaceState {
  snapshot: WorkspaceSnapshot | null;
  loadError: boolean;
  activeId: string | null;
  threads: Record<string, Message[]>;
  loadingId: string | null;
  query: string;
  /** Rezultatele căutării; `null` când nu se caută nimic. */
  hits: string[] | null;
  reply: ReplyProgress | null;
}

const INITIAL: WorkspaceState = {
  snapshot: null,
  loadError: false,
  activeId: null,
  threads: {},
  loadingId: null,
  query: '',
  hits: null,
  reply: null,
};

function replaceConversation(
  snapshot: WorkspaceSnapshot | null,
  conversation: ConversationSummary,
): WorkspaceSnapshot | null {
  if (!snapshot) {
    return snapshot;
  }
  return {
    ...snapshot,
    conversations: snapshot.conversations.map((c) => (c.id === conversation.id ? conversation : c)),
  };
}

export function lastPersonMessageId(thread: Message[]): string | undefined {
  return [...thread].reverse().find((m) => m.kind === 'person')?.id;
}

export function useWorkspace(gateway: WorkspaceGateway) {
  const [state, setState] = useState<WorkspaceState>(INITIAL);
  const abortRef = useRef<AbortController | null>(null);
  const threadsRef = useRef<Record<string, Message[]>>({});
  const queryRef = useRef('');

  useEffect(() => {
    threadsRef.current = state.threads;
  }, [state.threads]);

  const appendMessage = useCallback((conversationId: string, message: Message) => {
    setState((s) => ({
      ...s,
      threads: { ...s.threads, [conversationId]: [...(s.threads[conversationId] ?? []), message] },
    }));
  }, []);

  const openConversation = useCallback(
    async (id: string) => {
      setState((s) => ({ ...s, activeId: id }));
      if (threadsRef.current[id]) {
        return;
      }
      setState((s) => ({ ...s, loadingId: id }));
      try {
        const messages = await gateway.loadMessages(id);
        setState((s) => ({
          ...s,
          threads: { ...s.threads, [id]: messages },
          loadingId: s.loadingId === id ? null : s.loadingId,
        }));
      } catch {
        setState((s) => ({ ...s, loadingId: null, loadError: true }));
      }
    },
    [gateway],
  );

  const load = useCallback(async () => {
    try {
      const snapshot = await gateway.loadWorkspace();
      setState((s) => ({ ...s, snapshot }));
      const first = snapshot.conversations[0];
      if (first) {
        await openConversation(first.id);
      }
    } catch {
      setState((s) => ({ ...s, loadError: true }));
    }
  }, [gateway, openConversation]);

  useEffect(() => {
    // Încărcarea pornește asincron; setState are loc numai după răspunsul gateway-ului.
    const timer = setTimeout(() => void load(), 0);
    return () => {
      clearTimeout(timer);
      abortRef.current?.abort();
    };
  }, [load]);

  /** Reîncarcă istoricul conversației active (folosit de meniul demo). */
  const reloadConversation = useCallback(
    async (id: string) => {
      setState((s) => {
        const threads = { ...s.threads };
        delete threads[id];
        return { ...s, threads };
      });
      delete threadsRef.current[id];
      await openConversation(id);
    },
    [openConversation],
  );

  const runReply = useCallback(
    async (conversationId: string, promptMessageId: string) => {
      const controller = new AbortController();
      abortRef.current = controller;
      setState((s) => ({ ...s, reply: { conversationId, phase: 'thinking', text: '' } }));
      try {
        const message: AssistantMessage = await gateway.streamReply(
          { conversationId, promptMessageId },
          controller.signal,
          (event) =>
            setState((s) => ({
              ...s,
              reply:
                event.type === 'thinking'
                  ? { conversationId, phase: 'thinking', text: '' }
                  : { conversationId, phase: 'streaming', text: event.text },
            })),
        );
        appendMessage(conversationId, message);
      } catch {
        // Furnizor lipsă sau eroare: nu se inventează un răspuns.
      } finally {
        if (abortRef.current === controller) {
          abortRef.current = null;
        }
        setState((s) => ({ ...s, reply: null }));
      }
    },
    [gateway, appendMessage],
  );

  const busy = state.reply !== null;
  const providerReady = state.snapshot?.status.provider.kind === 'demo';

  const send = useCallback(
    async (text: string): Promise<boolean> => {
      const id = state.activeId;
      if (!id || busy || !providerReady || !text.trim()) {
        return false;
      }
      const { message, conversation } = await gateway.sendMessage(id, text);
      appendMessage(id, message);
      setState((s) => ({ ...s, snapshot: replaceConversation(s.snapshot, conversation) }));
      void runReply(id, message.id);
      return true;
    },
    [state.activeId, busy, providerReady, gateway, appendMessage, runReply],
  );

  const stop = useCallback(() => {
    abortRef.current?.abort();
  }, []);

  /** Reîncercarea creează o încercare nouă; istoricul rămâne neatins. */
  const retry = useCallback(
    (message?: AssistantMessage) => {
      const id = state.activeId;
      if (!id || busy || !providerReady) {
        return;
      }
      const promptId = message?.replyTo ?? lastPersonMessageId(state.threads[id] ?? []);
      if (promptId) {
        void runReply(id, promptId);
      }
    },
    [state.activeId, state.threads, busy, providerReady, runReply],
  );

  const createConversation = useCallback(
    async (scope: ConversationScope) => {
      const conversation = await gateway.createConversation(scope);
      setState((s) => ({
        ...s,
        snapshot: s.snapshot
          ? { ...s.snapshot, conversations: [conversation, ...s.snapshot.conversations] }
          : s.snapshot,
        threads: { ...s.threads, [conversation.id]: [] },
        activeId: conversation.id,
        query: '',
        hits: null,
      }));
      queryRef.current = '';
    },
    [gateway],
  );

  const rename = useCallback(
    async (id: string, title: string) => {
      const conversation = await gateway.renameConversation(id, title);
      setState((s) => ({ ...s, snapshot: replaceConversation(s.snapshot, conversation) }));
    },
    [gateway],
  );

  const setVisibility = useCallback(
    async (id: string, visibility: Visibility) => {
      const conversation = await gateway.setVisibility(id, visibility);
      setState((s) => ({ ...s, snapshot: replaceConversation(s.snapshot, conversation) }));
    },
    [gateway],
  );

  const search = useCallback(
    async (query: string) => {
      queryRef.current = query;
      setState((s) => ({ ...s, query, hits: query.trim() ? s.hits : null }));
      if (!query.trim()) {
        return;
      }
      const hits = await gateway.searchConversations(query);
      if (queryRef.current === query) {
        setState((s) => ({ ...s, hits }));
      }
    },
    [gateway],
  );

  /** Creează planul (versiune nouă) și îl anunță în conversație. */
  const preparePlan = useCallback(
    async (conversationId: string): Promise<PlanView> => {
      const { message, plan } = await gateway.preparePlan(conversationId);
      appendMessage(conversationId, message);
      return plan;
    },
    [gateway, appendMessage],
  );

  const applyStatus = useCallback((status: StatusSnapshot) => {
    setState((s) => (s.snapshot ? { ...s, snapshot: { ...s.snapshot, status } } : s));
  }, []);

  return {
    state,
    busy,
    providerReady,
    retryLoad: () => {
      setState((s) => ({ ...s, loadError: false }));
      return load();
    },
    openConversation,
    reloadConversation,
    send,
    stop,
    retry,
    createConversation,
    rename,
    setVisibility,
    search,
    preparePlan,
    applyStatus,
  };
}

export type Workspace = ReturnType<typeof useWorkspace>;
