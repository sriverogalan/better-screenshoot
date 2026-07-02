import { createRouter, createWebHistory } from "vue-router";
import MainLayout from "../layouts/MainLayout.vue";
import HistoryView from "../views/HistoryView.vue";
import SettingsView from "../views/SettingsView.vue";
import CaptureWindowView from "../views/CaptureWindowView.vue";
import OverlayView from "../views/OverlayView.vue";
import EditorView from "../views/EditorView.vue";
import OnboardingView from "../views/OnboardingView.vue";

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: "/",
      component: MainLayout,
      children: [
        { path: "", redirect: "/settings" },
        { path: "history", component: HistoryView },
        { path: "settings", component: SettingsView },
        { path: "capture-window", component: CaptureWindowView },
      ],
    },
    { path: "/overlay", component: OverlayView },
    { path: "/editor", component: EditorView },
    { path: "/onboarding", component: OnboardingView },
  ],
});

export default router;
