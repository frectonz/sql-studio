import { createFileRoute } from "@tanstack/react-router";
import { GitBranch } from "lucide-react";

import { fetchErd } from "@/api";
import { Skeleton } from "@/components/ui/skeleton";
import { ErdDiagram } from "@/components/erd/erd-diagram";
import {
  Card,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";

export const Route = createFileRoute("/schema")({
  component: Schema,
  loader: () => fetchErd(),
  pendingComponent: SchemaSkeleton,
});

function Schema() {
  const data = Route.useLoaderData();

  if (data.tables.length === 0) {
    return (
      <Card>
        <CardHeader className="flex items-center">
          <GitBranch className="mb-4 h-12 w-12 text-muted-foreground" />
          <CardTitle>No Tables Found</CardTitle>
          <CardDescription>
            The database has no tables to display in the schema diagram.
          </CardDescription>
        </CardHeader>
      </Card>
    );
  }

  return <ErdDiagram data={data} />;
}

function SchemaSkeleton() {
  return (
    <div className="flex flex-col gap-4">
      <Skeleton className="w-full h-[calc(100vh-12rem)]" />
    </div>
  );
}
