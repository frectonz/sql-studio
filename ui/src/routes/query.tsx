import "react-data-grid/lib/styles.css";
import { useRef, useState } from "react";

import { z } from "zod";
import { DataGrid } from "react-data-grid";
import { useQuery } from "@tanstack/react-query";
import { useDebounce } from "@uidotdev/usehooks";
import { Database, Maximize2, Play, ShieldX, Terminal } from "lucide-react";
import { createFileRoute } from "@tanstack/react-router";

import { cn } from "@/lib/utils";
import { fetchQuery } from "@/api";
import { useSql, useSqlDispatch } from "@/provider/sql.provider";

import {
  Card,
  CardTitle,
  CardHeader,
  CardDescription,
} from "@/components/ui/card";
import { Editor } from "@/components/editor";
import { Toggle } from "@/components/ui/toggle";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { useTheme } from "@/provider/theme.provider";

const DEFAULT_EDITOR_HEIGHT = 200;
const MIN_EDITOR_HEIGHT = 120;
const MAX_EDITOR_HEIGHT = Math.min(600, Math.floor(window.innerHeight * 0.6));

export const Route = createFileRoute("/query")({
  component: Query,
  validateSearch: z.object({ sql: z.string().optional() }),
});

function Query() {
  const currentTheme = useTheme();

  const codeState = useSql();
  const setCodeState = useSqlDispatch();
  const code = useDebounce(codeState, 100);

  const [autoExecute, setAutoExecute] = useState(true);
  const [editorHeight, setEditorHeight] = useState(DEFAULT_EDITOR_HEIGHT);
  const [editorMinimized, setEditorMinimized] = useState(false);

  const isResizingRef = useRef(false);
  const resizeStartYRef = useRef(0);
  const resizeStartHeightRef = useRef(DEFAULT_EDITOR_HEIGHT);

  const clampEditorHeight = (height: number) =>
    Math.min(Math.max(height, MIN_EDITOR_HEIGHT), MAX_EDITOR_HEIGHT);

  const { data, error, refetch } = useQuery({
    queryKey: ["query", code],
    queryFn: () => fetchQuery(code),
    enabled: autoExecute,
    retry: false,
  });

  const handleDividerPointerDown = (
    event: React.PointerEvent<HTMLDivElement>,
  ) => {
    event.preventDefault();
    event.currentTarget.setPointerCapture(event.pointerId);
    isResizingRef.current = true;
    resizeStartYRef.current = event.clientY;
    resizeStartHeightRef.current = editorHeight;
  };

  const handleDividerPointerMove = (
    event: React.PointerEvent<HTMLDivElement>,
  ) => {
    if (!isResizingRef.current) return;

    const nextHeight =
      resizeStartHeightRef.current + (event.clientY - resizeStartYRef.current);

    setEditorHeight(clampEditorHeight(nextHeight));
  };

  const handleDividerPointerUp = () => {
    isResizingRef.current = false;
  };

  const handleDividerKeyDown = (event: React.KeyboardEvent<HTMLDivElement>) => {
    if (event.key !== "ArrowUp" && event.key !== "ArrowDown") return;

    event.preventDefault();
    const delta = event.key === "ArrowDown" ? 10 : -10;
    setEditorHeight((height) => clampEditorHeight(height + delta));
  };

  const grid = !data ? (
    !autoExecute && code && error ? (
      <Card>
        <CardHeader className="flex items-center">
          <ShieldX className="mb-2 h-12 w-12 text-red-400" />
          <CardTitle className="text-red-400">Error</CardTitle>
          <CardDescription className="text-red-400">
            Query didn't execute successfully.
          </CardDescription>
        </CardHeader>
      </Card>
    ) : (
      <Skeleton className="w-full h-[300px]" />
    )
  ) : data.columns.length === 0 ? (
    <Card>
      <CardHeader className="flex items-center">
        <Database className="mb-4 h-12 w-12 text-muted-foreground" />
        <CardTitle>Query Executed</CardTitle>
        <CardDescription>Returned no data</CardDescription>
      </CardHeader>
    </Card>
  ) : (
    <Card className="p-2 overflow-auto">
      <DataGrid
        defaultColumnOptions={{ resizable: true }}
        columns={data.columns.map((col) => ({
          key: col,
          name: col,
          renderCell: ({ row }: { row: Record<string, unknown> }) => (
            <div style={{ whiteSpace: "pre-wrap" }}>
              {String(row[col] ?? "")}
            </div>
          ),
        }))}
        rows={data.rows.map((row) =>
          row.reduce((acc, curr, i) => {
            acc[data.columns[i]] = curr;
            return acc;
          }, {}),
        )}
        rowHeight={(row) => {
          const maxLines = data.columns.reduce((max, col) => {
            const val = String(row[col] ?? "");
            const lines = val.split(/\r?\n/).length;
            return Math.max(max, lines);
          }, 1);
          return Math.max(35, maxLines * 20 + 15);
        }}
        className={cn(currentTheme === "light" ? "rdg-light" : "rdg-dark")}
      />
    </Card>
  );

  return (
    <div className="grid gap-8">
      <div className="grid gap-4 grid-cols-1">
        {editorMinimized ? (
          <Button
            variant="outline"
            size="sm"
            className="w-fit"
            onClick={() => setEditorMinimized(false)}
          >
            <Maximize2 className="h-4 w-4" />
            Editor
          </Button>
        ) : (
          <Editor
            value={code}
            onChange={(val) => setCodeState({ type: "SET_SQL", data: val })}
            height={editorHeight}
          />
        )}

        {!editorMinimized && (
          <div
            role="separator"
            aria-orientation="horizontal"
            aria-label="Resize query editor"
            tabIndex={0}
            onPointerDown={handleDividerPointerDown}
            onPointerMove={handleDividerPointerMove}
            onPointerUp={handleDividerPointerUp}
            onKeyDown={handleDividerKeyDown}
            className="relative z-10 flex h-3 cursor-ns-resize touch-none select-none items-center justify-center rounded-sm outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
          >
            <div className="h-0.5 w-8 rounded bg-border transition-colors hover:bg-primary" />
          </div>
        )}

        <div className="flex gap-2 justify-between">
          <div className="flex gap-2">
            <Toggle
              size="sm"
              variant="outline"
              className="text-foreground"
              pressed={autoExecute}
              onPressedChange={(val) => setAutoExecute(val)}
              title={
                autoExecute ? "Disable Auto Execute" : "Enable Auto Execute"
              }
            >
              <Terminal className="h-4 w-4" />
            </Toggle>

            {!autoExecute && (
              <Button size="sm" onClick={() => refetch()}>
                <Play className="mr-2 h-4 w-4" /> Execute
              </Button>
            )}
          </div>
        </div>
      </div>

      {grid}
    </div>
  );
}
