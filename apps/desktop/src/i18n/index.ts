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
