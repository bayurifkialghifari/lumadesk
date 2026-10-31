import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { z } from "zod";

const tauriConfigSchema = z.object({
  app: z.object({
    security: z.object({
      csp: z.string(),
    }),
  }),
});

describe("Tauri security configuration", () => {
  it("allows asset fetches used by canvas background removal", () => {
    const configUrl = new URL("../../src-tauri/tauri.conf.json", import.meta.url);
    const config = tauriConfigSchema.parse(JSON.parse(readFileSync(configUrl, "utf8")));

    const connectSources = config.app.security.csp
      .split(";")
      .map((directive) => directive.trim())
      .find((directive) => directive.startsWith("connect-src "))
      ?.split(/\s+/)
      .slice(1);

    expect(connectSources).toEqual(expect.arrayContaining(["asset:", "http://asset.localhost"]));
  });
});
