import { FunctionComponent, useEffect, useRef } from "react";

import type { IDisposable } from "monaco-editor";
import { vsPlusTheme } from "monaco-sql-languages";
import EditorComponent, { useMonaco } from "@monaco-editor/react";

import {
  COMMAND_CONFIG,
  ID_LANGUAGE_SQL,
  autoSuggestionCompletionItems,
} from "./editor.config";

import { fetchAutocomplete } from "@/api";
import { Card } from "@/components/ui/card";
import { useQuery } from "@tanstack/react-query";
import { useTheme } from "@/provider/theme.provider";
import { useSqlFormattingProviders } from "@/lib/monaco";

type Props = {
  value: string;
  onChange?: (value: string) => void;
};

export const Editor: FunctionComponent<Props> = ({ value, onChange }) => {
  const currentTheme = useTheme();
  const monacoInstance = useMonaco();
  const providerRef = useRef<IDisposable | null>(null);
  

  const { data: autoCompleteData } = useQuery({
    queryKey: ["autocomplete"],
    queryFn: () => fetchAutocomplete(),
  });

  // Configure Monaco
  useEffect(() => {
    if (!monacoInstance) return;

    monacoInstance.languages.register({ id: ID_LANGUAGE_SQL });
    monacoInstance.languages.setLanguageConfiguration(
      ID_LANGUAGE_SQL,
      COMMAND_CONFIG
    );

    monacoInstance.editor.defineTheme("sql-dark", vsPlusTheme.darkThemeData);
    monacoInstance.editor.defineTheme("sql-light", vsPlusTheme.lightThemeData);
  }, [monacoInstance]);

  // Register completion provider
  useEffect(() => {
    if (!monacoInstance || !autoCompleteData) return;

    providerRef.current?.dispose();

    providerRef.current =
      monacoInstance.languages.registerCompletionItemProvider(ID_LANGUAGE_SQL, {
        provideCompletionItems: (model, position) => {
          const word = model.getWordUntilPosition(position);
          const range = {
            startLineNumber: position.lineNumber,
            endLineNumber: position.lineNumber,
            startColumn: word.startColumn,
            endColumn: word.endColumn,
          };
          const { suggestions } = autoSuggestionCompletionItems(range);

          const tableColumnSuggestions = autoCompleteData.tables.flatMap(
            ({ table_name, columns }) => {
              const alias = table_name.substring(0, 3);

              const table = {
                label: table_name,
                kind: monacoInstance.languages.CompletionItemKind.Variable,
                insertText: table_name,
                range,
              };

              const aliasTable = {
                label: `${table_name} AS ${alias}`,
                kind: monacoInstance.languages.CompletionItemKind.Variable,
                insertText: `${table_name} AS ${alias}`,
                range,
              };

              const col = columns.map((column) => ({
                label: column,
                kind: monacoInstance.languages.CompletionItemKind.Variable,
                insertText: column,
                range,
              }));

              const tableColumn = columns.map((column) => ({
                label: `${table_name}.${column}`,
                kind: monacoInstance.languages.CompletionItemKind.Variable,
                insertText: `${table_name}.${column}`,
                range,
              }));

              const tableColumnAlias = columns.map((column) => ({
                label: `${alias}.${column}`,
                kind: monacoInstance.languages.CompletionItemKind.Variable,
                insertText: `${alias}.${column}`,
                range,
              }));

              return [
                table,
                aliasTable,
                ...col,
                ...tableColumn,
                ...tableColumnAlias,
              ];
            }
          );

          return { suggestions: [...suggestions, ...tableColumnSuggestions] };
        },
      });

    return () => {
      providerRef.current?.dispose();
      providerRef.current = null;
    };
  }, [monacoInstance, autoCompleteData]);

  // Register formatting providers (document and range)
  // Use centralized formatting providers
  // Moved into lib to keep this component lean
  useSqlFormattingProviders(monacoInstance, { languageId: ID_LANGUAGE_SQL });
  

  // Avoid rendering until theme is known
  if (!currentTheme) return null;

  return (
    <Card className="p-2">
      <EditorComponent
        height="200px"
        value={value}
        onChange={(val) => onChange?.(val ?? "")}
        language={ID_LANGUAGE_SQL}
        theme={currentTheme === "light" ? "sql-light" : "sql-dark"}
        options={{
          minimap: { enabled: false },
          fontSize: 20,
          fontFamily: "JetBrains Mono",
          padding: { top: 20, bottom: 20 },
          automaticLayout: true,
          formatOnPaste: true,
          readOnly: onChange === undefined,
        }}
      />
    </Card>
  );
};
