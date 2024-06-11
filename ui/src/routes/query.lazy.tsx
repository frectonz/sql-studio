import { createLazyRoute } from "@tanstack/react-router";

export const Route = createLazyRoute("/query")({
  component: () => <h1 className="text-4xl">Query</h1>,
});
