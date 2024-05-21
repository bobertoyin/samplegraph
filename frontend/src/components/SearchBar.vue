<script setup lang="ts">
import { reactive, ref, watch } from "vue";
import { PhMagnifyingGlass, PhGraph, PhArrowSquareOut } from "@phosphor-icons/vue";

import type { SearchResponse } from "@/bindings/SearchResponse";
import type { SearchHit } from "@/bindings/SearchHit";
import ErrorMsg from "@/components/ErrorMsg.vue";
import Error from "@/utils/error";

let query = ref("");
let loading = ref(false);
let hits = ref<SearchHit[]>([]);
let error = reactive<Error>(new Error());

function updateQuery(value: string) {
    query.value = value.trim();
}

watch(query, async (query: String) => {
    if (query !== "") {
        loading.value = true;
        const response = await fetch(`/api/search?query=${query}`);
        if (response.ok) {
            const data: SearchResponse = await response.json();
            hits.value = data.hits;
            error.reset();
        } else {
            await error.setFromResponse(response);
        }
    } else {
        hits.value = [];
        error.reset();
    }
    loading.value = false;
});
</script>

<template>
    <form @submit.prevent>
        <div class="field">
            <div class="control has-icons-left" :class="{ 'is-loading': loading }">
                <span class="icon is-left"><PhMagnifyingGlass /></span>
                <input
                    class="input is-warning"
                    type="text"
                    placeholder="Search"
                    v-debounce:300ms="updateQuery"
                />
            </div>
        </div>
    </form>
    <div id="search-results" class="mt-5">
        <div v-if="error.isSet()">
            <ErrorMsg :error="error" />
        </div>
        <article v-else-if="hits.length > 0" v-for="hit in hits" :key="hit.id" class="media">
            <figure class="media-left">
                <p class="image hit-thumbnail">
                    <img :src="hit.thumbnail" :alt="hit.title + ' by ' + hit.artist" />
                </p>
            </figure>
            <div class="media-content">
                <div class="content">
                    <p>
                        <strong>{{ hit.title }}</strong> <small>{{ hit.artist }}</small>
                    </p>
                    <p>
                        <span class="icon-text has-text-warning">
                            <span class="icon"><PhGraph /></span>
                            <span
                                ><RouterLink :to="{ name: 'graph', params: { id: hit.id } }"
                                    >View Graph</RouterLink
                                ></span
                            >
                        </span>
                        <br />
                        <span class="icon-text has-text-warning">
                            <span class="icon"><PhArrowSquareOut /></span>
                            <span><a :href="hit.url" target="_blank">View on Genius</a></span>
                        </span>
                    </p>
                </div>
            </div>
        </article>
    </div>
</template>
