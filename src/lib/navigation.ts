import { Images, Layers3, Settings2 } from "@lucide/svelte";

export const navigation = [
  { label: "Koleksi", icon: Images, active: true },
  { label: "Desktop", icon: Layers3, active: false },
  { label: "Pengaturan", icon: Settings2, active: false },
] as const;
