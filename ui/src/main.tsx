import "./index.css";

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { RouterProvider, createRouter } from "@tanstack/react-router";
import React, { StrictMode } from "react";
import ReactDOM from "react-dom/client";

// Import the generated route tree
import { ThemeProvider } from "./provider/theme.provider";
import { routeTree } from "./routeTree.gen";

let basePath = document.querySelector<HTMLMetaElement>(
  `meta[name="BASE_PATH"]`,
);
let basepath = basePath?.content ?? "/";

// Create a new router instance
const router = createRouter({ routeTree, basepath });

// Register the router instance for type safety
declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}

const ReactQueryDevtools = import.meta.env.PROD
  ? () => null // Render nothing in production
  : React.lazy(() =>
      import("@tanstack/react-query-devtools").then((res) => ({
        default: res.ReactQueryDevtools,
      })),
    );

const queryClient = new QueryClient();

// Render the app
const rootElement = document.getElementById("root")!;
if (!rootElement.innerHTML) {
  const root = ReactDOM.createRoot(rootElement);
  root.render(
    <StrictMode>
      <QueryClientProvider client={queryClient}>
        <ThemeProvider>
          <RouterProvider router={router} />
        </ThemeProvider>
        <ReactQueryDevtools />
      </QueryClientProvider>
    </StrictMode>,
  );
}
