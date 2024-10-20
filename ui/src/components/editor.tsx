import "@/editorWorker";
import * as monaco from "monaco-editor/esm/vs/editor/editor.api";
import { vsPlusTheme } from "monaco-sql-languages";
import { FunctionComponent, useEffect, useRef, useState } from "react";

import { useTheme } from "@/provider/theme.provider";
import {
  COMMAND_CONFIG,
  ID_LANGUAGE_SQL,
  autoSuggestionCompletionItems,
} from "./editor.config";
import { Card } from "./ui/card";
import { fetchAutocomplete } from "@/api";
import { useQuery } from "@tanstack/react-query";

type Props = {
  value: string;
  onChange?: (value: string) => void;
};

export const Editor: FunctionComponent<Props> = ({ value, onChange }) => {
  const currentTheme = useTheme();
  const [editor, setEditor] =
    useState<monaco.editor.IStandaloneCodeEditor | null>(null);
  const monacoEl = useRef<HTMLDivElement>(null);

  const { data: autoCompleteData } = useQuery({
    queryKey: ["autocomplete"],
    queryFn: () => fetchAutocomplete(),
  });

  useEffect(() => {
    if (monacoEl) {
      setEditor((editor) => {
        if (editor) return editor;

        monaco.languages.register({ id: ID_LANGUAGE_SQL });
        monaco.languages.setLanguageConfiguration(
          ID_LANGUAGE_SQL,
          COMMAND_CONFIG
        );

        monaco.editor.defineTheme("sql-dark", vsPlusTheme.darkThemeData);
        monaco.editor.defineTheme("sql-light", vsPlusTheme.lightThemeData);

        const newEditor = monaco.editor.create(monacoEl.current!, {
          value,
          language: ID_LANGUAGE_SQL,
          minimap: {
            enabled: false,
          },
          fontSize: 20,
          padding: {
            top: 20,
            bottom: 20,
          },
          fontFamily: "JetBrains Mono",
          automaticLayout: true,
          readOnly: onChange === undefined,
        });

        newEditor.onDidChangeModelContent((_) => {
          onChange?.(newEditor.getValue());
        });

        return newEditor;
      });
    }

    return () => editor?.dispose();
  }, [monacoEl.current]);

  useEffect(() => {

      if(!autoCompleteData) return

    monaco.languages.registerCompletionItemProvider(ID_LANGUAGE_SQL, {
      provideCompletionItems: (model, position) => {
        const word = model.getWordUntilPosition(position);
        const range = {
          startLineNumber: position.lineNumber,
          endLineNumber: position.lineNumber,
          startColumn: word.startColumn,
          endColumn: word.endColumn,
        };
        const { suggestions } = autoSuggestionCompletionItems(range);

        const tableColumnSuggestions = autoCompleteData.tables.reduce((acc: any, { table_name, columns }) => {

          const alias = table_name.substring(0, 3);

          const table = {
            label: table_name,
            kind: monaco.languages.CompletionItemKind.Variable,
            insertText: table_name,
            range,
          }

          const aliasTable = {
            label: `${table_name} AS ${alias}`,
            kind: monaco.languages.CompletionItemKind.Variable,
            insertText: `${table_name} AS ${alias}`,
            range,
          }

          const col = columns.map((column) => ({
            label: column,
            kind: monaco.languages.CompletionItemKind.Variable,
            insertText: column,
            range,
          }));

          const tableColumn = columns.map((column) => ({
            label: `${table_name}.${column}`,
            kind: monaco.languages.CompletionItemKind.Variable,
            insertText: `${table_name}.${column}`,
            range,
          }));

          const tableColumnAlias = columns.map((column) => ({
            label: `${alias}.${column}`,
            kind: monaco.languages.CompletionItemKind.Variable,
            insertText: `${alias}.${column}`,
          }));

          return [...acc, table, aliasTable, ...col, ...tableColumn, ...tableColumnAlias];
        }, []);

        return { suggestions: [...suggestions, ...tableColumnSuggestions] };
      },
    });
  }, [autoCompleteData]);

  useEffect(() => {
    if (monacoEl.current) {
      monaco.editor.setTheme(
        currentTheme === "light" ? "sql-light" : "sql-dark"
      );
    }
  }, [currentTheme]);

  return (
    <Card className="p-2">
      <div className="w-full h-[200px]" ref={monacoEl} />
    </Card>
  );
};
