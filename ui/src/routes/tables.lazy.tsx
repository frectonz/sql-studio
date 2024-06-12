import "react-data-grid/lib/styles.css";

import DataGrid from "react-data-grid";
import { CopyBlock } from "react-code-blocks";
import { useQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";

import { fetchTable, fetchTables } from "@/api";
import { Tabs, TabsList, TabsContent, TabsTrigger } from "@/components/ui/tabs";
import { Card } from "@/components/ui/card";

export const Route = createFileRoute("/tables")({
  component: Tables,
  loader: () => fetchTables(),
});

function Tables() {
  const data = Route.useLoaderData();

  return (
    <Tabs defaultValue="0">
      <TabsList>
        {data.names.map((n, i) => (
          <TabsTrigger key={i} value={i.toString()}>
            {n}
          </TabsTrigger>
        ))}
      </TabsList>
      {data.names.map((n, i) => (
        <TabsContent key={i} value={i.toString()} className="py-4">
          <Table name={n} />
        </TabsContent>
      ))}
    </Tabs>
  );
}

type Props = {
  name: string;
};
function Table({ name }: Props) {
  const { data } = useQuery({
    queryKey: ["tables", name],
    queryFn: () => fetchTable(name),
  });

  if (!data) return;

  const columns = data.columns.map((col) => ({ key: col, name: col }));
  const rows = data.rows.map((row) =>
    row.reduce((acc, curr, i) => {
      acc[data.columns[i]] = curr;
      return acc;
    }, {}),
  );

  return (
    <div className="flex flex-1 flex-col gap-4 md:gap-8">
      <h2 className="px-2 scroll-m-20 border-b pb-2 text-3xl font-semibold tracking-tight first:mt-0">
        {data.name}
      </h2>
      <Card className="font-mono text-sm">
        <CopyBlock text={data.sql} language="sql" showLineNumbers={false} />
      </Card>
      <DataGrid columns={columns} rows={rows} className="rdg-light" />
    </div>
  );
}
