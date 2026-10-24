import "@fontsource-variable/inter";
import "@fontsource-variable/space-grotesk";
import { isTauri } from "@tauri-apps/api/core";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { mount } from "svelte";
import App from "./App.svelte";
import OverlayRenderer from "./lib/components/OverlayRenderer.svelte";
import { type OverlayId, overlayIdSchema } from "./lib/contracts";
import "./styles/global.css";

const target = document.getElementById("app");

if (target === null) {
  throw new TypeError("LumaDesk app root is missing");
}

const overlayId = currentOverlayId();
document.documentElement.dataset.windowKind = overlayId === null ? "main" : "overlay";

if (overlayId === null) mount(App, { target });
else mount(OverlayRenderer, { target, props: { overlayId } });

function currentOverlayId(): OverlayId | null {
  if (!isTauri()) return null;
  const label = getCurrentWebview().label;
  if (!label.startsWith("overlay-")) return null;
  const parsed = overlayIdSchema.safeParse(label.slice("overlay-".length));
  return parsed.success ? parsed.data : null;
}
