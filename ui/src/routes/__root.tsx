import { Menu, Database } from "lucide-react";
import { createRootRoute, Link, Outlet } from "@tanstack/react-router";
import { TanStackRouterDevtools } from "@tanstack/router-devtools";

import { Button } from "@/components/ui/button";
import { Sheet, SheetContent, SheetTrigger } from "@/components/ui/sheet";

export const Route = createRootRoute({
  component: Root,
});

export function Root() {
  return (
    <>
      <div className="flex min-h-screen w-full flex-col">
        <header className="sticky top-0 flex h-16 items-center gap-4 border-b bg-background px-4 md:px-6">
          <nav className="hidden flex-col gap-6 text-lg font-medium sm:flex sm:flex-row sm:items-center sm:gap-5 sm:text-sm md:gap-6">
            <Link
              to="/"
              className="flex items-center gap-2 text-lg font-semibold md:text-base"
            >
              <Database className="h-6 w-6" />
              <span className="sr-only">SQLite Studio</span>
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
                  <span className="sr-only">SQLite Studio</span>
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
        </header>
        <main className="flex flex-1 flex-col gap-4 p-4 md:gap-8 md:p-8">
          <Outlet />
        </main>
      </div>
      <TanStackRouterDevtools />
    </>
  );
}
