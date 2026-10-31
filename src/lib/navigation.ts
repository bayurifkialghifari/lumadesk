import { Images, Layers3, Settings2, WandSparkles } from "@lucide/svelte";

export type NavigationId = "collection" | "creator" | "desktop" | "settings";

export const navigation = [
  { id: "collection", label: "Collection", icon: Images, enabled: true },
  { id: "creator", label: "Creator", icon: WandSparkles, enabled: true },
  { id: "desktop", label: "Desktop", icon: Layers3, enabled: true },
  { id: "settings", label: "Settings", icon: Settings2, enabled: true },
] as const;
