import React from "react";
import { Menu, Database, Frown } from "lucide-react";
import { createRootRoute, Link, Outlet } from "@tanstack/react-router";

import { Button } from "@/components/ui/button";
import { Sheet, SheetContent, SheetTrigger } from "@/components/ui/sheet";

const TanStackRouterDevtools = import.meta.env.PROD
  ? () => null // Render nothing in production
  : React.lazy(() =>
      import("@tanstack/router-devtools").then((res) => ({
        default: res.TanStackRouterDevtools,
      })),
    );

export const Route = createRootRoute({
  component: Root,
  errorComponent: ErrorComponent,
});

export function Root() {
  return (
    <>
      <div className="flex min-h-screen w-full flex-col">
        <header className="sticky top-0 flex h-16 items-center gap-4 border-b bg-background px-4 md:px-6 z-50">
          <nav className="hidden flex-col gap-6 text-lg font-medium sm:flex sm:flex-row sm:items-center sm:gap-5 sm:text-sm md:gap-6">
            <Link
              to="/"
              className="flex items-center gap-2 text-lg font-semibold md:text-base"
            >
              <Database className="h-6 w-6" />
              <span className="text-foreground">SQLite Studio</span>
            </Link>
            <Link
              to="/"
              className="[&.active]:text-foreground text-muted-foreground transition-colors hover:text-foreground"
            >
              Overview
            </Link>
            <Link
              to="/tables"
              className="[&.active]:text-foreground text-muted-foreground transition-colors hover:text-foreground"
            >
              Tables
            </Link>
            <Link
              to="/query"
              className="[&.active]:text-foreground text-muted-foreground transition-colors hover:text-foreground"
            >
              Query
            </Link>
          </nav>

          <Sheet>
            <SheetTrigger asChild>
              <Button
                variant="outline"
                size="icon"
                className="shrink-0 sm:hidden"
              >
                <Menu className="h-5 w-5" />
                <span className="sr-only">Toggle navigation menu</span>
              </Button>
            </SheetTrigger>
            <SheetContent side="left">
              <nav className="grid gap-6 text-lg font-medium">
                <Link
                  href="#"
                  className="flex items-center gap-2 text-lg font-semibold"
                >
                  <Database className="h-6 w-6" />
                  <span className="text-foreground">SQLite Studio</span>
                </Link>
                <Link
                  to="/"
                  className="[&.active]:text-foreground text-muted-foreground hover:text-foreground"
                >
                  Overview
                </Link>
                <Link
                  to="/tables"
                  className="[&.active]:text-foreground text-muted-foreground hover:text-foreground"
                >
                  Tables
                </Link>
                <Link
                  to="/query"
                  className="[&.active]:text-foreground text-muted-foreground hover:text-foreground"
                >
                  Query
                </Link>
              </nav>
            </SheetContent>
          </Sheet>

          <Link
            to="/"
            className="flex sm:hidden items-center gap-2 text-lg font-semibold md:text-base"
          >
            <Database className="h-6 w-6" />
            <span className="text-foreground">SQLite Studio</span>
          </Link>

          <a
            target="_blank"
            href="https://github.com/frectonz/sqlite-studio"
            className="flex flex-1 items-center justify-end gap-2 text-lg font-semibold md:text-base"
          >
            <Github className="h-6 w-6" />
          </a>
        </header>
        <main className="flex flex-1 flex-col gap-4 p-4 md:gap-8 md:p-8">
          <Outlet />
        </main>
      </div>
      <TanStackRouterDevtools />
    </>
  );
}

function Github({ className }: { className: string }) {
  return (
    <svg
      role="img"
      className={className}
      viewBox="0 0 24 24"
      xmlns="http://www.w3.org/2000/svg"
    >
      <title>GitHub</title>
      <path d="M12 .297c-6.63 0-12 5.373-12 12 0 5.303 3.438 9.8 8.205 11.385.6.113.82-.258.82-.577 0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61C4.422 18.07 3.633 17.7 3.633 17.7c-1.087-.744.084-.729.084-.729 1.205.084 1.838 1.236 1.838 1.236 1.07 1.835 2.809 1.305 3.495.998.108-.776.417-1.305.76-1.605-2.665-.3-5.466-1.332-5.466-5.93 0-1.31.465-2.38 1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3 1.23.96-.267 1.98-.399 3-.405 1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23 3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 1.23 1.91 1.23 3.22 0 4.61-2.805 5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.606-.015 2.896-.015 3.286 0 .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12" />
    </svg>
  );
}

function ErrorComponent() {
  return (
    <div className="w-screen h-screen text-red-500 flex flex-col items-center justify-center gap-6">
      <Frown className="w-12 h-12" />
      <h1 className="scroll-m-20 text-3xl tracking-tight lg:text-4xl">
        Something Went Wrong.
      </h1>
    </div>
  );
}
