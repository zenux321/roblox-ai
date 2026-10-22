/** Reguli de răspuns ale furnizorului demonstrativ (fără model real). */
import type { AssistantExtra } from '../domain/types';
import { OWNER_NAME } from './demoData';

export interface DemoReply {
  paragraphs: string[];
  list?: string[];
  tag?: string;
  extras: AssistantExtra[];
}

export function demoReplyFor(prompt: string): DemoReply {
  const text = prompt.toLowerCase();
  if (/genereaz|thumbnail|imagin/.test(text)) {
    return {
      paragraphs: [
        'Pentru imagini am nevoie de OpenAI Images. Îți arăt costul și cine plătește înainte de orice generare.',
      ],
      extras: [{ type: 'image_generation_unconfigured' }],
    };
  }
  if (/(adaug|schimb|implement|modific|fă |fa )/.test(text)) {
    return {
      paragraphs: [
        `Asta ar modifica jocul, așa că nu o fac direct din conversație. Pregătesc un plan pe care îl poate aproba ${OWNER_NAME}.`,
      ],
      extras: [{ type: 'prepare_plan' }],
    };
  }
  if (text.includes('diferen')) {
    return {
      paragraphs: [
        'Am comparat jocul din Studio cu fișierele și cu notițele. Platforma6 lipsește din fișiere, iar ultimul salt are 12 studs, peste limita de 10 din notă. Vrei să păstrăm varianta din Studio sau pe cea din notă?',
      ],
      tag: 'Exemplu demonstrativ',
      extras: [],
    };
  }
  if (text.includes('checkpoint')) {
    return {
      paragraphs: [
        'Pe scurt: clientul rulează pe calculatorul jucătorului și poate fi modificat. Serverul e sursa de adevăr.',
      ],
      list: [
        'Clientul doar atinge piesa; evenimentul ajunge la server.',
        'Serverul verifică dacă piesa e un checkpoint real.',
        'Progresul se scrie numai pe server.',
      ],
      tag: 'Doar explicație · jocul nu se modifică',
      extras: [],
    };
  }
  return {
    paragraphs: [
      'Acesta este un răspuns demonstrativ. În aplicația reală răspunde modelul local ales, folosind numai contextul bifat din vault. Nimic din joc nu se schimbă din această conversație.',
    ],
    extras: [],
  };
}

/** Textul simplu al răspunsului, folosit pentru afișarea progresivă. */
export function plainText(reply: DemoReply): string {
  return [...reply.paragraphs, ...(reply.list ?? [])].join('\n');
}
