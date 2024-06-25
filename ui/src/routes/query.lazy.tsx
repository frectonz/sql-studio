import "react-data-grid/lib/styles.css";
import { useEffect, useState } from "react";

import { z } from "zod";
import DataGrid from "react-data-grid";
import { Play, Terminal } from "lucide-react";
import { useQuery } from "@tanstack/react-query";
import { useDebounce } from "@uidotdev/usehooks";
import { createFileRoute } from "@tanstack/react-router";

import { cn } from "@/lib/utils";
import { fetchQuery } from "@/api";
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
  const { sql } = Route.useSearch();
  const navigate = Route.useNavigate();

  const currentTheme = useTheme();
  const [codeState, setCode] = useState(sql ?? "select 1 + 1");
  const code = useDebounce(codeState, 100);

  const [autoExecute, setAutoExecute] = useState(true);

  const { data, refetch, isLoading } = useQuery({
    queryKey: ["query", code],
    queryFn: () => fetchQuery(code),
    enabled: autoExecute,
  });

  const grid = !data ? (
    isLoading && <Skeleton className="w-full h-[300px]" />
  ) : (
    <DataGrid
      columns={data.columns.map((col) => ({ key: col, name: col }))}
      rows={data.rows.map((row) =>
        row.reduce((acc, curr, i) => {
          acc[data.columns[i]] = curr;
          return acc;
        }, {}),
      )}
      className={cn(currentTheme === "light" ? "rdg-light" : "rdg-dark")}
    />
  );

  useEffect(() => {
    navigate({
      search: {
        sql: code,
      },
    });
  }, [code]);

  return (
    <>
      <div className="grid gap-2 grid-cols-1">
        <Editor value={code} onChange={(val) => setCode(val)} />
        <div className="flex gap-2">
          <Toggle
            size="sm"
            variant="outline"
            className="text-foreground"
            pressed={autoExecute}
            onPressedChange={(val) => setAutoExecute(val)}
            title={autoExecute ? "Disable Auto Execute" : "Enable Auto Execute"}
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

      {grid}
    </>
  );
}
