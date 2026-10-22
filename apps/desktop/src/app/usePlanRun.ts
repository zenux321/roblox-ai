import { useCallback, useEffect, useRef, useState } from 'react';

import type { PlanView, RunEventView, RunResults, RunView, WorkspaceGateway } from '../domain/types';

export interface PlanRunState {
  plan: PlanView | null;
  run: RunView | null;
  events: RunEventView[];
  results: RunResults | null;
  error: string | null;
  pending: boolean;
}

const INITIAL: PlanRunState = {
  plan: null,
  run: null,
  events: [],
  results: null,
  error: null,
  pending: false,
};

/** Planul curent, rularea lui, evenimentele (cu reluare după secvență) și rezultatele. */
export function usePlanRun(gateway: WorkspaceGateway) {
  const [state, setState] = useState<PlanRunState>(INITIAL);
  const runIdRef = useRef<string | null>(null);
  const lastSeqRef = useRef(0);

  const syncRun = useCallback(
    async (runId: string) => {
      const [run, events] = await Promise.all([
        gateway.getRun(runId),
        gateway.runEvents(runId, runIdRef.current === runId ? lastSeqRef.current : 0),
      ]);
      const fresh = runIdRef.current !== runId;
      runIdRef.current = runId;
      const maxSeq = events.reduce((m, e) => Math.max(m, e.seq), lastSeqRef.current);
      lastSeqRef.current = fresh ? events.reduce((m, e) => Math.max(m, e.seq), 0) : maxSeq;
      const results = run.hasResults ? await gateway.runResults(runId) : null;
      setState((s) => {
        const known = fresh ? [] : s.events;
        const seen = new Set(known.map((e) => e.seq));
        return {
          ...s,
          run,
          results,
          events: [...known, ...events.filter((e) => !seen.has(e.seq))].sort((a, b) => a.seq - b.seq),
        };
      });
    },
    [gateway],
  );

  const refresh = useCallback(async () => {
    const plan = await gateway.currentPlan();
    setState((s) => ({ ...s, plan }));
    if (plan?.latestRunId) {
      await syncRun(plan.latestRunId);
    } else {
      runIdRef.current = null;
      lastSeqRef.current = 0;
      setState((s) => ({ ...s, run: null, events: [], results: null }));
    }
  }, [gateway, syncRun]);

  useEffect(() => {
    const timer = setTimeout(() => void refresh().catch(() => undefined), 0);
    const unsubscribe = gateway.onRunEvent((runId) => {
      if (runId === runIdRef.current) {
        void syncRun(runId)
          .then(async () => {
            const plan = await gateway.currentPlan();
            setState((s) => ({ ...s, plan }));
          })
          .catch(() => undefined);
      }
    });
    return () => {
      clearTimeout(timer);
      unsubscribe();
    };
  }, [gateway, refresh, syncRun]);

  const act = useCallback(async (action: () => Promise<unknown>) => {
    setState((s) => ({ ...s, pending: true, error: null }));
    try {
      await action();
    } catch (error) {
      setState((s) => ({ ...s, error: error instanceof Error ? error.message : String(error) }));
    } finally {
      setState((s) => ({ ...s, pending: false }));
    }
  }, []);

  const plan = state.plan;
  const run = state.run;

  return {
    state,
    refresh,
    /** Planul nou creat din chat devine planul curent. */
    adoptPlan: useCallback((next: PlanView) => {
      runIdRef.current = null;
      lastSeqRef.current = 0;
      setState((s) => ({ ...s, plan: next, run: null, events: [], results: null, error: null }));
    }, []),
    approve: () =>
      act(async () => {
        if (plan) {
          const next = await gateway.approvePlan(plan.id, plan.version, plan.sha256);
          setState((s) => ({ ...s, plan: next }));
        }
      }),
    revoke: () =>
      act(async () => {
        if (plan) {
          const next = await gateway.revokeApproval(plan.id, plan.version);
          setState((s) => ({ ...s, plan: next }));
        }
      }),
    start: () =>
      act(async () => {
        if (plan) {
          const started = await gateway.startRun(plan.id, plan.version);
          runIdRef.current = null;
          await syncRun(started.id);
          setState((s) => (s.plan ? { ...s, plan: { ...s.plan, latestRunId: started.id } } : s));
        }
      }),
    pause: () => act(async () => run && (await syncRunAfter(gateway.pauseRun(run.id)))),
    resume: () => act(async () => run && (await syncRunAfter(gateway.resumeRun(run.id)))),
    cancel: () => act(async () => run && (await syncRunAfter(gateway.cancelRun(run.id)))),
    recover: () => act(async () => run && (await syncRunAfter(gateway.resolveRecovery(run.id)))),
  };

  async function syncRunAfter(request: Promise<RunView>) {
    const next = await request;
    await syncRun(next.id);
  }
}

export type PlanRun = ReturnType<typeof usePlanRun>;
