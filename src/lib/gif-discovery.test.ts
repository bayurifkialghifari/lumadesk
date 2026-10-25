import { describe, expect, it } from "vitest";
import { gifDiscoveryUrl } from "./gif-discovery";

describe("gifDiscoveryUrl", () => {
  it("opens the Tenor home page when collection search is empty", () => {
    expect(gifDiscoveryUrl("   ").href).toBe("https://tenor.com/");
  });

  it("turns collection search into a Tenor GIF search path", () => {
    expect(gifDiscoveryUrl("  pixel   cat  ").href).toBe("https://tenor.com/search/pixel-cat-gifs");
  });

  it("encodes characters that are unsafe in a URL path", () => {
    expect(gifDiscoveryUrl("happy & dance").href).toBe(
      "https://tenor.com/search/happy-%26-dance-gifs",
    );
  });
});
