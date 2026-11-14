import React, { useEffect } from "react";
import { Outlet, createRootRoute } from "@tanstack/react-router";

import { Logo } from "@/components/logo";
import { useTheme } from "@/provider/theme.provider";

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
  notFoundComponent: NotFoundComponent,
});

function Root() {
  const theme = useTheme();

  useEffect(() => {
    document.documentElement.classList.toggle("dark", theme === "dark");
  }, [theme]);

  return (
    <>
      <div className="flex min-h-screen w-full flex-col bg-background">
        <Outlet />
      </div>
      <TanStackRouterDevtools />
    </>
  );
}

function ErrorComponent() {
  return (
    <div className="min-h-[calc(100vh-250px)] flex gap-4 justify-center items-center flex-col">
      <Logo />
      <pre className="text-destructive font-[monospace]">
        {`
███████╗    ██████╗     ██████╗      ██████╗     ██████╗ 
██╔════╝    ██╔══██╗    ██╔══██╗    ██╔═══██╗    ██╔══██╗
█████╗      ██████╔╝    ██████╔╝    ██║   ██║    ██████╔╝
██╔══╝      ██╔══██╗    ██╔══██╗    ██║   ██║    ██╔══██╗
███████╗    ██║  ██║    ██║  ██║    ╚██████╔╝    ██║  ██║
╚══════╝    ╚═╝  ╚═╝    ╚═╝  ╚═╝     ╚═════╝     ╚═╝  ╚═╝
`}
      </pre>
    </div>
  );
}

function NotFoundComponent() {
  return (
    <div className="min-h-[calc(100vh-250px)] grid place-content-center">
      <pre className="text-primary font-[monospace]">
        {`
██╗  ██╗     ██████╗     ██╗  ██╗
██║  ██║    ██╔═████╗    ██║  ██║
███████║    ██║██╔██║    ███████║
╚════██║    ████╔╝██║    ╚════██║
     ██║    ╚██████╔╝         ██║
     ╚═╝     ╚═════╝          ╚═╝
`}
      </pre>
    </div>
  );
}
