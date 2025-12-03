import { useEffect, useRef } from "react";
import type { IDisposable } from "monaco-editor";
import type * as Monaco from "monaco-editor";
import { format as formatSql } from "sql-formatter";
import type { FormatOptionsWithLanguage } from "sql-formatter";

type UseSqlFormattingProvidersOptions = {
  languageId: string;
  getOptions?: () => FormatOptionsWithLanguage;
};

const getDefaultSqlFormatOptions = (): FormatOptionsWithLanguage => ({
  language: "sqlite",
  keywordCase: "upper",
});

export function useSqlFormattingProviders(
  monaco: typeof Monaco | null,
  { languageId, getOptions }: UseSqlFormattingProvidersOptions,
) {
  const disposablesRef = useRef<IDisposable[]>([]);

  useEffect(() => {
    if (!monaco) return;

    // Dispose any previous providers
    disposablesRef.current.forEach((d) => d.dispose());
    disposablesRef.current = [];

    const getOpts = getOptions ?? getDefaultSqlFormatOptions;

    const documentProvider =
      monaco.languages.registerDocumentFormattingEditProvider(languageId, {
        provideDocumentFormattingEdits: (model) => {
          const fullRange = model.getFullModelRange();
          const text = model.getValue();
          try {
            const formatted = formatSql(text, getOpts());
            return [
              {
                range: fullRange,
                text: formatted,
              },
            ];
          } catch (err) {
            // Keep UX stable if formatting fails
            // eslint-disable-next-line no-console
            console.error("SQL formatting error (document)", err);
            return [];
          }
        },
      });

    const rangeProvider =
      monaco.languages.registerDocumentRangeFormattingEditProvider(languageId, {
        provideDocumentRangeFormattingEdits: (model, range) => {
          const text = model.getValueInRange(range);
          try {
            const formatted = formatSql(text, getOpts());
            return [
              {
                range,
                text: formatted,
              },
            ];
          } catch (err) {
            // eslint-disable-next-line no-console
            console.error("SQL formatting error (range)", err);
            return [];
          }
        },
      });

    disposablesRef.current = [documentProvider, rangeProvider];

    return () => {
      disposablesRef.current.forEach((d) => d.dispose());
      disposablesRef.current = [];
    };
  }, [monaco, languageId, getOptions]);
}
