/**
 * Contractul cu serviciul Rust: exemplele JSON sunt generate de
 * `crates/app/tests/contract.rs`. Testul verifică exact cheile fiecărui tip
 * din `types.ts` (nicio cheie lipsă, nicio cheie în plus, camelCase).
 */
import { describe, expect, it } from 'vitest';

import samples from '../../../../tests/fixtures/contract/dto-samples.json';

type Json = Record<string, unknown>;

function keys(value: unknown, required: string[], optional: string[] = []) {
  expect(value).toBeTypeOf('object');
  const actual = Object.keys(value as Json).sort();
  for (const key of required) {
    expect(actual, `lipsește ${key}`).toContain(key);
  }
  for (const key of actual) {
    expect([...required, ...optional], `cheie neașteptată ${key}`).toContain(key);
  }
}

describe('contract DTO Rust ↔ TypeScript', () => {
  it('conversații', () => {
    for (const c of samples.conversations) {
      keys(c, ['id', 'title', 'visibility', 'scope', 'lastActivity', 'isNew']);
      expect(['private', 'shared']).toContain(c.visibility);
      expect(['project', 'general']).toContain(c.scope);
    }
  });

  it('mesaje: persoană, agent, asistent', () => {
    const kinds = new Set<string>();
    for (const raw of samples.messages as Json[]) {
      kinds.add(raw.kind as string);
      switch (raw.kind) {
        case 'person':
          keys(raw, ['kind', 'id', 'authorId', 'time', 'text', 'attachments']);
          for (const a of raw.attachments as Json[]) {
            keys(a, ['id', 'kind', 'name', 'meta', 'analysis'], ['note', 'alternative', 'preview']);
            expect(['analyzed', 'attached_only']).toContain(a.analysis);
          }
          break;
        case 'agent':
          keys(raw, ['kind', 'id', 'agent', 'title', 'computer', 'details']);
          break;
        case 'assistant':
          keys(
            raw,
            ['kind', 'id', 'time', 'model', 'paragraphs', 'extras', 'attempt', 'stopped'],
            ['tag', 'list', 'code', 'replyTo'],
          );
          for (const e of raw.extras as Json[]) {
            expect([
              'prepare_plan',
              'plan_card',
              'image_generation_unconfigured',
              'associate_project',
            ]).toContain(e.type);
          }
          break;
        default:
          throw new Error(`tip necunoscut ${String(raw.kind)}`);
      }
    }
    expect([...kinds].sort()).toEqual(['agent', 'assistant', 'person']);
  });

  it('plan aprobat', () => {
    const plan = samples.plan as Json;
    keys(
      plan,
      [
        'id',
        'version',
        'status',
        'title',
        'objective',
        'sha256',
        'estimatedDuration',
        'costEstimateUsd',
        'costLimitUsd',
        'maxRetries',
        'providerLabel',
        'tasks',
        'tests',
        'permissions',
      ],
      ['approval', 'latestRunId'],
    );
    expect(plan.status).toBe('approved');
    for (const t of plan.tasks as Json[]) {
      keys(t, ['id', 'number', 'title', 'description', 'agent', 'location', 'zone', 'after']);
    }
    for (const t of plan.tests as Json[]) {
      keys(t, ['id', 'title', 'kind']);
      expect(['static', 'studio']).toContain(t.kind);
    }
    keys(plan.approval, ['approvedBy', 'approvedAt', 'identityKind', 'planSha256', 'valid'], ['revokedAt']);
  });

  it('rulare, evenimente și rezultate', () => {
    keys(samples.run, [
      'id',
      'planId',
      'planVersion',
      'status',
      'tasks',
      'done',
      'total',
      'activity',
      'startedAt',
      'updatedAt',
      'lastSeq',
      'live',
      'hasResults',
    ]);
    for (const t of samples.run.tasks) {
      keys(t, ['id', 'title', 'state', 'attempt']);
    }
    for (const e of samples.events) {
      keys(e, ['seq', 'at', 'kind', 'level', 'message']);
      expect(['info', 'ok', 'warn', 'error']).toContain(e.level);
    }
    const results = samples.results as Json;
    keys(results, ['verdict', 'tests', 'limitations', 'durationSeconds', 'costUsd', 'reportPath', 'changes']);
    expect(results.verdict).toBe('needs_review');
    for (const t of results.tests as Json[]) {
      keys(t, ['id', 'title', 'status', 'note']);
    }
    for (const c of results.changes as Json[]) {
      keys(c, ['path', 'added', 'removed', 'isNew', 'lines']);
      for (const l of c.lines as Json[]) {
        keys(l, ['kind', 'text'], ['number']);
      }
    }
  });
});
