import dagre from "dagre";
import { type Node, type Edge, Position } from "@xyflow/react";

const NODE_WIDTH = 280;
const COLUMN_HEIGHT = 28;
const HEADER_HEIGHT = 44;

export function useLayout(
  nodes: Node[],
  edges: Edge[],
): { nodes: Node[]; edges: Edge[] } {
  const dagreGraph = new dagre.graphlib.Graph();
  dagreGraph.setDefaultEdgeLabel(() => ({}));
  dagreGraph.setGraph({ rankdir: "LR", nodesep: 50, ranksep: 100 });

  nodes.forEach((node) => {
    const columnCount = (node.data.columns as unknown[])?.length ?? 0;
    const height = HEADER_HEIGHT + columnCount * COLUMN_HEIGHT;
    dagreGraph.setNode(node.id, { width: NODE_WIDTH, height });
  });

  edges.forEach((edge) => {
    dagreGraph.setEdge(edge.source, edge.target);
  });

  dagre.layout(dagreGraph);

  const layoutedNodes = nodes.map((node) => {
    const nodeWithPosition = dagreGraph.node(node.id);
    const columnCount = (node.data.columns as unknown[])?.length ?? 0;
    const height = HEADER_HEIGHT + columnCount * COLUMN_HEIGHT;

    return {
      ...node,
      position: {
        x: nodeWithPosition.x - NODE_WIDTH / 2,
        y: nodeWithPosition.y - height / 2,
      },
      targetPosition: Position.Left,
      sourcePosition: Position.Right,
    };
  });

  return { nodes: layoutedNodes, edges };
}
