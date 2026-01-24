import { Handle, Position, type NodeProps } from "@xyflow/react";
import { Table as TableIcon, KeyRound } from "lucide-react";

import { cn } from "@/lib/utils";

type Column = {
  name: string;
  data_type: string;
  nullable: boolean;
  is_primary_key: boolean;
};

type TableNodeData = {
  label: string;
  columns: Column[];
};

export function TableNode({ data }: NodeProps) {
  const { label, columns } = data as TableNodeData;

  return (
    <div className="bg-card border border-border rounded-lg shadow-md min-w-[250px] overflow-hidden">
      <Handle type="target" position={Position.Left} className="!bg-primary" />
      <Handle type="source" position={Position.Right} className="!bg-primary" />

      <div className="bg-primary/10 px-3 py-2 border-b border-border flex items-center gap-2">
        <TableIcon className="h-4 w-4 text-primary" />
        <span className="font-semibold text-sm text-foreground uppercase">
          {label}
        </span>
      </div>

      <div className="divide-y divide-border">
        {columns.map((column) => (
          <div
            key={column.name}
            className={cn(
              "px-3 py-1.5 flex items-center gap-2 text-xs",
              column.is_primary_key && "bg-primary/5",
            )}
          >
            <div className="w-4 flex-shrink-0">
              {column.is_primary_key && (
                <KeyRound className="h-3 w-3 text-primary" />
              )}
            </div>
            <span className="font-medium text-foreground flex-1 truncate">
              {column.name}
            </span>
            <span className="text-muted-foreground uppercase text-[10px]">
              {column.data_type}
            </span>
            {column.nullable && (
              <span className="text-[9px] text-muted-foreground bg-muted px-1 rounded">
                NULL
              </span>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}
