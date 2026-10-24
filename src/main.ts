import "@fontsource-variable/inter";
import "@fontsource-variable/space-grotesk";
import { mount } from "svelte";
import App from "./App.svelte";
import "./styles/global.css";

const target = document.getElementById("app");

if (target === null) {
  throw new TypeError("LumaDesk app root is missing");
}

mount(App, { target });
