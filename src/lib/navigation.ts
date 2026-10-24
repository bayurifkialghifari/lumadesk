import { Images, Layers3, Settings2 } from "@lucide/svelte";

export type NavigationId = "collection" | "desktop" | "settings";

export const navigation = [
  { id: "collection", label: "Koleksi", icon: Images, enabled: true },
  { id: "desktop", label: "Desktop", icon: Layers3, enabled: true },
  { id: "settings", label: "Pengaturan", icon: Settings2, enabled: false },
] as const;
