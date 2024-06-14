import {
  Workflow,
  TextSearch,
  DatabaseZap,
  Table as TableIcon,
} from "lucide-react";
import { createFileRoute } from "@tanstack/react-router";
import { Bar, BarChart, ResponsiveContainer, XAxis, YAxis } from "recharts";

import { fetchOverview } from "@/api";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  CardDescription,
} from "@/components/ui/card";
import { Table, TableBody, TableCell, TableRow } from "@/components/ui/table";

export const Route = createFileRoute("/")({
  component: Index,
  loader: () => fetchOverview(),
});

function Index() {
  const data = Route.useLoaderData();

  return (
    <>
      <h2 className="scroll-m-20 border-b pb-2 text-3xl tracking-tight first:mt-0">
        Exploring <span className="font-bold">{data.file_name}</span>
      </h2>

      <div className="grid gap-4 md:grid-cols-2 md:gap-8 lg:grid-cols-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Tables</CardTitle>
            <TableIcon className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{data.tables}</div>
            <p className="text-xs text-muted-foreground">
              The number of tables in the DB.
            </p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Indexes</CardTitle>
            <DatabaseZap className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{data.indexes}</div>
            <p className="text-xs text-muted-foreground">
              The number of indexes across the whole DB.
            </p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Views</CardTitle>
            <TextSearch className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{data.views}</div>
            <p className="text-xs text-muted-foreground">
              The number of views in the DB.
            </p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Triggers</CardTitle>
            <Workflow className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{data.triggers}</div>
            <p className="text-xs text-muted-foreground">
              The number of triggers in the DB.
            </p>
          </CardContent>
        </Card>
      </div>

      <div className="grid gap-4 lg:grid-cols-2 xl:grid-cols-7">
        <Card className="xl:col-span-4">
          <CardHeader>
            <CardTitle>Rows Per Table</CardTitle>
          </CardHeader>
          <CardContent className="pl-2">
            <TheBarChart counts={data.counts} />
          </CardContent>
        </Card>
        <Card className="xl:col-span-3">
          <CardHeader className="flex flex-row items-center">
            <div className="grid gap-2">
              <CardTitle>More metadata</CardTitle>
              <CardDescription>More info about the DB</CardDescription>
            </div>
          </CardHeader>
          <CardContent>
            <Table>
              <TableBody>
                <TableRow>
                  <TableCell>
                    <div className="font-medium">File size</div>
                    <div className="text-sm text-muted-foreground md:inline">
                      The size of the DB on disk.
                    </div>
                  </TableCell>
                  <TableCell className="text-right">{data.file_size}</TableCell>
                </TableRow>

                <TableRow>
                  <TableCell>
                    <div className="font-medium">SQLite version</div>
                    <div className="text-sm text-muted-foreground md:inline">
                      The SQLite version the DB was created with.
                    </div>
                  </TableCell>
                  <TableCell className="text-right">
                    {data.sqlite_version}
                  </TableCell>
                </TableRow>

                {data.created && (
                  <TableRow>
                    <TableCell>
                      <div className="font-medium">Created on</div>
                      <div className="text-sm text-muted-foreground md:inline">
                        The date and time when the DB was created.
                      </div>
                    </TableCell>
                    <TableCell className="text-right">
                      {data.created.toUTCString()}
                    </TableCell>
                  </TableRow>
                )}

                <TableRow>
                  <TableCell>
                    <div className="font-medium">Modified on</div>
                    <div className="text-sm text-muted-foreground md:inline">
                      The date and time when the DB was last modified.
                    </div>
                  </TableCell>
                  <TableCell className="text-right">
                    {data.modified.toUTCString()}
                  </TableCell>
                </TableRow>
              </TableBody>
            </Table>
          </CardContent>
        </Card>
      </div>
    </>
  );
}

type TheBarChartProps = {
  counts: {
    count: number;
    name: string;
  }[];
};

export function TheBarChart({ counts }: TheBarChartProps) {
  return (
    <ResponsiveContainer width="100%" height={350}>
      <BarChart data={counts}>
        <XAxis
          dataKey="name"
          stroke="#888888"
          fontSize={12}
          tickLine={false}
          axisLine={false}
        />
        <YAxis
          stroke="#888888"
          fontSize={12}
          tickLine={false}
          axisLine={false}
        />
        <Bar
          dataKey="count"
          fill="currentColor"
          radius={[4, 4, 0, 0]}
          className="fill-primary"
        />
      </BarChart>
    </ResponsiveContainer>
  );
}
