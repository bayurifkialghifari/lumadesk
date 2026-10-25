import { Images, Layers3, Settings2 } from "@lucide/svelte";

export type NavigationId = "collection" | "desktop" | "settings";

export const navigation = [
  { id: "collection", label: "Collection", icon: Images, enabled: true },
  { id: "desktop", label: "Desktop", icon: Layers3, enabled: true },
  { id: "settings", label: "Settings", icon: Settings2, enabled: false },
] as const;
