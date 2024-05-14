import { createApp } from "vue";
import vueDebounce from "vue-debounce";
import VNetworkGraph from "v-network-graph";
import router from "./router";
import App from "./App.vue";
import "v-network-graph/lib/style.css";
import "@/assets/styles/main.scss";

createApp(App)
    .use(router)
    .use(VNetworkGraph)
    .directive("debounce", vueDebounce({ lock: true }))
    .mount("#app");
