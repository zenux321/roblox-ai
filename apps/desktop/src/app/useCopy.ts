import { useCallback, useEffect, useRef, useState } from 'react';

import { t } from '../i18n';

/** Copiere în clipboard cu eticheta „Copiat” timp de ~2 secunde. */
export function useCopy() {
  const [state, setState] = useState<{ key: string; ok: boolean } | null>(null);
  const timer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);

  useEffect(() => () => clearTimeout(timer.current), []);

  const copy = useCallback(async (key: string, text: string) => {
    let ok = true;
    try {
      await navigator.clipboard.writeText(text);
    } catch {
      ok = false;
    }
    setState({ key, ok });
    clearTimeout(timer.current);
    timer.current = setTimeout(() => setState(null), 2000);
  }, []);

  const label = useCallback(
    (key: string) => {
      if (state?.key !== key) {
        return t('chat.copy');
      }
      return state.ok ? t('chat.copied') : t('chat.copyFailed');
    },
    [state],
  );

  return { copy, label };
}
