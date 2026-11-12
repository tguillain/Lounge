import { describe, expect, it } from "vitest";
import { currentOutputLabel, markCurrent, outputTitle } from "./audio";
import type { AudioOutput, AudioOutputKind } from "./types";

const output = (id: string, name: string, detail: string | null, current = false, kind: AudioOutputKind = "speakers"): AudioOutput => ({
  id,
  name,
  detail,
  kind,
  current,
});

describe("sound outputs", () => {
  it("names an output by its device, or a TV by its own name", () => {
    expect(outputTitle(output("a", "Speakers", "Scarlett 2i2 USB"))).toBe("Scarlett 2i2 USB");
    expect(outputTitle(output("a", "Living room TV", null))).toBe("Living room TV");
    expect(outputTitle(output("a", "LG TV SSCR2", "NVIDIA High Definition Audio", false, "display"))).toBe("LG TV SSCR2");
  });

  it("shows which output is in use", () => {
    const outputs = [output("a", "Speakers", "Scarlett 2i2 USB"), output("b", "LG TV", null, true)];
    expect(currentOutputLabel(outputs)).toBe("LG TV");
    expect(currentOutputLabel([])).toBe("None");
    expect(currentOutputLabel(null)).toBeUndefined();
  });

  it("moves the current mark to the chosen output", () => {
    const outputs = markCurrent([output("a", "A", null, true), output("b", "B", null)], "b");
    expect(outputs.map((entry) => entry.current)).toEqual([false, true]);
  });
});
