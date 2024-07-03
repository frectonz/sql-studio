import { createFileRoute } from "@tanstack/react-router";
import {
  Workflow,
  TextSearch,
  DatabaseZap,
  Table as TableIcon,
} from "lucide-react";
import {
  Bar,
  XAxis,
  YAxis,
  Tooltip,
  BarChart,
  TooltipProps,
  ResponsiveContainer,
} from "recharts";
import {
  NameType,
  ValueType,
} from "recharts/types/component/DefaultTooltipContent";

import { fetchOverview } from "@/api";
import {
  Card,
  CardTitle,
  CardHeader,
  CardContent,
  CardDescription,
} from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { InfoCard, InfoCardProps } from "@/components/info-card";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";

export const Route = createFileRoute("/")({
  component: Index,
  loader: () => fetchOverview(),
  pendingComponent: IndexSkeleton,
});

function Index() {
  const data = Route.useLoaderData();

  const cards: InfoCardProps[] = [
    {
      title: "TABLES",
      value: data.tables.toLocaleString(),
      description: "The number of tables in the DB.",
      icon: TableIcon,
    },
    {
      title: "INDEXES",
      value: data.indexes.toLocaleString(),
      description: "The number of indexes in the DB.",
      icon: DatabaseZap,
    },
    {
      title: "VIEWS",
      value: data.views.toLocaleString(),
      description: "The number of views in the DB.",
      icon: TextSearch,
    },
    {
      title: "TRIGGERS",
      value: data.triggers.toLocaleString(),
      description: "The number of triggers in the DB.",
      icon: Workflow,
    },
  ];

  return (
    <>
      <h2 className="scroll-m-20 border-b pb-2 text-muted-foreground text-3xl tracking-tight first:mt-0">
        EXPLORING{" "}
        <span className="font-bold text-foreground">{data.file_name}</span>
      </h2>

      <div className="grid gap-4 md:grid-cols-2 md:gap-8 lg:grid-cols-4">
        {cards.map((card, i) => (
          <InfoCard
            key={i}
            title={card.title}
            value={card.value}
            description={card.description}
            icon={card.icon}
          />
        ))}
      </div>

      <div className="grid gap-8 lg:grid-cols-2 xl:grid-cols-7">
        <Card className="xl:col-span-4">
          <CardHeader>
            <CardTitle>ROWS PER TABLE</CardTitle>
          </CardHeader>
          <CardContent className="pl-2">
            <TheBarChart counts={data.row_counts} />
          </CardContent>
        </Card>
        <Card className="xl:col-span-3">
          <CardHeader className="flex flex-row items-center">
            <div className="grid gap-2">
              <CardTitle>MORE METADATA</CardTitle>
              <CardDescription>More info about the DB.</CardDescription>
            </div>
          </CardHeader>
          <CardContent>
            <Table>
              <TableBody>
                <MetadataRow
                  name="DATABASE SIZE"
                  description="The size of the DB on disk."
                  value={data.db_size}
                />

                {data.sqlite_version && (
                  <MetadataRow
                    name="SQLite VERSION"
                    description="The SQLite version the DB was created with."
                    value={data.sqlite_version}
                  />
                )}

                {data.created && (
                  <MetadataRow
                    name="CREATED ON"
                    description="The date and time when the DB was created."
                    value={data.created.toUTCString()}
                  />
                )}

                {data.modified && (
                  <MetadataRow
                    name="MODIFIED ON"
                    description="The date and time when the DB was created."
                    value={data.modified.toUTCString()}
                  />
                )}
              </TableBody>
            </Table>
          </CardContent>
        </Card>
      </div>

      <div className="grid gap-8 lg:grid-cols-2 xl:grid-cols-7">
        <Card className="xl:col-span-3">
          <CardHeader>
            <CardTitle>INDEXES PER TABLE</CardTitle>
          </CardHeader>
          <CardContent className="pl-2">
            <TheBarChart counts={data.index_counts} />
          </CardContent>
        </Card>

        <Card className="xl:col-span-4">
          <CardHeader>
            <CardTitle>COLUMNS PER TABLE</CardTitle>
          </CardHeader>
          <CardContent className="pl-2">
            <TheBarChart counts={data.column_counts} />
          </CardContent>
        </Card>
      </div>

      <div className="grid gap-8 lg:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle>INDEXES PER TABLE</CardTitle>
          </CardHeader>
          <CardContent className="pl-2 h-[400px] overflow-y-scroll">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Index</TableHead>
                  <TableHead className="text-right">Count</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {data.index_counts.map((col) => (
                  <TableRow key={col.name}>
                    <TableCell>{col.name}</TableCell>
                    <TableCell className="text-right">{col.count}</TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>COLUMNS PER TABLE</CardTitle>
          </CardHeader>
          <CardContent className="pl-2 h-[400px] overflow-y-scroll">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Column</TableHead>
                  <TableHead className="text-right">Count</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {data.column_counts.map((col) => (
                  <TableRow key={col.name}>
                    <TableCell>{col.name}</TableCell>
                    <TableCell className="text-right">{col.count}</TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </CardContent>
        </Card>
      </div>
    </>
  );
}

type MetadataRowProps = {
  name: string;
  description: string;
  value: string;
};

function MetadataRow({ name, description, value }: MetadataRowProps) {
  return (
    <TableRow>
      <TableCell>
        <div className="font-medium">{name}</div>
        <div className="text-sm text-muted-foreground md:inline">
          {description}
        </div>
      </TableCell>
      <TableCell className="text-right">{value}</TableCell>
    </TableRow>
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

      <div className="w-full grid gap-4 lg:grid-cols-2 xl:grid-cols-7">
        <Skeleton className="xl:col-span-3 h-[400px]" />
        <Skeleton className="xl:col-span-4 h-[400px]" />
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
          </span>
          .
        </p>
      </CardContent>
    </Card>
  );
}
