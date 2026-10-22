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

export const CONVERSATIONS: ConversationSummary[] = [
  {
    id: 'thumb',
    title: 'Thumbnail pentru joc',
    visibility: 'shared',
    scope: 'project',
    lastActivity: 'acum 2 min',
    isNew: false,
  },
  {
    id: 'checkpoint',
    title: 'De ce checkpointul e pe server',
    visibility: 'shared',
    scope: 'project',
    lastActivity: 'ieri',
    isNew: false,
  },
  {
    id: 'ideas',
    title: 'Idei pentru nivelul 2',
    visibility: 'private',
    scope: 'project',
    lastActivity: '20 oct.',
    isNew: false,
  },
  {
    id: 'general',
    title: 'Ce joc e ușor pentru început?',
    visibility: 'private',
    scope: 'general',
    lastActivity: '18 oct.',
    isNew: false,
  },
];

const CHECKPOINT_CODE = [
  'part.Touched:Connect(function(hit)',
  '    local player = Players:GetPlayerFromCharacter(hit.Parent)',
  '    if player and isCheckpoint(part) then',
  '        lastCheckpoint[player] = part  -- doar serverul scrie',
  '    end',
  'end)',
].join('\n');

function assistant(
  id: string,
  time: string,
  paragraphs: string[],
  rest: Partial<Omit<Extract<Message, { kind: 'assistant' }>, 'kind' | 'id' | 'time' | 'paragraphs'>> = {},
): Message {
  return {
    kind: 'assistant',
    id,
    time,
    model: DEMO_MODEL_LABEL,
    paragraphs,
    extras: [],
    attempt: 1,
    stopped: false,
    ...rest,
  };
}

export const THREADS: Record<string, Message[]> = {
  thumb: [
    {
      kind: 'person',
      id: 'thumb-1',
      authorId: 'mihai',
      time: '13:40',
      text: 'Am făcut două capturi din Studio cu traseul de lavă și un clip de la playtest.',
      attachments: [
        {
          id: 'a1',
          kind: 'image',
          name: 'captura-lava.png',
          meta: '1920×1080 · 1,8 MB',
          analysis: 'analyzed',
        },
        {
          id: 'a2',
          kind: 'image',
          name: 'captura-finish.png',
          meta: '1920×1080 · 1,6 MB',
          analysis: 'analyzed',
        },
        {
          id: 'a3',
          kind: 'video',
          name: 'playtest-traseu.mp4',
          meta: '0:42 · 24 MB',
          analysis: 'attached_only',
          note: 'Modelul local nu poate analiza clipuri video.',
          alternative: 'Extrage 3 cadre ca imagini',
        },
      ],
    },
    {
      kind: 'agent',
      id: 'thumb-2',
      agent: 'Cercetător',
      title: 'a găsit 2 ghiduri în sursele salvate',
      computer: 'PC-ul lui Mihai',
      details: [
        'Creator Hub · imagini de experiență: 16:9, fără text mic (salvată pe 20 oct.)',
        'Creator Hub · reguli pentru imaginile publicate (salvată pe 20 oct.)',
        'Fără acces nou la internet: cercetarea online cere un plan aprobat',
        'Durată 18 s · model local llama3.2 3B',
      ],
    },
    {
      kind: 'person',
      id: 'thumb-3',
      authorId: 'andrei',
      time: '13:44',
      text: 'Generează un thumbnail pentru jocul acesta, folosind capturile atașate. Stil luminos, lava portocalie, 2 variante.',
      attachments: [],
    },
    assistant(
      'thumb-4',
      '13:44',
      [
        'Pot pregăti generarea cu OpenAI Images, cu cele două capturi ca referință. Clipul nu îl pot folosi direct, dar pot extrage cadre din el.',
      ],
      { extras: [{ type: 'image_generation_unconfigured' }], replyTo: 'thumb-3' },
    ),
  ],
  checkpoint: [
    {
      kind: 'person',
      id: 'cp-1',
      authorId: 'andrei',
      time: 'ieri, 18:02',
      text: 'De ce verificăm checkpointul pe server și nu pe client?',
      attachments: [],
    },
    assistant(
      'cp-2',
      '18:02',
      [
        'Pe scurt: clientul rulează pe calculatorul jucătorului și poate fi modificat. Serverul e sursa de adevăr.',
      ],
      {
        tag: 'Doar explicație · jocul nu se modifică',
        list: [
          'Clientul doar atinge piesa; evenimentul ajunge la server.',
          'Serverul verifică dacă piesa e un checkpoint real.',
          'Progresul se scrie numai pe server.',
        ],
        code: { language: 'Luau', file: 'CheckpointService.server.luau', code: CHECKPOINT_CODE },
        replyTo: 'cp-1',
      },
    ),
    {
      kind: 'person',
      id: 'cp-3',
      authorId: 'mihai',
      time: '18:10',
      text: 'Deci și un cronometru ar trebui să fie tot pe server?',
      attachments: [],
    },
    assistant(
      'cp-4',
      '18:10',
      [
        'Da. Timpul oficial se măsoară pe server, iar clientul doar îl afișează. Asta ar modifica jocul, așa că îți pot pregăti un plan pentru aprobare.',
      ],
      { extras: [{ type: 'prepare_plan' }], replyTo: 'cp-3' },
    ),
  ],
  ideas: [
    {
      kind: 'person',
      id: 'id-1',
      authorId: 'andrei',
      time: '20 oct., 21:15',
      text: 'Idei pentru nivelul 2, ceva cu gheață?',
      attachments: [],
    },
    assistant('id-2', '21:15', ['Trei idei care se potrivesc cu nivelul 1:'], {
      tag: 'Conversație privată · Mihai nu o vede',
      list: [
        'Platforme de gheață care alunecă ușor în direcția săriturii.',
        'Stalactite care cad după 2 secunde de când stai sub ele.',
        'Un checkpoint încălzit, singurul loc unde nu aluneci.',
      ],
      replyTo: 'id-1',
    }),
  ],
  general: [
    {
      kind: 'person',
      id: 'g-1',
      authorId: 'andrei',
      time: '18 oct., 10:30',
      text: 'Ce tip de joc e cel mai ușor pentru început?',
      attachments: [],
    },
    assistant('g-2', '10:30', ['Un obby. Are reguli simple și se poate testa repede.'], {
      list: [
        'Spawn, câteva platforme și un final.',
        'Un singur script pe server pentru checkpoint.',
        'Se poate extinde treptat, nivel cu nivel.',
      ],
      extras: [{ type: 'associate_project' }],
      replyTo: 'g-1',
    }),
  ],
};

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
