import "@/editorWorker";
import * as monaco from "monaco-editor/esm/vs/editor/editor.api";
import { FunctionComponent, useRef, useState, useEffect } from "react";

import { Card } from "./ui/card";

type Props = {
  value: string;
  onChange?: (value: string) => void;
};

export const Editor: FunctionComponent<Props> = ({ value, onChange }) => {
  const [editor, setEditor] =
    useState<monaco.editor.IStandaloneCodeEditor | null>(null);
  const monacoEl = useRef(null);

  useEffect(() => {
    if (monacoEl) {
      setEditor((editor) => {
        if (editor) return editor;

        const newEditor = monaco.editor.create(monacoEl.current!, {
          value,
          language: "sql",
          minimap: {
            enabled: false,
          },
          fontSize: 20,
          padding: {
            top: 20,
            bottom: 20,
          },
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

  return (
    <Card className="p-1">
      <div className="w-full h-[200px]" ref={monacoEl}></div>
    </Card>
  );
};
