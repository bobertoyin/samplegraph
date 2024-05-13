<script setup lang="ts">
import { ref, reactive, watch } from "vue";

import * as vNG from "v-network-graph";
import {
    ForceLayout,
    type ForceNodeDatum,
    type ForceEdgeDatum,
} from "v-network-graph/lib/force-layout";

import ErrorMsg from "@/components/ErrorMsg.vue";
import Error from "@/utils/error";
import type { GraphResponse } from "@/bindings/GraphResponse";

const configs = reactive(
    vNG.defineConfigs({
        view: {
            layoutHandler: new ForceLayout({
                positionFixedByDrag: false,
                positionFixedByClickWithAltKey: true,
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
                radius: 24,
            },
        },
        edge: {
            marker: {
                target: {
                    type: "arrow",
                },
            },
        },
    }),
);

const { id } = defineProps({ id: { type: String, required: true } });

const loading = ref(false);
const nodes = reactive<Record<string, { image: String }>>({});
const edges = reactive<Record<string, { source: String; target: String }>>({});
let error = reactive<Error>(new Error());

watch(
    () => id,
    async (id: string) => {
        loading.value = true;
        const response = await fetch(`/api/graph/${id}`);
        if (response.ok) {
            const data: GraphResponse = await response.json();

            data.graph.nodes.forEach((node) => {
                nodes[String(node)] = { image: data.songs[node].thumbnail };
            });

            data.graph.edges.forEach((edge, index) => {
                const [source_idx, target_idx] = edge;
                const source = data.graph.nodes[source_idx];
                const target = data.graph.nodes[target_idx];
                edges[String(index)] = { source: String(source), target: String(target) };
            });
        } else {
            await error.setFromResponse(response);
        }
        loading.value = false;
    },
    { immediate: true },
);
</script>

<template>
    <div v-if="error.isSet()" class="p-6">
        <ErrorMsg :error="error" />
    </div>
    <div v-else-if="loading" class="p-6">
        <progress class="progress is-warning" max="100"></progress>
    </div>
    <v-network-graph v-else class="graph" :nodes="nodes" :edges="edges" :configs="configs">
        <defs>
            <!--
        Define the path for clipping the face image.
        To change the size of the applied node as it changes,
        add the `clipPathUnits="objectBoundingBox"` attribute
        and specify the relative size (0.0~1.0).
      -->
            <clipPath id="faceCircle" clipPathUnits="objectBoundingBox">
                <circle cx="0.5" cy="0.5" r="0.5" />
            </clipPath>
        </defs>

        <!-- Replace the node component -->
        <template #override-node="{ nodeId, scale, config, ...slotProps }">
            <!-- circle for filling background -->
            <circle
                class="face-circle"
                :r="config.radius * scale"
                fill="#ffffff"
                v-bind="slotProps"
            />
            <!--
        The base position of the <image /> is top left. The node's
        center should be (0,0), so slide it by specifying x and y.
      -->
            <image
                class="face-picture"
                :x="-config.radius * scale"
                :y="-config.radius * scale"
                :width="config.radius * scale * 2"
                :height="config.radius * scale * 2"
                :xlink:href="`${nodes[nodeId].image}`"
                clip-path="url(#faceCircle)"
            />
            <!-- circle for drawing stroke -->
            <circle
                class="face-circle"
                :r="config.radius * scale"
                fill="none"
                stroke="#808080"
                :stroke-width="1 * scale"
                v-bind="slotProps"
            />
        </template>
    </v-network-graph>
</template>
