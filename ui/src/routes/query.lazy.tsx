import "react-data-grid/lib/styles.css";
import { useEffect, useState } from "react";

import { z } from "zod";
import DataGrid from "react-data-grid";
import { useQuery } from "@tanstack/react-query";
import { useDebounce } from "@uidotdev/usehooks";
import {
  Database,
  PencilLine,
  Play,
  Save,
  ShieldX,
  SlidersVertical,
  Terminal,
  Trash2,
} from "lucide-react";
import { createFileRoute } from "@tanstack/react-router";

import { cn } from "@/lib/utils";
import { fetchQuery } from "@/api";
import { Editor } from "@/components/editor";
import { Toggle } from "@/components/ui/toggle";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { useTheme } from "@/provider/theme.provider";
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import {
  useQueries,
  QueriesProvider,
  useQueriesDispatch,
  SavedQuery as SavedQueryType,
} from "@/provider/queries.provider";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Label } from "@/components/ui/label";
import { Input } from "@/components/ui/input";
import {
  Card,
  CardTitle,
  CardHeader,
  CardDescription,
} from "@/components/ui/card";

export const Route = createFileRoute("/query")({
  component: () => (
    <QueriesProvider>
      <Queries />
    </QueriesProvider>
  ),
  validateSearch: z.object({ sql: z.string().optional() }),
});

function Queries() {
  const queries = useQueries();

  return (
    <Tabs defaultValue="custom">
      <TabsList>
        <TabsTrigger value="custom">
          <SlidersVertical className="mr-2 h-4 w-4" /> Custom
        </TabsTrigger>

        {queries.map((n, i) => (
          <TabsTrigger key={i} value={i.toString()}>
            {n.name}
          </TabsTrigger>
        ))}
      </TabsList>

      <TabsContent value="custom" className="py-4">
        <CustomQuery />
      </TabsContent>

      {queries.map((query, i) => (
        <TabsContent key={i} value={i.toString()} className="py-4">
          <SavedQuery index={i} query={query} />
        </TabsContent>
      ))}
    </Tabs>
  );
}

function CustomQuery() {
  const { sql } = Route.useSearch();
  const navigate = Route.useNavigate();

  const dispatch = useQueriesDispatch();

  return (
    <Query
      sql={sql ?? "select 1 + 1"}
      onChange={(val) =>
        navigate({
          search: {
            sql: val,
          },
        })
      }
      onSave={(val) =>
        dispatch({
          type: "SAVE_QUERY",
          data: val,
        })
      }
    />
  );
}

type SavedQueryProps = {
  index: number;
  query: SavedQueryType;
};

function SavedQuery({ query, index }: SavedQueryProps) {
  const navigate = Route.useNavigate();
  const dispatch = useQueriesDispatch();

  navigate({
    search: undefined,
  });

  return (
    <Query
      sql={query.query}
      onDelete={() =>
        dispatch({
          type: "DELETE_QUERY",
          index,
        })
      }
      onUpdate={(val) =>
        dispatch({
          type: "UPDATE_QUERY",
          data: val,
          index,
        })
      }
    />
  );
}

type QueryProps = {
  sql: string;

  onDelete?: () => void;
  onSave?: (query: SavedQueryType) => void;
  onUpdate?: (val: string) => void;

  onChange?: (val: string) => void;
};

function Query({ sql, onChange, onSave, onDelete, onUpdate }: QueryProps) {
  const currentTheme = useTheme();
  const [codeState, setCode] = useState(sql);
  const code = useDebounce(codeState, 100);

  const [autoExecute, setAutoExecute] = useState(true);
  const [name, setName] = useState("");

  const { data, error, refetch } = useQuery({
    queryKey: ["query", code],
    queryFn: () => fetchQuery(code),
    enabled: autoExecute,
    retry: false,
  });

  const grid = !data ? (
    !autoExecute && code && error ? (
      <Card>
        <CardHeader className="flex items-center">
          <ShieldX className="mb-2 h-12 w-12 text-red-400" />
          <CardTitle className="text-red-400">Error</CardTitle>
          <CardDescription className="text-red-400">
            Query didn't execute successfully.
          </CardDescription>
        </CardHeader>
      </Card>
    ) : (
      <Skeleton className="w-full h-[300px]" />
    )
  ) : data.columns.length === 0 ? (
    <Card>
      <CardHeader className="flex items-center">
        <Database className="mb-4 h-12 w-12 text-muted-foreground" />
        <CardTitle>Query Executed</CardTitle>
        <CardDescription>Returned no data</CardDescription>
      </CardHeader>
    </Card>
  ) : (
    <DataGrid
      columns={data.columns.map((col) => ({ key: col, name: col }))}
      rows={data.rows.map((row) =>
        row.reduce((acc, curr, i) => {
          acc[data.columns[i]] = curr;
          return acc;
        }, {}),
      )}
      className={cn(currentTheme === "light" ? "rdg-light" : "rdg-dark")}
    />
  );

  useEffect(() => {
    onChange && onChange(code);
  }, [code]);

  return (
    <div className="grid gap-8">
      <div className="grid gap-4 grid-cols-1">
        <Editor value={code} onChange={(val) => setCode(val)} />

        <div className="flex gap-2 justify-between">
          <div className="flex gap-2">
            <Toggle
              size="sm"
              variant="outline"
              className="text-foreground"
              pressed={autoExecute}
              onPressedChange={(val) => setAutoExecute(val)}
              title={
                autoExecute ? "Disable Auto Execute" : "Enable Auto Execute"
              }
            >
              <Terminal className="h-4 w-4" />
            </Toggle>

            {!autoExecute && (
              <Button size="sm" onClick={() => refetch()}>
                <Play className="mr-2 h-4 w-4" /> Execute
              </Button>
            )}
          </div>

          <div className="flex gap-2">
            {onSave && (
              <Dialog>
                <DialogTrigger asChild>
                  <Button size="sm">
                    <Save className="mr-2 h-4 w-4" /> Save
                  </Button>
                </DialogTrigger>
                <DialogContent className={`sm:max-w-[450px] ${currentTheme}`}>
                  <DialogHeader>
                    <DialogTitle className="text-primary">
                      Save Query
                    </DialogTitle>
                    <DialogDescription>
                      Save this query so that you can run it later.
                    </DialogDescription>
                  </DialogHeader>
                  <div className="grid gap-4">
                    <Label htmlFor="name" className="text-primary">
                      Name
                    </Label>
                    <Input
                      id="name"
                      value={name}
                      onChange={(e) => setName(e.target.value)}
                      className="text-foreground"
                    />
                  </div>
                  <DialogFooter>
                    <DialogClose asChild>
                      <Button
                        type="submit"
                        onClick={() => {
                          onSave({ name, query: code });
                          setName("");
                        }}
                      >
                        Save
                      </Button>
                    </DialogClose>
                  </DialogFooter>
                </DialogContent>
              </Dialog>
            )}

            {onUpdate && code !== sql && (
              <Button
                variant="secondary"
                size="sm"
                onClick={() => onUpdate(code)}
              >
                <PencilLine className="mr-2 h-4 w-4" /> Update
              </Button>
            )}

            {onDelete && (
              <Button
                variant="destructive"
                size="sm"
                onClick={() => onDelete()}
              >
                <Trash2 className="mr-2 h-4 w-4" /> Delete
              </Button>
            )}
          </div>
        </div>
      </div>

      {grid}
    </div>
  );
}
