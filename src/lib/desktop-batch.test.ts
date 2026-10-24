import { describe, expect, it } from "vitest";
import { runSettledBatch } from "./desktop-batch";

describe("desktop overlay batch operations", () => {
  it("attempts every overlay and reports partial failure", async () => {
    const attempted: string[] = [];

    const outcome = await runSettledBatch(["first", "broken", "last"], async (id) => {
      attempted.push(id);
      if (id === "broken") throw new Error("cannot update overlay");
    });

    expect(attempted).toEqual(["first", "broken", "last"]);
    expect(outcome).toEqual({ total: 3, succeeded: 2, failed: 1 });
  });
});
