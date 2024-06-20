import { createFileRoute } from "@tanstack/react-router";
import {
  DatabaseZap,
  Table as TableIcon,
  TextSearch,
  Workflow,
} from "lucide-react";
import {
  Bar,
  BarChart,
  ResponsiveContainer,
  Tooltip,
  TooltipProps,
  XAxis,
  YAxis,
} from "recharts";

import { fetchOverview } from "@/api";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { Table, TableBody, TableCell, TableRow } from "@/components/ui/table";
import {
  NameType,
  ValueType,
} from "recharts/types/component/DefaultTooltipContent";

export const Route = createFileRoute("/")({
  component: Index,
  loader: () => fetchOverview(),
  pendingComponent: IndexSkeleton,
});

function Index() {
  const data = Route.useLoaderData();

  return (
    <>
      <h2 className="scroll-m-20 border-b pb-2 text-muted-foreground text-3xl tracking-tight first:mt-0">
        Exploring{" "}
        <span className="font-bold text-foreground">{data.file_name}</span>
      </h2>

      <div className="grid gap-4 md:grid-cols-2 md:gap-8 lg:grid-cols-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Tables</CardTitle>
            <TableIcon className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {data.tables.toLocaleString()}
            </div>
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
            <div className="text-2xl font-bold">
              {data.indexes.toLocaleString()}
            </div>
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
            <div className="text-2xl font-bold">
              {data.views.toLocaleString()}
            </div>
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
            <div className="text-2xl font-bold">
              {data.triggers.toLocaleString()}
            </div>
            <p className="text-xs text-muted-foreground">
              The number of triggers in the DB.
            </p>
          </CardContent>
        </Card>
      </div>

      <div className="grid gap-8 lg:grid-cols-2 xl:grid-cols-7">
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

                {data.sqlite_version && (
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
                )}

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

                {data.modified && (
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
                )}
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

const compactNumberFormatter = Intl.NumberFormat("en-US", {
  notation: "compact",
  maximumFractionDigits: 1,
});

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
          className="hidden"
        />
        <YAxis
          stroke="#888888"
          fontSize={12}
          tickLine={false}
          axisLine={false}
          tickFormatter={(number) => compactNumberFormatter.format(number)}
        />
        <Bar
          dataKey="count"
          fill="currentColor"
          radius={[4, 4, 0, 0]}
          className="fill-primary"
        />
        <Tooltip content={<CustomTooltip />} cursor={{ fill: "#00ffa61e" }} />
      </BarChart>
    </ResponsiveContainer>
  );
}

function IndexSkeleton() {
  return (
    <>
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

      <div className="w-full grid gap-4 lg:grid-cols-2 xl:grid-cols-7">
        <Skeleton className="xl:col-span-4 h-[400px]" />
        <Skeleton className="xl:col-span-3 h-[400px]" />
      </div>
    </>
  );
}

function CustomTooltip({
  active,
  payload,
  label,
}: TooltipProps<ValueType, NameType>) {
  if (!active || !payload || !payload.length) return null;

  return (
    <Card className="p-3">
      <CardContent className="p-0">
        <div className="font-bold"># {payload[0]?.value?.toLocaleString()}</div>
        <p className="text-xs text-muted-foreground">
          Table <span className="text-primary font-semibold">{label}</span> has{" "}
          <span className="text-primary font-semibold">
            {compactNumberFormatter.format(payload[0]?.value as number)}
          </span>{" "}
          rows.
        </p>
      </CardContent>
    </Card>
  );
}
