<script setup lang="ts">
import { ref, reactive, watch, computed } from "vue";
import * as vNG from "v-network-graph";
import {
    ForceLayout,
    type ForceNodeDatum,
    type ForceEdgeDatum,
} from "v-network-graph/lib/force-layout";
import { PhVinylRecord, PhLineSegment } from "@phosphor-icons/vue";

import ErrorMsg from "@/components/ErrorMsg.vue";
import Error from "@/utils/error";
import type { GraphResponse } from "@/bindings/GraphResponse";

const NODE_RADIUS = 24;

const configs = reactive(
    vNG.defineConfigs({
        view: {
            layoutHandler: new ForceLayout({
                positionFixedByDrag: false,
                positionFixedByClickWithAltKey: false,
                noAutoRestartSimulation: true,
                createSimulation: (d3, nodes, edges) => {
                    const forceLink = d3
                        .forceLink<ForceNodeDatum, ForceEdgeDatum>(edges)
                        .id((d: { id: any }) => d.id);
                    return d3
                        .forceSimulation(nodes)
                        .force("edge", forceLink.distance(100))
                        .force("charge", d3.forceManyBody().strength(-800))
                        .force("center", d3.forceCenter().strength(0.05))
                        .alphaMin(0.001);
                },
            }),
        },
        node: {
            normal: {
                radius: NODE_RADIUS,
            },
            label: {
                visible: false,
            },
            selectable: 1,
        },
        edge: {
            selectable: 1,
            marker: {
                target: {
                    type: "custom",
                    customId: "arrow-marker",
                },
            },
        },
    }),
);

const eventHandlers: vNG.EventHandlers = {
    "node:select": (event) => {
        if (event.length === 0) {
            selectedNodeId.value = null;
            document.getElementById("node-tooltip")?.classList.add("hidden");
        } else {
            selectedNodeId.value = event[0];
            document.getElementById("node-tooltip")?.classList.remove("hidden");
        }
    },
    "edge:select": (event) => {
        if (event.length === 0) {
            selectedEdgeId.value = null;
            document.getElementById("edge-tooltip")?.classList.add("hidden");
        } else {
            selectedEdgeId.value = event[0];
            document.getElementById("edge-tooltip")?.classList.remove("hidden");
        }
    },
};

const { id } = defineProps({ id: { type: String, required: true } });

const graph = ref<vNG.Instance>();
const layouts = ref<{ nodes: vNG.NodePositions }>();

const selectedNodeId = ref<string | null | undefined>();
const selectedNodePos = computed(() => {
    if (layouts.value && selectedNodeId.value) {
        return layouts.value["nodes"][selectedNodeId.value];
    } else {
        return { x: 0, y: 0 };
    }
});
const nodeToolTip = ref<HTMLDivElement>();
const nodeTooltipPos = ref({ left: "0px", top: "0px" });

const selectedEdgeId = ref<string | null | undefined>();
const selectedEdgePos = computed(() => {
    if (layouts.value && selectedEdgeId.value) {
        const edge = edges[selectedEdgeId.value];
        const source = edge.source;
        const target = edge.target;
        return {
            x: (layouts.value["nodes"][source].x + layouts.value["nodes"][target].x) / 2,
            y: (layouts.value["nodes"][source].y + layouts.value["nodes"][target].y) / 2,
        };
    } else {
        return { x: 0, y: 0 };
    }
});
const edgeToolTip = ref<HTMLDivElement>();
const edgeTooltipPos = ref({ left: "0px", top: "0px" });

const loading = ref(false);
const nodes = reactive<
    Record<string, { title: string; artist: string; image: string; degree: number }>
>({});
const edges = reactive<Record<string, { source: string; target: string; label: string }>>({});
let node_count = 0;
let edge_count = 0;
const error = reactive<Error>(new Error());

watch(
    () => id,
    async (id: string) => {
        loading.value = true;
        const response = await fetch(`/api/graph/${id}`);
        if (response.ok) {
            const data: GraphResponse = await response.json();

            data.graph.nodes.forEach((node) => {
                const song = data.songs[node];
                const key = String(node);
                if (!(key in nodes)) {
                    node_count += 1;
                }
                nodes[key] = {
                    image: data.songs[node].thumbnail,
                    title: song.title,
                    artist: song.artist,
                    degree: song.degree,
                };
            });

            data.graph.edges.forEach((edge, index) => {
                const [source_idx, target_idx, relationship] = edge;
                const key = String(index);
                const source = data.graph.nodes[source_idx];
                const target = data.graph.nodes[target_idx];
                if (!(key in edges)) {
                    edge_count += 1;
                }
                edges[key] = {
                    source: String(source),
                    target: String(target),
                    label: relationship.replace("_", " "),
                };
            });
        } else {
            await error.setFromResponse(response);
        }
        loading.value = false;
    },
    { immediate: true },
);

watch(
    () => selectedNodePos.value,
    () => {
        if (graph.value && nodeToolTip.value && selectedNodePos.value) {
            const position = graph.value.translateFromSvgToDomCoordinates(selectedNodePos.value);
            nodeTooltipPos.value = {
                left: position.x - nodeToolTip.value.offsetWidth / 2 + "px",
                top: position.y - NODE_RADIUS - nodeToolTip.value.offsetHeight - 12 + "px",
            };
        }
    },
    { deep: true },
);

watch(
    () => selectedEdgePos.value,
    () => {
        if (graph.value && edgeToolTip.value && selectedEdgePos.value) {
            const position = graph.value.translateFromSvgToDomCoordinates(selectedEdgePos.value);
            edgeTooltipPos.value = {
                left: position.x - edgeToolTip.value.offsetWidth / 2 + "px",
                top: position.y - 2 - edgeToolTip.value.offsetHeight - 10 + "px",
            };
        }
    },
    { deep: true },
);
</script>

<template>
    <div v-if="error.isSet()" class="p-6">
        <ErrorMsg :error="error" />
    </div>
    <div v-else-if="loading" class="p-6">
        <progress class="progress is-warning" max="100"></progress>
    </div>
    <div v-else class="m-0 p-0 graph">
        <div id="graph-info">
            <p>
                <span class="icon-text">
                    <span class="icon has-text-warning"><PhVinylRecord /></span>
                    <span>{{ node_count }} song{{ node_count != 1 ? "s" : null }}</span>
                </span>
            </p>
            <p>
                <span class="icon-text">
                    <span class="icon has-text-warning"><PhLineSegment /></span>
                    <span>{{ edge_count }} relationship{{ edge_count != 1 ? "s" : null }}</span>
                </span>
            </p>
        </div>
        <div ref="nodeToolTip" id="node-tooltip" class="hidden" :style="{ ...nodeTooltipPos }">
            <p v-if="selectedNodeId" class="is-size-7">
                <strong>{{ nodes[selectedNodeId].title }}</strong
                >{{}}
                <small>{{ nodes[selectedNodeId].artist }}</small>
            </p>
        </div>
        <div ref="edgeToolTip" id="edge-tooltip" class="hidden" :style="{ ...edgeTooltipPos }">
            <p v-if="selectedEdgeId" class="is-size-7">
                {{ edges[selectedEdgeId].label }}
            </p>
        </div>
        <v-network-graph
            ref="graph"
            v-model:layouts="layouts"
            :nodes="nodes"
            :edges="edges"
            :configs="configs"
            :eventHandlers="eventHandlers"
        >
            <defs>
                <clipPath id="coverArt" clipPathUnits="objectBoundingBox">
                    <circle cx="0.5" cy="0.5" r="0.5" />
                </clipPath>
                <marker
                    id="arrow-marker"
                    markerWidth="5"
                    markerHeight="5"
                    refX="1"
                    refY="2.5"
                    orient="auto"
                    markerUnits="strokeWidth"
                    class="v-ng-marker"
                >
                    <polygon points="0 0, 5 2.5, 0 5"></polygon>
                </marker>
                <marker
                    id="arrow-marker-selected"
                    markerWidth="5"
                    markerHeight="5"
                    refX="1"
                    refY="2.5"
                    orient="auto"
                    markerUnits="strokeWidth"
                    class="v-ng-marker"
                >
                    <polygon points="0 0, 5 2.5, 0 5"></polygon>
                </marker>
            </defs>

            <template #override-node="{ nodeId, scale, config, ...slotProps }">
                <circle :r="config.radius * scale" v-bind="slotProps" />
                <image
                    :x="-config.radius * scale"
                    :y="-config.radius * scale"
                    :width="config.radius * scale * 2"
                    :height="config.radius * scale * 2"
                    :xlink:href="nodes[nodeId].image"
                    clip-path="url(#coverArt)"
                />
                <circle
                    class="node-outline"
                    v-bind:class="{ 'root-node': nodes[nodeId].degree == 0 }"
                    :r="config.radius * scale"
                    fill="none"
                    :stroke-width="1 * scale"
                    v-bind="slotProps"
                />
            </template>
        </v-network-graph>
    </div>
</template>
