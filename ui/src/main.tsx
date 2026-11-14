import "./index.css";

import ReactDOM from "react-dom/client";
import React, { StrictMode } from "react";
import { RouterProvider, createRouter } from "@tanstack/react-router";
import {
  QueryCache,
  QueryClient,
  QueryClientProvider,
} from "@tanstack/react-query";

import { Unauthorized } from "@/api";
import { routeTree } from "./routeTree.gen";
import { SqlProvider } from "@/provider/sql.provider";
import { ThemeProvider } from "@/provider/theme.provider";

let basePath = document.querySelector<HTMLMetaElement>(
  `meta[name="BASE_PATH"]`,
);
let basepath = basePath?.content ?? "/";

const router = createRouter({ routeTree, basepath });

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

const queryClient = new QueryClient({
  queryCache: new QueryCache({
    onError: (error) => {
      if (error instanceof Unauthorized) router.navigate({ to: "/login" });
    },
  }),
  defaultOptions: {
    queries: {
      retry: (count, error) => !(error instanceof Unauthorized) && count < 3,
    },
  },
});

const rootElement = document.getElementById("root")!;
if (!rootElement.innerHTML) {
  const root = ReactDOM.createRoot(rootElement);
  root.render(
    <StrictMode>
      <SqlProvider>
        <QueryClientProvider client={queryClient}>
          <ThemeProvider>
            <RouterProvider router={router} />
          </ThemeProvider>
          <ReactQueryDevtools />
        </QueryClientProvider>
      </SqlProvider>
    </StrictMode>,
  );
}
