import ro from './ro.json';

type Leaf<T, P extends string = ''> = {
  [K in keyof T & string]: T[K] extends string
    ? `${P}${K}`
    : T[K] extends readonly unknown[]
      ? never
      : Leaf<T[K], `${P}${K}.`>;
}[keyof T & string];

export type MessageKey = Leaf<typeof ro>;

function lookup(key: string): unknown {
  return key.split('.').reduce<unknown>((node, part) => {
    if (node && typeof node === 'object' && part in node) {
      return (node as Record<string, unknown>)[part];
    }
    return undefined;
  }, ro);
}

/** Text din ro.json, cu variabile `{nume}` înlocuite. */
export function t(key: MessageKey, vars: Record<string, string | number> = {}): string {
  const value = lookup(key);
  const text = typeof value === 'string' ? value : key;
  return text.replace(/\{(\w+)\}/g, (match, name: string) => (name in vars ? String(vars[name]) : match));
}

export const chatSuggestions: readonly string[] = ro.chat.suggestions;

/** Formatare USD în stil românesc: „0,16 $”. */
export function formatUsd(value: number): string {
  return `${(Math.round(value * 100) / 100).toFixed(2).replace('.', ',')} $`;
}

/** Verdictul unei rulări: [titlu, explicație]. */
export function verdictText(verdict: keyof typeof ro.results.verdicts): readonly [string, string] {
  const [title = '', text = ''] = ro.results.verdicts[verdict];
  return [title, text];
}

const MONTHS = [
  'ian.',
  'feb.',
  'mar.',
  'apr.',
  'mai',
  'iun.',
  'iul.',
  'aug.',
  'sept.',
  'oct.',
  'nov.',
  'dec.',
];
const ISO = /^\d{4}-\d{2}-\d{2}T/;

function pad(n: number): string {
  return String(n).padStart(2, '0');
}

export function formatTime(iso: string): string {
  const d = new Date(iso);
  return Number.isNaN(d.getTime()) ? iso : `${pad(d.getHours())}:${pad(d.getMinutes())}`;
}

export function formatDateTime(iso: string): string {
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) {
    return iso;
  }
  return `${d.getDate()} ${MONTHS[d.getMonth()]} ${d.getFullYear()}, ${formatTime(iso)}`;
}

/**
 * Etichetă scurtă pentru momentul unui mesaj sau al unei conversații. Valorile
 * care nu sunt ISO (datele demo, „acum 2 min”) rămân neschimbate.
 */
export function formatWhen(value: string, now: Date = new Date()): string {
  if (!ISO.test(value)) {
    return value;
  }
  const d = new Date(value);
  if (Number.isNaN(d.getTime())) {
    return value;
  }
  const minutes = Math.round((now.getTime() - d.getTime()) / 60000);
  if (minutes < 1) {
    return 'acum';
  }
  const sameDay = d.toDateString() === now.toDateString();
  if (sameDay) {
    return formatTime(value);
  }
  const yesterday = new Date(now);
  yesterday.setDate(now.getDate() - 1);
  if (d.toDateString() === yesterday.toDateString()) {
    return `ieri, ${formatTime(value)}`;
  }
  return `${d.getDate()} ${MONTHS[d.getMonth()]}`;
}

export function formatDuration(seconds: number): string {
  if (seconds < 60) {
    return `${seconds} s`;
  }
  const m = Math.floor(seconds / 60);
  return `${m} min ${pad(seconds % 60)} s`;
}
