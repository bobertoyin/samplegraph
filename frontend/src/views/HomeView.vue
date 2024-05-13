<script setup lang="ts">
import { PhVinylRecord, PhGraph } from "@phosphor-icons/vue";
import SearchBar from "@/components/SearchBar.vue";
</script>

<template>
    <main>
        <section class="hero is-fullheight">
            <div class="hero-body is-justify-content-center">
                <div>
                    <h1 class="title">
                        <span class="icon-text">
                            <div>
                                <span class="icon"><PhVinylRecord /></span>
                                <span class="icon has-text-warning"><PhGraph weight="fill" /></span>
                            </div>
                            <span>Sample<span class="has-text-warning">Graph</span></span>
                        </span>
                    </h1>
                    <h3 class="subtitle">
                        Explore the relationships between songs through their samples.
                    </h3>
                    <SearchBar />
                </div>
            </div>
        </section>
    </main>
</template>
