import { Moon, Sun, Power, LogOut } from "lucide-react";
import { useQueryClient } from "@tanstack/react-query";
import {
  Link,
  Outlet,
  createFileRoute,
  redirect,
  useNavigate,
} from "@tanstack/react-router";

import { cn } from "@/lib/utils";
import { Logo } from "@/components/logo";
import { Unauthorized, fetchMetadata, logout, sendShutdown } from "@/api";
import { setTheme, useTheme } from "@/provider/theme.provider";

import {
  AlertDialog,
  AlertDialogTitle,
  AlertDialogCancel,
  AlertDialogAction,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogContent,
  AlertDialogTrigger,
  AlertDialogDescription,
} from "@/components/ui/alert-dialog";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";

export const Route = createFileRoute("/_app")({
  beforeLoad: async () => {
    try {
      return { metadata: await fetchMetadata() };
    } catch (error) {
      if (error instanceof Unauthorized) throw redirect({ to: "/login" });
      throw error;
    }
  },
  component: App,
});

function App() {
  const theme = useTheme();
  const changeTheme = setTheme();
  const { metadata } = Route.useRouteContext();

  return (
    <>
      <header className="sticky top-0 flex h-14 items-center justify-between gap-4 border-b bg-background px-4 md:px-6 z-50">
        <nav className="hidden flex-col gap-6 text-lg font-medium sm:flex sm:flex-row sm:items-center sm:gap-5 sm:text-sm md:gap-6">
          <Link to="/">
            <Logo />
          </Link>
          <Link
            to="/"
            className="[&.active]:text-primary [&.active]:font-extrabold uppercase text-[14px]  text-muted-foreground transition-colors hover:text-foreground"
          >
            Overview
          </Link>
          <Link
            to="/tables"
            className="[&.active]:text-primary [&.active]:font-extrabold uppercase  text-[14px] text-muted-foreground transition-colors hover:text-foreground"
          >
            Tables
          </Link>
          <Link
            to="/query"
            className="[&.active]:text-primary [&.active]:font-extrabold text-[14px] uppercase text-muted-foreground transition-colors hover:text-foreground"
          >
            Query
          </Link>
          <Link
            to="/schema"
            className="[&.active]:text-primary [&.active]:font-extrabold text-[14px] uppercase text-muted-foreground transition-colors hover:text-foreground"
          >
            Schema
          </Link>
        </nav>

        <MobileNav />

        {/* logo for mobile nav*/}
        <Link to="/" className="sm:hidden w-28">
          <Logo />
        </Link>

        <div className="flex gap-4 items-center">
          <p className="text-primary hidden sm:block text-xs text-right">
            [{metadata.version}]
          </p>
          {metadata.can_shutdown && <Shutdown />}
          {metadata.can_logout && <Logout />}
          <button
            className="cursor-pointer text-foreground"
            onClick={() => {
              changeTheme(theme === "dark" ? "light" : "dark");
            }}
          >
            {theme === "dark" ? (
              <Sun
                size={28}
                className="pointer-events-none bg-primary/20 text-primary rounded-full p-1"
              />
            ) : (
              <Moon
                size={28}
                className="pointer-events-none bg-primary/10 text-primary rounded-full p-1"
              />
            )}
          </button>
          <a
            target="_blank"
            href="https://github.com/frectonz/sql-studio"
            className="flex items-center justify-end gap-2 text-lg font-semibold md:text-base"
          >
            <Github className="h-6 w-6  rounded-full " />
          </a>
        </div>
      </header>
      <main className="flex flex-1 flex-col gap-4 p-4 md:gap-8 md:p-8">
        <Outlet />
      </main>
    </>
  );
}

function Logout() {
  const navigate = useNavigate();
  const queryClient = useQueryClient();

  async function handleLogout() {
    await logout();
    await navigate({ to: "/login" });
    queryClient.clear();
  }

  return (
    <button
      className="cursor-pointer text-foreground"
      title="Log out"
      onClick={handleLogout}
    >
      <LogOut
        size={28}
        className="bg-primary/20 text-primary rounded-full p-1"
      />
    </button>
  );
}

function Shutdown() {
  const currentTheme = useTheme();

  return (
    <AlertDialog>
      <AlertDialogTrigger className="cursor-pointer text-foreground">
        <Power
          size={28}
          className="bg-primary/20 text-primary rounded-full p-1"
        />
      </AlertDialogTrigger>
      <AlertDialogContent className={`sm:max-w-[450px] ${currentTheme}`}>
        <AlertDialogHeader>
          <AlertDialogTitle className="text-primary">
            Do you really want to shutdown?
          </AlertDialogTitle>
          <AlertDialogDescription>
            Send a signal to the server telling it to shutdown.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>Cancel</AlertDialogCancel>
          <AlertDialogAction
            onClick={() => {
              sendShutdown();
            }}
          >
            Shutdown
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}

function MobileNav() {
  const currentTheme = useTheme();
  return (
    <nav className="flex items-center sm:hidden justify-between">
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <button className="cursor-pointer p-1 size-8">
            <svg
              viewBox="0 0 48 48"
              fill="none"
              xmlns="http://www.w3.org/2000/svg"
            >
              <path
                fillRule="evenodd"
                clipRule="evenodd"
                d="M6 16C6 15.4696 6.21071 14.9609 6.58579 14.5858C6.96086 14.2107 7.46957 14 8 14H40C40.5304 14 41.0391 14.2107 41.4142 14.5858C41.7893 14.9609 42 15.4696 42 16C42 16.5304 41.7893 17.0391 41.4142 17.4142C41.0391 17.7893 40.5304 18 40 18H8C7.46957 18 6.96086 17.7893 6.58579 17.4142C6.21071 17.0391 6 16.5304 6 16ZM6 32C6 31.4696 6.21071 30.9609 6.58579 30.5858C6.96086 30.2107 7.46957 30 8 30H28C28.5304 30 29.0391 30.2107 29.4142 30.5858C29.7893 30.9609 30 31.4696 30 32C30 32.5304 29.7893 33.0391 29.4142 33.4142C29.0391 33.7893 28.5304 34 28 34H8C7.46957 34 6.96086 33.7893 6.58579 33.4142C6.21071 33.0391 6 32.5304 6 32Z"
                className="fill-primary"
              />
            </svg>
            <span className="sr-only">Menu</span>
          </button>
        </DropdownMenuTrigger>
        <DropdownMenuContent
          align="start"
          className={cn(
            "uppercase font-medium rounded-none bg-background text-primary border-2",
            currentTheme,
          )}
        >
          <DropdownMenuLabel>Menu</DropdownMenuLabel>
          <DropdownMenuSeparator />
          <DropdownMenuItem asChild>
            <Link
              to="/"
              className="[&.active]:text-primary [&.active]:font-extrabold text-[14px] uppercase text-muted-foreground transition-colors [&.active]:hover:text-accent-foreground"
            >
              Overview
            </Link>
          </DropdownMenuItem>
          <DropdownMenuItem asChild>
            <Link
              to="/tables"
              className="[&.active]:text-primary [&.active]:font-extrabold text-[14px] uppercase text-muted-foreground transition-colors [&.active]:hover:text-accent-foreground"
            >
              Tables
            </Link>
          </DropdownMenuItem>
          <DropdownMenuItem asChild>
            <Link
              to="/query"
              className="[&.active]:text-primary [&.active]:font-extrabold text-[14px] uppercase text-muted-foreground transition-colors [&.active]:hover:text-accent-foreground"
            >
              Query
            </Link>
          </DropdownMenuItem>
          <DropdownMenuItem asChild>
            <Link
              to="/schema"
              className="[&.active]:text-primary [&.active]:font-extrabold text-[14px] uppercase text-muted-foreground transition-colors [&.active]:hover:text-accent-foreground"
            >
              Schema
            </Link>
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </nav>
  );
}

function Github({ className }: { className: string }) {
  return (
    <svg
      role="img"
      className={cn("fill-black dark:fill-primary", className)}
      viewBox="0 0 24 24"
      xmlns="http://www.w3.org/2000/svg"
    >
      <title>GitHub</title>
      <path d="M12 .297c-6.63 0-12 5.373-12 12 0 5.303 3.438 9.8 8.205 11.385.6.113.82-.258.82-.577 0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61C4.422 18.07 3.633 17.7 3.633 17.7c-1.087-.744.084-.729.084-.729 1.205.084 1.838 1.236 1.838 1.236 1.07 1.835 2.809 1.305 3.495.998.108-.776.417-1.305.76-1.605-2.665-.3-5.466-1.332-5.466-5.93 0-1.31.465-2.38 1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3 1.23.96-.267 1.98-.399 3-.405 1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23 3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 1.23 1.91 1.23 3.22 0 4.61-2.805 5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.606-.015 2.896-.015 3.286 0 .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12" />
    </svg>
  );
}
