import React, { useEffect } from "react";
import { useParams } from "react-router-dom";
import { ScaleLoader } from "react-spinners";
import ReactFlow, { Controls, Node, Edge, MarkerType } from "reactflow";
import { graphlib, layout } from "dagre";
import "reactflow/dist/style.css";

import { GraphResponse } from "./bindings/GraphResponse";

async function get_graph(id: number): Promise<GraphResponse> {
  const response = await fetch(`/api/graph/${id}?degree=3`);
  if (!response.ok) {
    throw new Error(await response.text());
  }
  return await response.json();
}

export default function Graph(): React.ReactElement {
  const { startId } = useParams();
  const id = Number(startId);

  if (Number.isNaN(id) || id < 0 || id > 4294967295) {
    throw new Response("", { status: 500, statusText: "Invalid Song ID" });
  }

  const [loading, setLoading] = React.useState(true);
  const [error, setError] = React.useState<string>();
  const [initialNodes, setInitialNodes] = React.useState<Node[]>([]);
  const [initialEdges, setInitialEdges] = React.useState<Edge[]>([]);

  useEffect(() => {
    get_graph(id)
      .then((response) => {
        let layoutGraph = new graphlib.Graph();
        layoutGraph.setDefaultEdgeLabel(() => ({}));
        layoutGraph.setGraph({ rankdir: "TB" });

        let nodes = response.graph.nodes.map((node) => {
          const id = String(node);
          return {
            id: id,
            data: {
              label: (
                <a href={response.songs[id].url} target="_blank">
                  {response.songs[id].full_title}
                </a>
              ),
            },
            position: {
              x: 0,
              y: 0,
            },
            style: {
              backgroundColor: "black",
              border: "1px solid white",
            },
            width: 150,
            height: 100,
          };
        });
        nodes.forEach((node) => {
          layoutGraph.setNode(node.id, { width: 150, height: 100 });
        });

        let edges: Edge[] = response.graph.edges.map((edge, index) => {
          const [srcIdx, tgtIdx, rel] = edge;
          const source = String(response.graph.nodes[srcIdx]);
          const target = String(response.graph.nodes[tgtIdx]);
          return {
            id: `edge ${index}`,
            source: source,
            target: target,
            label: rel.replace("_", " "),
            style: {
              strokeWidth: 2,
              stroke: "white",
            },
            markerEnd: {
              type: MarkerType.ArrowClosed,
              color: "white",
            },
          };
        });
        edges.forEach((edge) => {
          layoutGraph.setEdge(edge.source, edge.target);
        });

        layout(layoutGraph);

        nodes.forEach((node) => {
          const withPosition = layoutGraph.node(node.id);
          node.position = {
            x: withPosition.x - 75,
            y: withPosition.y - 50,
          };
        });

        setInitialNodes(nodes);
        setInitialEdges(edges);
        setLoading(false);
      })
      .catch((error) => {
        const err = error as Error;
        setError(err.message);
        setLoading(false);
      });
  }, [id, setLoading, setInitialNodes, setInitialEdges, setError, get_graph]);

  if (error) {
    throw new Response("", { statusText: error });
  }

  return (
    <>
      {loading ? (
        <ScaleLoader
          radius={0}
          loading={loading}
          speedMultiplier={0.75}
          color="#39ff14"
        />
      ) : (
        <div className="graph">
          <ReactFlow
            fitView
            defaultNodes={initialNodes}
            defaultEdges={initialEdges}
          >
            <Controls />
          </ReactFlow>
        </div>
      )}
    </>
  );
}
