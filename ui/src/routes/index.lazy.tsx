import { createFileRoute } from "@tanstack/react-router";

import { $fetch } from "@/api";

export const Route = createFileRoute("/")({
  component: Index,
  loader: () => $fetch("/"),
});

function Index() {
  const { data } = Route.useLoaderData();
  console.log(data);
  return (
    <div className="p-2">
      <h3>{data?.file_name}</h3>
      <p>SQLite Version: {data?.sqlite_version}</p>
      <p>File Size: {data?.file_size}</p>
      <p>Created on: {data?.created}</p>
      <p>Modified on: {data?.modified}</p>
      <p>Tables: {data?.tables}</p>
      <p>Indexes: {data?.indexes}</p>
      <p>Triggers: {data?.triggers}</p>
      <p>Views: {data?.views}</p>
    </div>
  );
}
