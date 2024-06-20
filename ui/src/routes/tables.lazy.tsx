import "react-data-grid/lib/styles.css";

import { useInfiniteQuery, useQuery } from "@tanstack/react-query";
import { Link, createFileRoute } from "@tanstack/react-router";
import { DatabaseZap, HardDrive, Table as TableIcon, TableProperties } from "lucide-react";
import { CodeBlock, irBlack as CodeDarkTheme } from "react-code-blocks";
import DataGrid from "react-data-grid";
import { z } from "zod";

import { fetchTable, fetchTableData, fetchTables } from "@/api";

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { cn } from "@/lib/utils";
import { useTheme } from "@/provider/theme.provider";

export const Route = createFileRoute("/tables")({
  component: Tables,
  loader: () => fetchTables(),
  pendingComponent: TablesSkeleton,
  validateSearch: z.object({ table: z.string().optional() }),
});

function Tables() {
  const data = Route.useLoaderData();
  const { table } = Route.useSearch();

  const tab = table ? data.tables.findIndex(({ name }) => name === table).toString() : "0";

  return (
    <Tabs defaultValue={tab}>
      <TabsList>
        {data.tables.map((n, i) => (
          <TabsTrigger key={i} value={i.toString()}>
            <Link search={{ table: n.name }}>
              {n.name} [{n.count.toLocaleString()}]
            </Link>
          </TabsTrigger>
        ))}
      </TabsList>
      {data.tables.map(({ name }, i) => (
        <TabsContent key={i} value={i.toString()} className="py-4">
          <Table name={name} />
        </TabsContent>
      ))}
    </Tabs>
  );
}

function TablesSkeleton() {
  return <Skeleton className="w-[70vw] h-[30px]" />;
}

type Props = {
  name: string;
};
function Table({ name }: Props) {
  const currentTheme = useTheme();
  const { data } = useQuery({
    queryKey: ["tables", name],
    queryFn: () => fetchTable(name),
  });

  if (!data) return <TableSkeleton />;

  return (
    <div className="flex flex-1 flex-col gap-4 md:gap-8">
      <h2 className="px-2 text-foreground scroll-m-20 border-b pb-2 text-3xl font-semibold tracking-tight first:mt-0">
        {data.name}
      </h2>

      <div className="grid gap-4 md:grid-cols-2 md:gap-8 lg:grid-cols-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Row Count</CardTitle>
            <TableIcon className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{data.row_count.toLocaleString()}</div>
            <p className="text-xs text-muted-foreground">The number of rows in the table.</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Indexes</CardTitle>
            <DatabaseZap className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{data.index_count.toLocaleString()}</div>
            <p className="text-xs text-muted-foreground">The number of indexes in the table.</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Columns</CardTitle>
            <TableProperties className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{data.column_count.toLocaleString()}</div>
            <p className="text-xs text-muted-foreground">The number of columns in the table.</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Table Size</CardTitle>
            <HardDrive className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{data.table_size}</div>
            <p className="text-xs text-muted-foreground">The size of the table on disk.</p>
          </CardContent>
        </Card>
      </div>

      <Card className="font-mono text-sm">
        <CodeBlock
          text={data.sql}
          language="sql"
          theme={currentTheme === "dark" ? CodeDarkTheme : undefined}
          showLineNumbers={false}
          customStyle={{
            FontFace: "JetBrains Mono",
            padding: "10px",
            backgroundColor: currentTheme === "dark" ? "#091813" : "#f5faf9",
            borderRadius: "10px",
          }}
        />
      </Card>

      <TableData name={data.name} />
    </div>
  );
}

function TableSkeleton() {
  return (
    <div className="flex flex-1 flex-col gap-4 md:gap-8">
      <div className="flex flex-col gap-2">
        <Skeleton className="w-[50vw] h-[50px]" />
        <span className="border-b" />
      </div>

      <div className="grid gap-4 md:grid-cols-2 md:gap-8 lg:grid-cols-4">
        <Skeleton className="h-[100px]" />
        <Skeleton className="h-[100px]" />
        <Skeleton className="h-[100px]" />
        <Skeleton className="h-[100px]" />
      </div>

      <Skeleton className="h-[400px]" />
      <Skeleton className="h-[400px]" />
    </div>
  );
}

function isAtBottom({ currentTarget }: React.UIEvent<HTMLDivElement>): boolean {
  return currentTarget.scrollTop + 10 >= currentTarget.scrollHeight - currentTarget.clientHeight;
}

type TableDataProps = {
  name: string;
};
function TableData({ name }: TableDataProps) {
  const currentTheme = useTheme();
  const { isLoading, data, fetchNextPage, hasNextPage } = useInfiniteQuery({
    queryKey: ["tables", "data", name],
    queryFn: ({ pageParam }) => fetchTableData(name, pageParam),
    initialPageParam: 1,
    getNextPageParam: (lastPage, _, lastPageParams) => {
      if (lastPage.rows.length === 0) return undefined;
      return lastPageParams + 1;
    },
  });

  if (!data) return <Skeleton className="h-[400px]" />;

  function handleScroll(event: React.UIEvent<HTMLDivElement>) {
    if (isLoading || !isAtBottom(event) || !hasNextPage) return;
    fetchNextPage();
  }

  const columns = data.pages[0].columns.map((col) => ({ key: col, name: col }));

  const grouped = data.pages.map((page) =>
    page.rows.map((row) =>
      row.reduce((acc, curr, i) => {
        acc[page.columns[i]] = curr;
        return acc;
      }, {})
    )
  ) as never[][];
  const rows = [].concat(...grouped);

  return (
    <DataGrid
      rows={rows}
      columns={columns}
      onScroll={handleScroll}
      className={cn(currentTheme === "light" ? "rdg-light" : "rdg-dark")}
    />
  );
}
