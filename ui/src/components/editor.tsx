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
import { useGetAllTables, useGetTable } from "./editor.hook";
import { Card } from "./ui/card";

type Props = {
  value: string;
  onChange?: (value: string) => void;
};

export const Editor: FunctionComponent<Props> = ({ value, onChange }) => {
  const currentTheme = useTheme();
  const [editor, setEditor] =
    useState<monaco.editor.IStandaloneCodeEditor | null>(null);
  const monacoEl = useRef<HTMLDivElement>(null);

  const { data: dataTable } = useGetAllTables();
  const { columns, handleSetTables } = useGetTable();

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
    if (!monacoEl.current) return;

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

        const tables = dataTable?.tables.map((table) => table.name) || [];
        const tableSuggestions = tables.map((table) => ({
          label: table,
          kind: monaco.languages.CompletionItemKind.Variable,
          insertText: `"${table}"`,
          range,
        }));

        console.log("tableSuggestions: ", tableSuggestions);

        // TODO: Implement column suggestions
        const columnSuggestions = columns.map((column) => ({
          label: column,
          kind: monaco.languages.CompletionItemKind.Variable,
          insertText: column,
          range,
        }));

        // TODO: Implement table column suggestions
        const tableColumnSuggestions = tables.flatMap((table) =>
          columns.map((column) => ({
            label: `${table}.${column}`,
            kind: monaco.languages.CompletionItemKind.Variable,
            insertText: `${table}.${column}`,
            range,
          }))
        );

        const allSuggestions = [
          ...suggestions,
          ...tableSuggestions,
          // ...columnSuggestions,
          // ...tableColumnSuggestions,
        ];

        console.log("allSuggestions: ", allSuggestions);

        return { suggestions: allSuggestions };
      },
    });
  }, [monacoEl.current, dataTable, columns]);

  useEffect(() => {
    if (monacoEl.current) {
      monaco.editor.setTheme(
        currentTheme === "light" ? "sql-light" : "sql-dark"
      );
    }
  }, [currentTheme]);

  useEffect(() => {
    if (!dataTable) return;

    const tables = dataTable.tables.map((table) => table.name);
    handleSetTables((prev) => {
      if (prev.length === 0) return prev;
      const isExist = prev.some((table) => tables.includes(table));
      if (isExist) return prev;
      return [...prev, ...tables];
    });
  }, [dataTable]);

  return (
    <Card className="p-2">
      <div className="w-full h-[200px]" ref={monacoEl} />
    </Card>
  );
};
