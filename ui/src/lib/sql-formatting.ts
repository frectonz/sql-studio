import { useEffect, useRef } from "react";
import type { IDisposable } from "monaco-editor";
import type * as Monaco from "monaco-editor";
import { format as formatSql } from "sql-formatter";
import type { FormatOptionsWithLanguage } from "sql-formatter";
import { ID_LANGUAGE_SQL } from "@/components/editor.config";

const getDefaultSqlFormatOptions = (): FormatOptionsWithLanguage => ({
  language: "sqlite",
  keywordCase: "upper",
});

export function useSqlFormattingProviders(monaco: typeof Monaco | null) {
  const disposablesRef = useRef<IDisposable[]>([]);

  useEffect(() => {
    if (!monaco) return;

    disposablesRef.current.forEach((d) => d.dispose());
    disposablesRef.current = [];

    const getOpts = getDefaultSqlFormatOptions;

    const documentProvider =
      monaco.languages.registerDocumentFormattingEditProvider(ID_LANGUAGE_SQL, {
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
            console.error("SQL formatting error (document)", err);
            return [];
          }
        },
      });

    const rangeProvider =
      monaco.languages.registerDocumentRangeFormattingEditProvider(
        ID_LANGUAGE_SQL,
        {
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
              console.error("SQL formatting error (range)", err);
              return [];
            }
          },
        },
      );

    disposablesRef.current = [documentProvider, rangeProvider];

    return () => {
      disposablesRef.current.forEach((d) => d.dispose());
      disposablesRef.current = [];
    };
  }, [monaco]);
}
