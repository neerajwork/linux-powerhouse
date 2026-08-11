import { describe, expect, it } from "vitest";

describe("System Health history", () => {
  it("bounds retained snapshots to the latest 60", () => {
    const history = Array.from({ length: 65 }, (_, timestamp) => ({ timestamp }));
    const bounded = history.slice(-60);
    expect(bounded).toHaveLength(60);
    expect(bounded[0].timestamp).toBe(5);
    expect(bounded.at(-1)?.timestamp).toBe(64);
  });

  it("classifies anomaly direction from first and latest snapshots", () => {
    const classify = (first: number, latest: number) => latest < first ? "Improving" : latest > first ? "Worsening" : "Stable";
    expect(classify(4, 1)).toBe("Improving");
    expect(classify(1, 4)).toBe("Worsening");
    expect(classify(2, 2)).toBe("Stable");
  });
});
