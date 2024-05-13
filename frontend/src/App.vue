<script setup lang="ts">
import { RouterView } from "vue-router";
import { provide } from "vue";

provide("weight", "bold");
</script>

<template>
    <RouterView />
</template>
