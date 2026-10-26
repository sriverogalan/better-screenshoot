import { createRouter, createWebHistory } from "vue-router";
import MainLayout from "../layouts/MainLayout.vue";
import HistoryView from "../views/HistoryView.vue";
import OverlayView from "../views/OverlayView.vue";
import EditorView from "../views/EditorView.vue";
import OnboardingView from "../views/OnboardingView.vue";
import GeneralSettingsView from "../views/settings/GeneralSettingsView.vue";
import CaptureSettingsView from "../views/settings/CaptureSettingsView.vue";
import ShortcutsSettingsView from "../views/settings/ShortcutsSettingsView.vue";
import SystemCaptureSettingsView from "../views/settings/SystemCaptureSettingsView.vue";
import IntegrationsSettingsView from "../views/settings/IntegrationsSettingsView.vue";
import UpdatesSettingsView from "../views/settings/UpdatesSettingsView.vue";

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: "/",
      component: MainLayout,
      children: [
        { path: "", redirect: "/history" },
        { path: "history", component: HistoryView },
        { path: "settings", redirect: "/settings/general" },
        { path: "settings/general", component: GeneralSettingsView },
        { path: "settings/capture", component: CaptureSettingsView },
        { path: "settings/shortcuts", component: ShortcutsSettingsView },
        { path: "settings/system-capture", component: SystemCaptureSettingsView },
        { path: "settings/integrations", component: IntegrationsSettingsView },
        { path: "settings/updates", component: UpdatesSettingsView },
      ],
    },
    { path: "/overlay", component: OverlayView },
    { path: "/editor", component: EditorView },
    { path: "/onboarding", component: OnboardingView },
    { path: "/menubar", component: () => import("../views/MenubarView.vue") },
  ],
});

export default router;
