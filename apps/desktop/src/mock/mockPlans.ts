/**
 * Simularea în browser a planului și a rulării, cu aceeași semantică precum
 * serviciul Rust (crates/app): aprobare legată de hash, un singur job,
 * pauză/oprire/revocare, rezultate oneste. Folosită doar pentru preview și teste.
 */
import type {
  FileDiff,
  PlanView,
  RunEventView,
  RunResults,
  RunStatus,
  RunView,
  TaskRunState,
} from '../domain/types';

const COURSE = [
  '--!strict',
  '-- Traseul obby-ului: cinci platforme, lavă, checkpointuri și finish.',
  'local ReplicatedStorage = game:GetService("ReplicatedStorage")',
  'local Config = require(ReplicatedStorage.Shared.Config)',
  '',
  'for index = 1, Config.PlatformCount do',
  '\tlocal platform = makePart(`Platform{index}`)',
  'end',
];
const CHECKPOINT = [
  '--!strict',
  '-- Checkpoint, respawn și finish, validate numai pe server.',
  'respawnRequest.OnServerEvent:Connect(function(player, requested)',
  '\tif requested ~= nil and requested ~= lastCheckpoint[player] then',
  '\t\treturn -- cererea falsificată este ignorată',
  '\tend',
  '\trespawn(player)',
  'end)',
];
const REPORT = ['# Raport verificări statice', '', '| Test | Rezultat |', '|---|---|'];

const TESTS: { title: string; kind: 'static' | 'studio' }[] = [
  { title: 'Structura Rojo este validă', kind: 'static' },
  { title: 'Fără module externe încărcate după ID', kind: 'static' },
  { title: 'Fișierele modificate rămân în căile aprobate', kind: 'static' },
  { title: 'Checkpoint, respawn și finish testate în Studio', kind: 'studio' },
  { title: 'RemoteEvent falsificat respins de server (playtest)', kind: 'studio' },
];

function newFile(path: string, lines: string[]): FileDiff {
  return {
    path,
    added: lines.length,
    removed: 0,
    isNew: true,
    lines: lines.map((text, i) => ({ kind: 'add', number: i + 1, text })),
  };
}

async function sha256(text: string): Promise<string> {
  const bytes = new TextEncoder().encode(text);
  const digest = await crypto.subtle.digest('SHA-256', bytes);
  return [...new Uint8Array(digest)].map((b) => b.toString(16).padStart(2, '0')).join('');
}

const nowIso = () => new Date().toISOString();

interface Step {
  kind: string;
  level: RunEventView['level'];
  message: string;
  apply?: (run: RunView) => void;
}

interface RunRecord {
  view: RunView;
  events: RunEventView[];
  results: RunResults | null;
  steps: Step[];
  next: number;
  paused: boolean;
  timer: ReturnType<typeof setTimeout> | undefined;
  changes: FileDiff[];
  checks: number;
}

function setTask(run: RunView, id: string, state: TaskRunState) {
  run.tasks = run.tasks.map((t) => (t.id === id ? { ...t, state, attempt: Math.max(t.attempt, 1) } : t));
  run.done = run.tasks.filter((t) => t.state === 'succeeded').length;
}

export class MockPlanEngine {
  private plans: PlanView[] = [];
  private runs = new Map<string, RunRecord>();
  private listeners = new Set<(runId: string, seq: number) => void>();
  private seq = 0;
  private liveId: string | null = null;

  constructor(private readonly stepMs: number) {}

  current(): PlanView | null {
    const plan = [...this.plans].reverse().find((p) => p.status !== 'superseded');
    return plan ? structuredClone(plan) : null;
  }

  private find(id: string, version: number): PlanView {
    const plan = this.plans.find((p) => p.id === id && p.version === version);
    if (!plan) {
      throw new Error('Planul nu există.');
    }
    return plan;
  }

  private record(runId: string): RunRecord {
    const run = this.runs.get(runId);
    if (!run) {
      throw new Error('Rularea nu există.');
    }
    return run;
  }

  async create(): Promise<PlanView> {
    if (this.liveId) {
      throw new Error('O rulare este activă.');
    }
    const previous = this.current();
    this.plans = this.plans.map((p) => (p.status === 'superseded' ? p : { ...p, status: 'superseded' }));
    const version = (previous?.version ?? 0) + 1;
    const title = 'Obby de test: checkpoint și finish';
    const objective =
      'Creează un obby de test cu cinci platforme, checkpoint și finish, fără publicare. ' +
      'Lava trimite jucătorul la ultimul checkpoint, iar serverul validează progresul.';
    const plan: PlanView = {
      id: 'plan-obby-demo',
      version,
      status: 'awaiting_approval',
      title,
      objective,
      sha256: await sha256(`${version}:${title}:${objective}`),
      estimatedDuration: '~1 min',
      costEstimateUsd: 0,
      costLimitUsd: 0,
      maxRetries: 2,
      providerLabel: 'Furnizor demo (fals) · local, gratuit',
      tasks: [
        {
          id: 'generate',
          number: 1,
          title: 'Scrie traseul și checkpointul pe server',
          description: 'Cinci platforme, lavă, finish; progresul se scrie numai pe server',
          agent: 'Constructor',
          location: 'Fișiere locale',
          zone: 'src/server',
          after: '—',
        },
        {
          id: 'verify',
          number: 2,
          title: 'Verifică fișierele și scrie raportul',
          description: 'Verificări statice; playtestul din Studio rămâne neexecutat până la M4',
          agent: 'Verificator',
          location: 'Fișiere locale',
          zone: 'reports/static-checks.md',
          after: 'după 1',
        },
      ],
      tests: TESTS.map((t, i) => ({ id: `t${i + 1}`, ...t })),
      permissions: [
        { label: 'Fișiere', value: 'doar în src/, reports/' },
        { label: 'Roblox Studio', value: 'nu este folosit în acest plan' },
        { label: 'Model AI', value: 'Furnizor demo (fals) · local, gratuit' },
        { label: 'Date trimise pe internet', value: 'Niciuna: furnizorul demonstrativ rulează local' },
        { label: 'Git', value: 'fără commit și fără push' },
        { label: 'Publicare în Roblox', value: 'nu e inclusă' },
      ],
    };
    this.plans.push(plan);
    return structuredClone(plan);
  }

  approve(id: string, version: number, shown: string): PlanView {
    const plan = this.find(id, version);
    if (plan.status !== 'awaiting_approval') {
      throw new Error('Planul nu așteaptă aprobare.');
    }
    if (plan.sha256 !== shown) {
      throw new Error('Planul afișat diferă de cel salvat.');
    }
    plan.status = 'approved';
    plan.approval = {
      approvedBy: 'DEMO\\utilizator',
      approvedAt: nowIso(),
      identityKind: 'os_user_unauthenticated',
      planSha256: plan.sha256,
      valid: true,
    };
    return structuredClone(plan);
  }

  revoke(id: string, version: number): PlanView {
    const plan = this.find(id, version);
    if (!plan.approval || plan.approval.revokedAt) {
      throw new Error('Nu există o aprobare activă.');
    }
    plan.approval = { ...plan.approval, revokedAt: nowIso(), valid: false };
    plan.status = 'awaiting_approval';
    if (this.liveId && this.record(this.liveId).view.planVersion === version) {
      this.stop(this.liveId, 'Aprobarea a fost revocată; nu se mai produc efecte.');
    }
    return structuredClone(plan);
  }

  private emit(record: RunRecord, kind: string, level: RunEventView['level'], message: string) {
    this.seq += 1;
    const event: RunEventView = { seq: this.seq, at: nowIso(), kind, level, message };
    record.events.push(event);
    record.view.lastSeq = event.seq;
    record.view.activity = message;
    record.view.updatedAt = event.at;
    this.listeners.forEach((l) => l(record.view.id, event.seq));
  }

  private setStatus(record: RunRecord, status: RunStatus) {
    record.view.status = status;
  }

  start(id: string, version: number): RunView {
    if (this.liveId) {
      throw new Error('O altă rulare este activă pe acest PC.');
    }
    const plan = this.find(id, version);
    if (plan.status !== 'approved' || !plan.approval?.valid) {
      throw new Error('Planul nu este aprobat.');
    }
    const runId = `run-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 6)}`;
    const view: RunView = {
      id: runId,
      planId: id,
      planVersion: version,
      status: 'queued',
      tasks: plan.tasks.map((t) => ({ id: t.id, title: t.title, state: 'pending', attempt: 0 })),
      done: 0,
      total: plan.tasks.length,
      activity: '',
      startedAt: nowIso(),
      updatedAt: nowIso(),
      lastSeq: 0,
      live: true,
      hasResults: false,
    };
    const record: RunRecord = {
      view,
      events: [],
      results: null,
      paused: false,
      timer: undefined,
      next: 0,
      changes: [],
      checks: 0,
      steps: [
        { kind: 'running', level: 'info', message: 'Rulare pornită', apply: (r) => (r.status = 'running') },
        {
          kind: 'task_started',
          level: 'info',
          message: 'Pornește: Scrie traseul și checkpointul pe server (încercarea 1)',
          apply: (r) => setTask(r, 'generate', 'running'),
        },
        {
          kind: 'proposal',
          level: 'info',
          message: 'Constructorul a propus 2 fișiere (furnizor demonstrativ, cost 0,00 $)',
        },
        {
          kind: 'file_written',
          level: 'ok',
          message: `Scris src/server/Course.server.luau · ${COURSE.length} linii`,
          apply: () => record.changes.push(newFile('src/server/Course.server.luau', COURSE)),
        },
        {
          kind: 'file_written',
          level: 'ok',
          message: `Scris src/server/CheckpointService.server.luau · ${CHECKPOINT.length} linii`,
          apply: () => record.changes.push(newFile('src/server/CheckpointService.server.luau', CHECKPOINT)),
        },
        {
          kind: 'task_succeeded',
          level: 'ok',
          message: 'Gata: Scrie traseul și checkpointul pe server',
          apply: (r) => setTask(r, 'generate', 'succeeded'),
        },
        {
          kind: 'task_started',
          level: 'info',
          message: 'Pornește: Verifică fișierele și scrie raportul (încercarea 1)',
          apply: (r) => setTask(r, 'verify', 'running'),
        },
        ...TESTS.filter((t) => t.kind === 'static').map((t): Step => ({
          kind: 'check',
          level: 'ok',
          message: `Verificare „${t.title}”: trecută`,
          apply: () => (record.checks += 1),
        })),
        {
          kind: 'file_written',
          level: 'ok',
          message: 'Scris reports/static-checks.md',
          apply: () => record.changes.push(newFile('reports/static-checks.md', REPORT)),
        },
        {
          kind: 'task_succeeded',
          level: 'ok',
          message: 'Gata: Verifică fișierele și scrie raportul',
          apply: (r) => setTask(r, 'verify', 'succeeded'),
        },
      ],
    };
    this.runs.set(runId, record);
    this.liveId = runId;
    plan.latestRunId = runId;
    this.emit(record, 'queued', 'info', 'Rulare creată după aprobarea planului');
    this.schedule(record);
    return structuredClone(view);
  }

  private schedule(record: RunRecord) {
    record.timer = setTimeout(() => this.advance(record), this.stepMs);
  }

  private advance(record: RunRecord) {
    if (record.paused) {
      if (record.view.status !== 'paused') {
        this.setStatus(record, 'paused');
        this.emit(record, 'paused', 'info', 'În pauză. Niciun pas nou nu pornește.');
      }
      return;
    }
    const step = record.steps[record.next];
    if (!step) {
      this.setStatus(record, 'succeeded');
      this.finish(record);
      return;
    }
    record.next += 1;
    step.apply?.(record.view);
    this.emit(record, step.kind, step.level, step.message);
    this.schedule(record);
  }

  private finish(record: RunRecord) {
    const cancelled = record.view.status === 'cancelled';
    const tests = TESTS.map((t, i) => {
      if (t.kind === 'studio') {
        return {
          id: `t${i + 1}`,
          title: t.title,
          status: 'not_run' as const,
          note: 'Necesită Roblox Studio; playtestul vine în etapa M4.',
        };
      }
      const ran = record.checks > i;
      return {
        id: `t${i + 1}`,
        title: t.title,
        status: ran ? ('passed' as const) : ('not_run' as const),
        note: ran ? 'Verificare statică trecută' : 'Verificarea nu a rulat: rularea s-a oprit înainte.',
      };
    });
    record.results = {
      verdict: cancelled ? 'cancelled' : 'needs_review',
      tests,
      limitations: [
        'Playtestul în Roblox Studio nu a fost executat: integrarea Studio vine în etapa M4.',
        'Codul a fost propus de un furnizor demonstrativ (răspuns prestabilit), nu de un model real.',
        'Aprobarea folosește numele utilizatorului de sistem, fără autentificare; legarea de contul Windows vine în M2.',
      ],
      durationSeconds: Math.max(1, Math.round((Date.now() - Date.parse(record.view.startedAt)) / 1000)),
      costUsd: 0,
      reportPath: record.changes.some((c) => c.path === 'reports/static-checks.md')
        ? 'reports/static-checks.md'
        : null,
      changes: record.changes,
    };
    record.view.live = false;
    record.view.hasResults = true;
    if (this.liveId === record.view.id) {
      this.liveId = null;
    }
    this.emit(
      record,
      'finished',
      'warn',
      cancelled
        ? 'Rulare oprită; rezultatele parțiale sunt salvate'
        : 'Necesită verificare: unele teste nu au fost executate',
    );
  }

  private stop(runId: string, reason: string) {
    const record = this.record(runId);
    clearTimeout(record.timer);
    this.setStatus(record, 'cancelled');
    this.emit(record, 'cancelled', 'warn', reason);
    this.finish(record);
  }

  pause(runId: string): RunView {
    const record = this.record(runId);
    if (!record.view.live) {
      throw new Error('Rularea nu este activă.');
    }
    record.paused = true;
    return structuredClone(record.view);
  }

  resume(runId: string): RunView {
    const record = this.record(runId);
    if (record.view.status === 'recovery_required') {
      throw new Error('Rularea cere recuperare.');
    }
    const wasPaused = record.view.status === 'paused';
    record.paused = false;
    if (wasPaused) {
      this.setStatus(record, 'running');
      this.emit(record, 'running', 'info', 'Rulare reluată');
      this.schedule(record);
    }
    return structuredClone(record.view);
  }

  cancel(runId: string): RunView {
    const record = this.record(runId);
    if (!['succeeded', 'failed', 'cancelled'].includes(record.view.status)) {
      this.stop(runId, 'Rulare oprită de utilizator. Niciun pas nou nu mai pornește.');
    }
    return structuredClone(record.view);
  }

  get(runId: string): RunView {
    return structuredClone(this.record(runId).view);
  }

  events(runId: string, after: number): RunEventView[] {
    return structuredClone(this.record(runId).events.filter((e) => e.seq > after));
  }

  results(runId: string): RunResults | null {
    return structuredClone(this.record(runId).results);
  }

  /** Scenariu demo: rularea pare întreruptă de o repornire în timpul primului pas. */
  simulateRestart(): string | null {
    const plan = this.current();
    if (!plan?.latestRunId) {
      return null;
    }
    const record = this.record(plan.latestRunId);
    clearTimeout(record.timer);
    record.view.live = false;
    record.view.status = 'recovery_required';
    record.view.tasks = record.view.tasks.map((t) =>
      t.state === 'running' || (t.id === 'generate' && t.state !== 'succeeded')
        ? { ...t, state: 'recovery_required', attempt: Math.max(1, t.attempt) }
        : t,
    );
    if (this.liveId === record.view.id) {
      this.liveId = null;
    }
    this.emit(
      record,
      'recovery',
      'warn',
      'Aplicația s-a închis în timpul unui pas. Verificăm efectul înainte de a continua, fără să-l repetăm orbește.',
    );
    return record.view.id;
  }

  resolveRecovery(runId: string): RunView {
    const record = this.record(runId);
    if (record.view.status !== 'recovery_required') {
      throw new Error('Rularea nu cere recuperare.');
    }
    record.view.tasks = record.view.tasks.map((t) =>
      t.state === 'recovery_required' ? { ...t, state: 'pending' } : t,
    );
    this.emit(record, 'recovery', 'info', 'Pasul „generate” nu s-a terminat; se reia în siguranță.');
    record.view.status = 'running';
    record.view.live = true;
    this.liveId = runId;
    record.next = Math.min(record.next, 1);
    record.changes = [];
    record.checks = 0;
    this.schedule(record);
    return structuredClone(record.view);
  }

  subscribe(listener: (runId: string, seq: number) => void): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }
}
