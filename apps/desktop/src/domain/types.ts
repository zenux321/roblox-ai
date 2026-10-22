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
  | { type: 'plan_card'; title: string; summary: string; approver: string }
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

export interface ReplyRequest {
  conversationId: string;
  /** Mesajul utilizatorului la care se răspunde. */
  promptMessageId: string;
}

/** Interfața prin care UI-ul vorbește cu nucleul. */
export interface WorkspaceGateway {
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
  preparePlan(conversationId: string): Promise<AssistantMessage>;
}

/** Comenzi folosite doar de meniul „Demo: stări”; nu există în gateway-ul real. */
export interface DemoControls {
  setProviderAvailable(available: boolean): StatusSnapshot;
  setSlowLoading(slow: boolean): void;
}
