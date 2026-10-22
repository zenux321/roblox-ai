/**
 * Contractele interfeței cu nucleul. În această etapă sunt servite de
 * `MockWorkspaceGateway` (date demonstrative); în etapele următoare de
 * worker prin comenzi Tauri tipizate. React nu ține secrete.
 */

export type Visibility = 'private' | 'shared';
export type ConversationScope = 'project' | 'general';
export type Tone = 'success' | 'accent' | 'warning' | 'danger' | 'muted';

export interface Person {
  id: string;
  name: string;
  initial: string;
  /** Persoana care folosește aplicația pe acest PC. */
  isSelf: boolean;
  palette: 'self' | 'peer';
}

export interface ConversationSummary {
  id: string;
  title: string;
  visibility: Visibility;
  scope: ConversationScope;
  /** Etichetă de afișare a ultimei activități (de ex. „acum 2 min”). */
  lastActivity: string;
  /** Conversație nou creată, încă goală: vizibilitatea se mai poate alege. */
  isNew: boolean;
}

export type AttachmentKind = 'image' | 'video' | 'doc' | 'code';
/** Atașare nu înseamnă analiză (CHAT_AND_ASSETS). */
export type AnalysisState = 'analyzed' | 'attached_only';

export interface Attachment {
  id: string;
  kind: AttachmentKind;
  name: string;
  meta: string;
  analysis: AnalysisState;
  note?: string;
  alternative?: string;
  preview?: string;
}

export interface CodeSnippet {
  language: string;
  file: string;
  code: string;
}

export type AssistantExtra =
  | { type: 'prepare_plan' }
  | {
      type: 'plan_card';
      title: string;
      summary: string;
      approver: string;
      planId?: string;
      planVersion?: number;
    }
  | { type: 'image_generation_unconfigured' }
  | { type: 'associate_project' };

export interface PersonMessage {
  kind: 'person';
  id: string;
  authorId: string;
  time: string;
  text: string;
  attachments: Attachment[];
}

export interface AgentActivityMessage {
  kind: 'agent';
  id: string;
  agent: string;
  title: string;
  computer: string;
  details: string[];
}

export interface AssistantMessage {
  kind: 'assistant';
  id: string;
  time: string;
  model: string;
  tag?: string;
  paragraphs: string[];
  list?: string[];
  code?: CodeSnippet;
  extras: AssistantExtra[];
  /** Mesajul utilizatorului la care răspunde. */
  replyTo?: string;
  /** Numărul încercării; o reîncercare nu șterge încercările anterioare. */
  attempt: number;
  stopped: boolean;
}

export type Message = PersonMessage | AgentActivityMessage | AssistantMessage;

export interface ConnectionRow {
  label: string;
  value: string;
  state: 'connected' | 'not_configured' | 'demo';
}

export interface Finding {
  kind: 'info' | 'difference';
  title?: string;
  text: string;
}

export interface ProjectInfo {
  id: string;
  name: string;
  initials: string;
  subtitle: string;
  kindLabel: string;
  connections: ConnectionRow[];
  findings: Finding[];
}

export interface RecentProject {
  name: string;
  connections: string;
  current: boolean;
}

export interface AgentCard {
  id: string;
  name: string;
  state: string;
  tone: Tone;
  current: string;
  lastResult: string;
  computer: string;
  tools: string;
  output: string;
}

export interface TaskCard {
  title: string;
  who: string;
  assignee: 'self' | 'peer' | 'agent';
  state: string;
  tone: Tone;
  progress?: number;
  dependency?: string;
}

export interface MemberCard {
  name: string;
  role: string;
  palette: 'self' | 'peer';
  initial: string;
  online: boolean;
  status: string;
  permissions: string;
}

export interface ComputerCard {
  name: string;
  tone: Tone;
  acceptance: string;
  model: string;
  load: string;
  task: string;
}

export interface ContextItem {
  id: string;
  title: string;
  group: string;
  selected: boolean;
}

export type ProviderStatus = { kind: 'demo'; label: string } | { kind: 'missing'; label: string };

export interface StatusSnapshot {
  connection: { kind: 'solo' };
  studio: { state: 'not_configured' };
  cost: { spentUsd: number; limitUsd: number };
  provider: ProviderStatus;
  limits: { maxAgentSteps: number; maxRetries: number; taskTimeoutMinutes: number };
}

export interface WorkspaceSnapshot {
  project: ProjectInfo;
  recentProjects: RecentProject[];
  people: Person[];
  conversations: ConversationSummary[];
  agents: AgentCard[];
  tasks: TaskCard[];
  members: MemberCard[];
  computers: ComputerCard[];
  context: ContextItem[];
  status: StatusSnapshot;
}

export type ReplyEvent = { type: 'thinking' } | { type: 'streaming'; text: string };

// ---- Plan, aprobare, rulare (M1) -------------------------------------------

export type PlanStatus = 'draft' | 'awaiting_approval' | 'approved' | 'superseded';
export type RunStatus =
  | 'queued'
  | 'running'
  | 'paused'
  | 'waiting_dependency'
  | 'waiting_approval'
  | 'recovery_required'
  | 'succeeded'
  | 'failed'
  | 'cancelled';
export type TaskRunState = 'pending' | 'running' | 'succeeded' | 'failed' | 'recovery_required';

export interface PlanTaskView {
  id: string;
  number: number;
  title: string;
  description: string;
  agent: string;
  location: string;
  zone: string;
  after: string;
}

export interface PlanTestView {
  id: string;
  title: string;
  /** `static` = verificare automată acum; `studio` = cere Roblox Studio. */
  kind: 'static' | 'studio';
}

export interface ApprovalView {
  approvedBy: string;
  approvedAt: string;
  identityKind: string;
  planSha256: string;
  revokedAt?: string;
  valid: boolean;
}

export interface PlanView {
  id: string;
  version: number;
  status: PlanStatus;
  title: string;
  objective: string;
  sha256: string;
  estimatedDuration: string;
  costEstimateUsd: number;
  costLimitUsd: number;
  maxRetries: number;
  providerLabel: string;
  tasks: PlanTaskView[];
  tests: PlanTestView[];
  permissions: { label: string; value: string }[];
  approval?: ApprovalView;
  latestRunId?: string;
}

export interface RunView {
  id: string;
  planId: string;
  planVersion: number;
  status: RunStatus;
  tasks: { id: string; title: string; state: TaskRunState; attempt: number }[];
  done: number;
  total: number;
  activity: string;
  startedAt: string;
  updatedAt: string;
  lastSeq: number;
  live: boolean;
  hasResults: boolean;
}

export interface RunEventView {
  seq: number;
  at: string;
  kind: string;
  level: 'info' | 'ok' | 'warn' | 'error';
  message: string;
}

export interface TestResult {
  id: string;
  title: string;
  status: 'passed' | 'failed' | 'not_run';
  note: string;
}

export interface DiffLine {
  kind: 'add' | 'del' | 'ctx';
  number?: number;
  text: string;
}

export interface FileDiff {
  path: string;
  added: number;
  removed: number;
  isNew: boolean;
  lines: DiffLine[];
}

export interface RunResults {
  verdict: 'passed' | 'needs_review' | 'failed' | 'cancelled';
  tests: TestResult[];
  limitations: string[];
  durationSeconds: number;
  costUsd: number;
  reportPath: string | null;
  changes: FileDiff[];
}

/** Planul și rularea, comandate de UI. Implementate de worker (M2) / serviciul local (M1). */
export interface PlanGateway {
  currentPlan(): Promise<PlanView | null>;
  preparePlan(conversationId: string): Promise<{ message: AssistantMessage; plan: PlanView }>;
  approvePlan(planId: string, version: number, sha256: string): Promise<PlanView>;
  revokeApproval(planId: string, version: number): Promise<PlanView>;
  startRun(planId: string, version: number): Promise<RunView>;
  pauseRun(runId: string): Promise<RunView>;
  resumeRun(runId: string): Promise<RunView>;
  cancelRun(runId: string): Promise<RunView>;
  getRun(runId: string): Promise<RunView>;
  /** `SubscribeEvents(afterSequence)`: evenimentele de după `afterSeq`. */
  runEvents(runId: string, afterSeq: number): Promise<RunEventView[]>;
  runResults(runId: string): Promise<RunResults | null>;
  resolveRecovery(runId: string): Promise<RunView>;
  /** Notificare că o rulare are evenimente noi; întoarce funcția de dezabonare. */
  onRunEvent(listener: (runId: string, seq: number) => void): () => void;
}

export interface ReplyRequest {
  conversationId: string;
  /** Mesajul utilizatorului la care se răspunde. */
  promptMessageId: string;
}

/** Interfața prin care UI-ul vorbește cu nucleul. */
export interface WorkspaceGateway extends PlanGateway {
  readonly isDemo: boolean;
  loadWorkspace(): Promise<WorkspaceSnapshot>;
  loadMessages(conversationId: string): Promise<Message[]>;
  searchConversations(query: string): Promise<string[]>;
  createConversation(scope: ConversationScope): Promise<ConversationSummary>;
  renameConversation(id: string, title: string): Promise<ConversationSummary>;
  /** Permisă numai pentru o conversație nouă, încă goală. */
  setVisibility(id: string, visibility: Visibility): Promise<ConversationSummary>;
  sendMessage(
    conversationId: string,
    text: string,
  ): Promise<{ message: PersonMessage; conversation: ConversationSummary }>;
  /**
   * Generează răspunsul. La `signal.abort()` păstrează textul parțial ca
   * mesaj oprit; costurile/efectele deja produse nu se anulează retroactiv.
   */
  streamReply(
    request: ReplyRequest,
    signal: AbortSignal,
    onEvent: (event: ReplyEvent) => void,
  ): Promise<AssistantMessage>;
}

/** Comenzi folosite doar de meniul „Demo: stări” (previzualizare în browser). */
export interface DemoControls {
  setProviderAvailable(available: boolean): StatusSnapshot;
  setSlowLoading(slow: boolean): void;
  /** Rularea curentă devine „recuperare după repornire”; întoarce id-ul ei. */
  simulateRestart(): string | null;
}
