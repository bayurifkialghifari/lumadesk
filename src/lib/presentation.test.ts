import { describe, expect, it } from "vitest";
import { filenameFromPath } from "./presentation";

describe("presentation helpers", () => {
  it("extracts file names from Unix and Windows paths", () => {
    expect(filenameFromPath("/home/user/loop.gif")).toBe("loop.gif");
    expect(filenameFromPath("C:\\Users\\user\\loop.gif")).toBe("loop.gif");
  });
});
