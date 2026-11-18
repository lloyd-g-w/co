import { describe, expect, it } from "vite-plus/test";
import { backoffDelay } from "./backoff.ts";

describe("backoffDelay", () => {
  it("grows exponentially and is capped", () => {
    const mid = { random: () => 0.5 };
    expect(backoffDelay(0, mid)).toBe(375);
    expect(backoffDelay(1, mid)).toBe(750);
    expect(backoffDelay(20, mid)).toBe(11_250);
  });
});
