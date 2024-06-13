import "react-data-grid/lib/styles.css";

import { useState } from "react";
import DataGrid from "react-data-grid";
import { useQuery } from "@tanstack/react-query";
import { createLazyRoute } from "@tanstack/react-router";

import { fetchQuery } from "@/api";
import { Editor } from "@/components/editor";

export const Route = createLazyRoute("/query")({
  component: Query,
});

function Query() {
  const [code, setCode] = useState("SELECT 1 + 1");

  const { data } = useQuery({
    queryKey: ["query", code],
    queryFn: () => fetchQuery(code),
  });

  const columns = data?.columns.map((col) => ({ key: col, name: col })) ?? [];
  const rows =
    data?.rows.map((row) =>
      row.reduce((acc, curr, i) => {
        acc[data.columns[i]] = curr;
        return acc;
      }, {}),
    ) ?? [];

  return (
    <>
      <div className="grid gap-2 grid-cols-1">
        <Editor value={code} onChange={(v) => setCode(v)} />
      </div>

      <DataGrid columns={columns} rows={rows} className="rdg-light" />
    </>
  );
}
