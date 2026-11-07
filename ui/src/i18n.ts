/**
 * Хук перевода интерфейса.
 *
 * `useT` подписывается на язык в сторе и возвращает функцию перевода,
 * поэтому компоненты перерисовываются при смене языка.
 */

import { useCallback } from 'react';

import { translate, type Language, type TranslationVars } from './dict';
import { useStore } from './store';

/** Возвращает функцию перевода для текущего языка интерфейса. */
export function useT(): (key: string, vars?: TranslationVars) => string {
  const language = useStore((state) => state.language);
  return useCallback(
    (key: string, vars?: TranslationVars) => translate(language, key, vars),
    [language],
  );
}

/** Текущий язык интерфейса. */
export function useLanguage(): Language {
  return useStore((state) => state.language);
}
