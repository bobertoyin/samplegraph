<script setup lang="ts">
import { useRoute } from "vue-router";
import GraphComponent from "@/components/GraphComponent.vue";
const route = useRoute();
const id = route.params.id instanceof Array ? route.params.id[0] : route.params.id;
</script>

<template>
    <main id="graph-view" class="flex is-align-content-center is-justify-content-center">
        <GraphComponent :id="id" />
    </main>
</template>
