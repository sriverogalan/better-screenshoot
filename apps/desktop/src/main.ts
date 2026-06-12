import { createApp } from "vue";
import { createPinia } from "pinia";
import VueKonva from "vue-konva";
import App from "./App.vue";
import router from "./router";
import { i18n } from "./i18n";
import "./styles.css";

const app = createApp(App);
app.use(i18n);
app.use(createPinia());
app.use(router);
app.use(VueKonva);
app.mount("#app");
