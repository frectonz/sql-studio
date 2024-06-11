import { createLazyRoute } from "@tanstack/react-router";

export const Route = createLazyRoute("/tables")({
  component: () => <h1 className="text-4xl">Tables</h1>,
});
