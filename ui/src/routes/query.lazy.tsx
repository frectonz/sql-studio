import "react-data-grid/lib/styles.css";

import { useState } from "react";
import DataGrid from "react-data-grid";
import { useQuery } from "@tanstack/react-query";
import { createLazyRoute } from "@tanstack/react-router";

import { fetchQuery } from "@/api";
import { Editor } from "@/components/editor";
import { Skeleton } from "@/components/ui/skeleton";

export const Route = createLazyRoute("/query")({
  component: Query,
});

function Query() {
  const [code, setCode] = useState("select 1 + 1");

  const { data, error } = useQuery({
    queryKey: ["query", code],
    queryFn: () => fetchQuery(code),
  });
  const grid = !data ? (
    error ? (
      <div>
        {/* We can have stacktrace displayed here based on the response from backend */}
        <p className="text-destructive">
          No such resource returned from this query :({" "}
        </p>
      </div>
    ) : (
      <Skeleton className="w-full h-[300px]" />
    )
  ) : (
    <DataGrid
      columns={data.columns.map((col) => ({ key: col, name: col }))}
      rows={data.rows.map((row) =>
        row.reduce((acc, curr, i) => {
          acc[data.columns[i]] = curr;
          return acc;
        }, {})
      )}
      className="rdg-light"
    />
  );

  return (
    <>
      <div className="grid gap-2 grid-cols-1">
        <Editor value={code} onChange={(v) => setCode(v)} />
      </div>

      {grid}
    </>
  );
}
