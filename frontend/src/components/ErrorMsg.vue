<script setup lang="ts">
import Error from "@/utils/error";

const { error } = defineProps({ error: { type: Error, required: true } });
</script>

<template>
    <div class="notification is-danger">
        {{ error.status }} {{ error.statusText }} {{ error.message ? "-" : null }}
        {{ error.message }}
    </div>
</template>
