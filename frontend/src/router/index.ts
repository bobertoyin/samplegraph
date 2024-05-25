import { createRouter, createWebHistory } from "vue-router";

const router = createRouter({
    history: createWebHistory(import.meta.env.BASE_URL),
    routes: [
        {
            path: "/",
            name: "home",
            component: () => import("@/views/HomeView.vue"),
        },
        {
            path: "/graph/:id",
            name: "graph",
            component: () => import("@/views/GraphView.vue"),
        },
    ],
});

export default router;
