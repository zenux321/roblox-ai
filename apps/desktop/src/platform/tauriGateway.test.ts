import { emit } from '@tauri-apps/api/event';
import { clearMocks, mockIPC } from '@tauri-apps/api/mocks';
import { afterEach, describe, expect, it } from 'vitest';

import { TauriWorkspaceGateway } from './tauriGateway';

afterEach(() => clearMocks());

describe('TauriWorkspaceGateway (IPC simulat)', () => {
  it('trimite argumentele în camelCase așteptat de comenzile Rust', async () => {
    const calls: [string, unknown][] = [];
    mockIPC((cmd, args) => {
      calls.push([cmd, args]);
      if (cmd === 'approve_plan') {
        return { id: 'plan-obby-demo', version: 1, status: 'approved' };
      }
      return [];
    });
    const gw = new TauriWorkspaceGateway();
    await gw.loadMessages('c1');
    await gw.approvePlan('plan-obby-demo', 1, 'abc');
    await gw.runEvents('run-1', 7);
    expect(calls).toEqual([
      ['load_messages', { conversationId: 'c1' }],
      ['approve_plan', { planId: 'plan-obby-demo', version: 1, sha256: 'abc' }],
      ['run_events', { runId: 'run-1', afterSeq: 7 }],
    ]);
  });

  it('transformă erorile serviciului `{ code, message }` în Error', async () => {
    mockIPC(() => {
      // Tauri respinge promisiunea cu obiectul serializat al erorii Rust, nu cu un Error.
      // eslint-disable-next-line @typescript-eslint/only-throw-error
      throw { code: 'denied', message: 'planul afișat diferă de cel salvat' };
    });
    const gw = new TauriWorkspaceGateway();
    await expect(gw.approvePlan('p', 1, 'x')).rejects.toThrow('planul afișat diferă de cel salvat');
  });

  it('răspunsul vine prin evenimente legate de requestId; oprirea cere cancel_reply', async () => {
    const calls: [string, Record<string, unknown>][] = [];
    mockIPC(
      (cmd, args) => {
        calls.push([cmd, (args ?? {}) as Record<string, unknown>]);
        if (cmd === 'start_reply') {
          const requestId = (args as { requestId: string }).requestId;
          setTimeout(() => {
            void emit('rab://reply', { requestId: 'altul', event: { type: 'streaming', text: 'NU' } });
            void emit('rab://reply', { requestId, event: { type: 'streaming', text: 'Bun' } });
            void emit('rab://reply', {
              requestId,
              event: {
                type: 'done',
                message: {
                  kind: 'assistant',
                  id: 'm2',
                  time: 'acum',
                  model: 'model simulat',
                  paragraphs: ['Bun'],
                  extras: [],
                  attempt: 1,
                  stopped: false,
                },
              },
            });
          }, 5);
          return requestId;
        }
        return true;
      },
      { shouldMockEvents: true },
    );
    const gw = new TauriWorkspaceGateway();
    const seen: string[] = [];
    const controller = new AbortController();
    const message = await gw.streamReply(
      { conversationId: 'c1', promptMessageId: 'm1' },
      controller.signal,
      (e) => {
        if (e.type === 'streaming') {
          seen.push(e.text);
        }
      },
    );
    expect(message.id).toBe('m2');
    expect(seen).toEqual(['Bun']);
    controller.abort();
    await new Promise((r) => setTimeout(r, 0));
    const start = calls.find(([cmd]) => cmd === 'start_reply')?.[1];
    expect(start).toMatchObject({ conversationId: 'c1', promptMessageId: 'm1' });
    expect(calls.find(([cmd]) => cmd === 'cancel_reply')?.[1]).toEqual({ requestId: start?.requestId });
  });

  it('notificările de rulare ajung la abonați', async () => {
    mockIPC(() => undefined, { shouldMockEvents: true });
    const gw = new TauriWorkspaceGateway();
    const got: [string, number][] = [];
    const stop = gw.onRunEvent((runId, seq) => got.push([runId, seq]));
    await new Promise((r) => setTimeout(r, 0));
    await emit('rab://run-event', { runId: 'run-1', seq: 4 });
    expect(got).toEqual([['run-1', 4]]);
    stop();
  });
});
