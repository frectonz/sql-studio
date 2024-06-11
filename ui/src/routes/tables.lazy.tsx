import { createFileRoute } from "@tanstack/react-router";

import { fetchTables } from "@/api";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";

export const Route = createFileRoute("/tables")({
  component: Tables,
  loader: () => fetchTables(),
});

function Tables() {
  const data = Route.useLoaderData();
  data.names.sort();

  return (
    <Tabs defaultValue="0" className="p-2 w-full overflow-x-scroll">
      <TabsList>
        {data.names.map((n, i) => (
          <TabsTrigger key={i} value={i.toString()}>
            {n}
          </TabsTrigger>
        ))}
      </TabsList>
    </Tabs>
  );
}
