import type { Component } from "vue";
import {
  IconAdjustmentsHorizontal,
  IconCamera,
  IconCommand,
  IconDeviceDesktop,
  IconPhoto,
  IconPlug,
  IconRefresh,
} from "@tabler/icons-vue";

export interface NavItem {
  path: string;
  labelKey: string;
  descriptionKey?: string;
  icon: Component;
  tint: string;
}

export const LIBRARY_NAV: NavItem[] = [
  {
    path: "/history",
    labelKey: "nav.history",
    icon: IconPhoto,
    tint: "var(--color-sys-blue)",
  },
];

export const SETTINGS_NAV: NavItem[] = [
  {
    path: "/settings/general",
    labelKey: "nav.general",
    descriptionKey: "settings.pageDescriptions.general",
    icon: IconAdjustmentsHorizontal,
    tint: "var(--color-sys-gray)",
  },
  {
    path: "/settings/capture",
    labelKey: "settings.sections.capture",
    descriptionKey: "settings.pageDescriptions.capture",
    icon: IconCamera,
    tint: "var(--color-sys-indigo)",
  },
  {
    path: "/settings/shortcuts",
    labelKey: "settings.sections.globalShortcuts",
    descriptionKey: "settings.pageDescriptions.shortcuts",
    icon: IconCommand,
    tint: "var(--color-sys-orange)",
  },
  {
    path: "/settings/system-capture",
    labelKey: "settings.sections.captureMode",
    descriptionKey: "settings.pageDescriptions.captureMode",
    icon: IconDeviceDesktop,
    tint: "var(--color-sys-purple)",
  },
  {
    path: "/settings/integrations",
    labelKey: "settings.sections.integrations",
    descriptionKey: "settings.pageDescriptions.integrations",
    icon: IconPlug,
    tint: "var(--color-sys-green)",
  },
  {
    path: "/settings/updates",
    labelKey: "settings.updates.title",
    descriptionKey: "settings.pageDescriptions.updates",
    icon: IconRefresh,
    tint: "var(--color-sys-teal)",
  },
];

export function findNavItem(path: string): NavItem | undefined {
  return [...LIBRARY_NAV, ...SETTINGS_NAV].find((item) => item.path === path);
}
