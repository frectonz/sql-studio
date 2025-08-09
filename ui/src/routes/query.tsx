import "react-data-grid/lib/styles.css";
import { useState } from "react";

import { z } from "zod";
import { DataGrid } from "react-data-grid";
import { useQuery } from "@tanstack/react-query";
import { useDebounce } from "@uidotdev/usehooks";
import { Database, Play, ShieldX, Terminal } from "lucide-react";
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

  const { data, error, refetch } = useQuery({
    queryKey: ["query", code],
    queryFn: () => fetchQuery(code),
    enabled: autoExecute,
    retry: false,
  });

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
        columns={data.columns.map((col) => ({ key: col, name: col }))}
        rows={data.rows.map((row) =>
          row.reduce((acc, curr, i) => {
            acc[data.columns[i]] = curr;
            return acc;
          }, {}),
        )}
        className={cn(currentTheme === "light" ? "rdg-light" : "rdg-dark")}
      />
    </Card>
  );

  return (
    <div className="grid gap-8">
      <div className="grid gap-4 grid-cols-1">
        <Editor
          value={code}
          onChange={(val) => setCodeState({ type: "SET_SQL", data: val })}
        />

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
