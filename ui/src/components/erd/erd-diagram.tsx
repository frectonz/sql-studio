import { useMemo, useCallback } from "react";
import {
  ReactFlow,
  Background,
  Controls,
  MiniMap,
  useNodesState,
  useEdgesState,
  type Node,
  type Edge,
  type ColorMode,
  MarkerType,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";

import { useTheme } from "@/provider/theme.provider";
import { TableNode } from "./table-node";
import { useLayout } from "./use-layout";

type Column = {
  name: string;
  data_type: string;
  nullable: boolean;
  is_primary_key: boolean;
};

type Table = {
  name: string;
  columns: Column[];
};

type Relationship = {
  from_table: string;
  from_column: string;
  to_table: string;
  to_column: string;
};

type ErdData = {
  tables: Table[];
  relationships: Relationship[];
};

const nodeTypes = {
  tableNode: TableNode,
};

type Props = {
  data: ErdData;
};

export function ErdDiagram({ data }: Props) {
  const theme = useTheme();

  const { initialNodes, initialEdges } = useMemo(() => {
    const nodes: Node[] = data.tables.map((table) => ({
      id: table.name,
      type: "tableNode",
      position: { x: 0, y: 0 },
      data: {
        label: table.name,
        columns: table.columns,
      },
    }));

    const edges: Edge[] = data.relationships.map((rel, index) => ({
      id: `edge-${index}`,
      source: rel.from_table,
      target: rel.to_table,
      sourceHandle: rel.from_column,
      targetHandle: rel.to_column,
      type: "smoothstep",
      animated: true,
      markerEnd: {
        type: MarkerType.ArrowClosed,
        color: "var(--primary)",
      },
      style: {
        stroke: "var(--primary)",
        strokeWidth: 2,
      },
      label: `${rel.from_column} → ${rel.to_column}`,
      labelStyle: {
        fill: "var(--muted-foreground)",
        fontSize: 10,
      },
      labelBgStyle: {
        fill: "var(--background)",
      },
    }));

    return { initialNodes: nodes, initialEdges: edges };
  }, [data]);

  const { nodes: layoutedNodes, edges: layoutedEdges } = useLayout(
    initialNodes,
    initialEdges,
  );

  const [nodes, setNodes, onNodesChange] = useNodesState(layoutedNodes);
  const [edges, setEdges, onEdgesChange] = useEdgesState(layoutedEdges);

  const onInit = useCallback(() => {
    setNodes(layoutedNodes);
    setEdges(layoutedEdges);
  }, [layoutedNodes, layoutedEdges, setNodes, setEdges]);

  return (
    <div className="w-full h-[calc(100vh-8rem)] rounded-lg border border-border overflow-hidden">
      <ReactFlow
        nodes={nodes}
        edges={edges}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        onInit={onInit}
        nodeTypes={nodeTypes}
        colorMode={theme as ColorMode}
        fitView
        fitViewOptions={{ padding: 0.2 }}
        minZoom={0.1}
        maxZoom={2}
        defaultEdgeOptions={{
          type: "smoothstep",
        }}
      >
        <Background gap={16} size={1} />
        <Controls />
        <MiniMap
          nodeStrokeColor="var(--primary)"
          nodeColor="var(--card)"
          nodeBorderRadius={4}
        />
      </ReactFlow>
    </div>
  );
}
