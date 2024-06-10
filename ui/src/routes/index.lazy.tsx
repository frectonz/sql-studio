import {
  Workflow,
  TextSearch,
  DatabaseZap,
  Table as TableIcon,
} from "lucide-react";
import { createFileRoute } from "@tanstack/react-router";

import { $fetch } from "@/api";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  CardDescription,
} from "@/components/ui/card";
import { Table, TableBody, TableCell, TableRow } from "@/components/ui/table";

export const Route = createFileRoute("/")({
  component: Index,
  loader: () => $fetch("/"),
  errorComponent: () => <h1>Error</h1>,
});

function Index() {
  const { data } = Route.useLoaderData();

  return (
    <>
      <h2 className="scroll-m-20 border-b pb-2 text-3xl tracking-tight first:mt-0">
        Exploring <span className="font-bold">{data?.file_name}</span>
      </h2>

      <div className="grid gap-4 md:grid-cols-2 md:gap-8 lg:grid-cols-4">
        <Card x-chunk="dashboard-01-chunk-1">
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Tables</CardTitle>
            <TableIcon className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{data?.tables}</div>
            <p className="text-xs text-muted-foreground">
              The number of tables in the DB.
            </p>
          </CardContent>
        </Card>
        <Card x-chunk="dashboard-01-chunk-2">
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Indexes</CardTitle>
            <DatabaseZap className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{data?.indexes}</div>
            <p className="text-xs text-muted-foreground">
              The number of indexes across the whole DB.
            </p>
          </CardContent>
        </Card>
        <Card x-chunk="dashboard-01-chunk-3">
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Views</CardTitle>
            <TextSearch className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{data?.views}</div>
            <p className="text-xs text-muted-foreground">
              The number of views in the DB.
            </p>
          </CardContent>
        </Card>
        <Card x-chunk="dashboard-01-chunk-3">
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Triggers</CardTitle>
            <Workflow className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{data?.triggers}</div>
            <p className="text-xs text-muted-foreground">
              The number of triggers in the DB.
            </p>
          </CardContent>
        </Card>
      </div>

      <Card className="xl:col-span-2">
        <CardHeader className="flex flex-row items-center">
          <div className="grid gap-2">
            <CardTitle>More metadata</CardTitle>
            <CardDescription>More info about the DB</CardDescription>
          </div>
        </CardHeader>
        <CardContent>
          <Table>
            <TableBody>
              <TableRow>
                <TableCell>
                  <div className="font-medium">File size</div>
                  <div className="hidden text-sm text-muted-foreground md:inline">
                    The size of the DB on disk.
                  </div>
                </TableCell>
                <TableCell className="hidden xl:table-column"></TableCell>
                <TableCell className="hidden xl:table-column"></TableCell>
                <TableCell className="hidden md:table-cell lg:hidden xl:table-column">
                  {" "}
                </TableCell>
                <TableCell className="text-right">{data?.file_size}</TableCell>
              </TableRow>

              <TableRow>
                <TableCell>
                  <div className="font-medium">SQLite version</div>
                  <div className="hidden text-sm text-muted-foreground md:inline">
                    The SQLite version the DB was created with.
                  </div>
                </TableCell>
                <TableCell className="hidden xl:table-column"></TableCell>
                <TableCell className="hidden xl:table-column"></TableCell>
                <TableCell className="hidden md:table-cell lg:hidden xl:table-column">
                  {" "}
                </TableCell>
                <TableCell className="text-right">
                  {data?.sqlite_version}
                </TableCell>
              </TableRow>

              <TableRow>
                <TableCell>
                  <div className="font-medium">Created on</div>
                  <div className="hidden text-sm text-muted-foreground md:inline">
                    The date and time when the DB was created.
                  </div>
                </TableCell>
                <TableCell className="hidden xl:table-column"></TableCell>
                <TableCell className="hidden xl:table-column"></TableCell>
                <TableCell className="hidden md:table-cell lg:hidden xl:table-column">
                  {" "}
                </TableCell>
                <TableCell className="text-right">
                  {data?.created.toUTCString()}
                </TableCell>
              </TableRow>

              <TableRow>
                <TableCell>
                  <div className="font-medium">Modified on</div>
                  <div className="hidden text-sm text-muted-foreground md:inline">
                    The date and time when the DB was last modified.
                  </div>
                </TableCell>
                <TableCell className="hidden xl:table-column"></TableCell>
                <TableCell className="hidden xl:table-column"></TableCell>
                <TableCell className="hidden md:table-cell lg:hidden xl:table-column">
                  {" "}
                </TableCell>
                <TableCell className="text-right">
                  {data?.modified.toUTCString()}
                </TableCell>
              </TableRow>
            </TableBody>
          </Table>
        </CardContent>
      </Card>
    </>
  );
}
