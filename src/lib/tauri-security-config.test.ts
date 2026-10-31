import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { z } from "zod";

const tauriConfigSchema = z.object({
  app: z.object({
    security: z.object({
      csp: z.string(),
      assetProtocol: z.object({
        enable: z.boolean(),
        scope: z.array(z.string()),
      }),
    }),
  }),
  bundle: z.object({
    createUpdaterArtifacts: z.boolean(),
  }),
  plugins: z.object({
    updater: z.object({
      endpoints: z.array(z.url()),
      pubkey: z.string().min(1),
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
    expect(config.app.security.assetProtocol.enable).toBe(true);
    expect(config.app.security.assetProtocol.scope).toContain("$APPDATA/gallery/**");
  });

  it("pins the signed GitHub release feed and updater permissions", () => {
    const configUrl = new URL("../../src-tauri/tauri.conf.json", import.meta.url);
    const capabilitiesUrl = new URL("../../src-tauri/capabilities/default.json", import.meta.url);
    const config = tauriConfigSchema.parse(JSON.parse(readFileSync(configUrl, "utf8")));
    const capabilities = z
      .object({ permissions: z.array(z.union([z.string(), z.object({}).passthrough()])) })
      .parse(JSON.parse(readFileSync(capabilitiesUrl, "utf8")));

    expect(config.bundle.createUpdaterArtifacts).toBe(true);
    expect(config.plugins.updater.endpoints).toEqual([
      "https://github.com/bayurifkialghifari/lumadesk/releases/latest/download/latest.json",
    ]);
    expect(capabilities.permissions).toEqual(
      expect.arrayContaining(["updater:default", "process:allow-restart"]),
    );
    expect(capabilities.permissions).toContainEqual({
      identifier: "opener:allow-open-url",
      allow: expect.arrayContaining([
        { url: "https://*" },
        { url: "https://github.com/bayurifkialghifari/lumadesk/releases/latest" },
      ]),
    });
  });
});
