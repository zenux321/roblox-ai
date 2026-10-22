/**
 * Date demonstrative preluate din prototipul v5, reconciliate cu
 * DESIGN_HANDOFF: fără „Studio conectat”, fără conexiuni pretinse, model
 * simulat etichetat ca atare. Nimic de aici nu provine dintr-un proiect real.
 */
import type {
  AgentCard,
  ComputerCard,
  ContextItem,
  ConversationSummary,
  MemberCard,
  Message,
  Person,
  ProjectInfo,
  RecentProject,
  TaskCard,
} from '../domain/types';
import seed from './demo-seed.json';

export const DEMO_MODEL_LABEL = 'model simulat';
export const OWNER_NAME = 'Andrei';
export const PEER_NAME = 'Mihai';

export const PEOPLE: Person[] = [
  { id: 'andrei', name: 'Tu', initial: 'A', isSelf: true, palette: 'self' },
  { id: 'mihai', name: PEER_NAME, initial: 'M', isSelf: false, palette: 'peer' },
];

export const PROJECT: ProjectInfo = {
  id: 'obby-vulcanic',
  name: 'Obby Vulcanic',
  initials: 'OV',
  subtitle: 'Joc existent · proiect demonstrativ',
  kindLabel: 'Joc existent · exemplu demo, fără conexiuni reale',
  connections: [
    { label: 'Folder proiect', value: 'D:\\Roblox\\ObbyVulcanic', state: 'demo' },
    { label: 'Roblox Studio', value: 'Nicio instanță aleasă', state: 'not_configured' },
    { label: 'Vault · memorie comună', value: 'RobloxAIBuilder/Projects/Obby Vulcanic', state: 'demo' },
    { label: 'GitHub', value: 'Neconectat', state: 'not_configured' },
  ],
  findings: [
    { kind: 'info', text: '42 de scripturi, 6 platforme, 1 checkpoint · 9 note în memoria comună' },
    {
      kind: 'difference',
      title: 'Diferență: joc ↔ fișiere',
      text: 'Platforma6 există în Studio, dar nu și în fișierele proiectului.',
    },
    {
      kind: 'difference',
      title: 'Diferență: joc ↔ notițe',
      text: 'Nota „Dificultatea platformelor” spune maximum 10 studs; în joc ultimul salt are 12.',
    },
  ],
};

export const RECENT_PROJECTS: RecentProject[] = [
  { name: 'Obby Vulcanic', connections: 'Exemplu demo · proiectul curent', current: true },
  { name: 'Tycoon Pizzerie', connections: 'Exemplu demo · vault și GitHub', current: false },
  { name: 'Horror: Școala Veche', connections: 'Exemplu demo · folder lipsă, reconectează', current: false },
];

/**
 * Conversațiile demonstrative sunt într-un singur fișier JSON, folosit și de
 * nucleul Rust pentru datele inițiale ale bazei de date (crates/app).
 */
export const CONVERSATIONS: ConversationSummary[] = seed.conversations as ConversationSummary[];
export const THREADS: Record<string, Message[]> = seed.threads as Record<string, Message[]>;

export const AGENTS: AgentCard[] = [
  {
    id: 'planner',
    name: 'Planificator',
    state: 'Disponibil',
    tone: 'success',
    current: 'Nicio sarcină',
    lastResult: 'plan v2 pentru checkpoint',
    computer: 'PC-ul meu',
    tools: 'memory.search · project.read',
    output: 'propune un plan',
  },
  {
    id: 'researcher',
    name: 'Cercetător',
    state: 'Lucrează',
    tone: 'accent',
    current: 'Caută în sursele salvate exemple de imagini',
    lastResult: '2 ghiduri din Creator Hub',
    computer: 'PC-ul lui Mihai · llama3.2 3B',
    tools: 'memory.search · research.fetch (doar în plan)',
    output: 'note cu surse',
  },
  {
    id: 'builder',
    name: 'Constructor',
    state: 'Așteaptă',
    tone: 'warning',
    current: 'Așteaptă aprobarea planului v3',
    lastResult: 'patch pentru CheckpointService',
    computer: 'PC-ul meu · qwen2.5-coder 7B',
    tools: 'project.read · project.propose_patch',
    output: 'propune un patch',
  },
  {
    id: 'reviewer',
    name: 'Verificator',
    state: 'Oprit',
    tone: 'muted',
    current: 'Pornește după pasul de construcție',
    lastResult: '4 din 4 teste trecute (21 oct.)',
    computer: 'PC-ul meu',
    tools: 'project.read · tests.read_results',
    output: 'raport de verificare',
  },
  {
    id: 'memory_curator',
    name: 'Curator memorie',
    state: 'Eroare',
    tone: 'danger',
    current: 'Nu poate propune nota: ai editat-o în Obsidian între timp',
    lastResult: 'notă propusă în Runs/',
    computer: 'PC-ul meu',
    tools: 'memory.search · memory.propose_note',
    output: 'propune o notă',
  },
];

export const TASKS: TaskCard[] = [
  {
    title: 'Capturi pentru thumbnail',
    who: 'Mihai',
    assignee: 'peer',
    state: 'Gata',
    tone: 'success',
    progress: 100,
  },
  {
    title: 'Thumbnail în 2 variante',
    who: 'Generare imagini · PC-ul meu',
    assignee: 'agent',
    state: 'Blocat',
    tone: 'warning',
    dependency: 'Lipsește OpenAI Images · depinde de: Capturi',
  },
  {
    title: 'Cronometru pe server',
    who: 'Constructor · PC-ul meu',
    assignee: 'agent',
    state: 'După aprobare',
    tone: 'muted',
    dependency: 'Depinde de: aprobarea planului v3',
  },
  {
    title: 'Verifică traseul pe telefon',
    who: 'Mihai',
    assignee: 'peer',
    state: 'În lucru',
    tone: 'accent',
    progress: 40,
  },
];

export const MEMBERS: MemberCard[] = [
  {
    name: 'Andrei (tu)',
    role: 'Proprietar',
    palette: 'self',
    initial: 'A',
    online: true,
    status: 'Online · discută thumbnailul',
    permissions: 'Aprobă planuri, integrează modificări, gestionează membrii',
  },
  {
    name: 'Mihai',
    role: 'Colaborator',
    palette: 'peer',
    initial: 'M',
    online: true,
    status: 'Exemplu demo · colaborarea vine în etapa C1',
    permissions: 'Discută și propune; nu aprobă planuri și nu integrează',
  },
];

export const COMPUTERS: ComputerCard[] = [
  {
    name: 'PC-ul meu',
    tone: 'success',
    acceptance: 'Acceptă sarcini',
    model: 'qwen2.5-coder 7B',
    load: 'GPU 42% · RAM 9/16 GB',
    task: 'Așteaptă aprobarea planului v3',
  },
  {
    name: 'PC-ul lui Mihai',
    tone: 'success',
    acceptance: 'Ocupat',
    model: 'llama3.2 3B',
    load: 'GPU 71% · RAM 6/8 GB',
    task: 'Cercetare: imagini pentru obby',
  },
];

export const CONTEXT: ContextItem[] = [
  { id: 'c1', title: 'Decizie: fără DataStore în MVP', group: 'Memorie comună', selected: true },
  { id: 'c2', title: 'Dificultatea platformelor', group: 'Memorie comună', selected: true },
  { id: 'p1', title: 'Idei personale pentru nivelul 2', group: 'Note personale · doar tu', selected: false },
  { id: 'd1', title: 'design-obby.pdf', group: 'Document selectat', selected: false },
];
