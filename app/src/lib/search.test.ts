import { describe, expect, it } from "vitest";
import { FIRST_KEY, KEYBOARD_COLUMNS, KEYBOARD_ROWS, keyAt, matchesSearch, moveOnKeyboard, typeKey } from "./search";

describe("matchesSearch", () => {
  it("matches any part of the name, ignoring case and accents", () => {
    expect(matchesSearch("BeamNG.drive", "drive")).toBe(true);
    expect(matchesSearch("Pokémon Legends", "pokemon")).toBe(true);
    expect(matchesSearch("Counter-Strike 2", "COUNTER")).toBe(true);
  });

  it("ignores punctuation and spaces", () => {
    expect(matchesSearch("Counter-Strike 2", "counterstrike")).toBe(true);
    expect(matchesSearch("Brick by Brick: Tidy Up the Small Town!", "tidy up the")).toBe(true);
  });

  it("matches initials, keeping numbers whole", () => {
    expect(matchesSearch("Counter-Strike 2", "cs2")).toBe(true);
    expect(matchesSearch("Call of Duty®", "cod")).toBe(true);
    expect(matchesSearch("Call of Duty®", "cs")).toBe(false);
  });

  it("keeps every game for an empty search", () => {
    expect(matchesSearch("Anything", "  ")).toBe(true);
  });
});

describe("the on-screen keyboard", () => {
  it("fills every row to the same width", () => {
    for (const row of KEYBOARD_ROWS) {
      expect(row.reduce((width, key) => width + key.span, 0)).toBe(KEYBOARD_COLUMNS);
    }
  });

  it("starts on Q", () => {
    expect(keyAt(FIRST_KEY).id).toBe("q");
  });

  it("moves between keys and stops at the edges", () => {
    expect(keyAt(moveOnKeyboard(FIRST_KEY, "right")).id).toBe("w");
    expect(moveOnKeyboard(FIRST_KEY, "left")).toEqual(FIRST_KEY);
    expect(keyAt(moveOnKeyboard(FIRST_KEY, "up")).id).toBe("1");
  });

  it("lands on the wide space key from either column it covers", () => {
    expect(keyAt({ row: 3, column: 7 }).id).toBe("space");
    expect(keyAt({ row: 3, column: 8 }).id).toBe("space");
    const fromK = moveOnKeyboard({ row: 2, column: 7 }, "down");
    expect(keyAt(fromK).id).toBe("space");
    expect(keyAt(moveOnKeyboard(fromK, "right")).id).toBe("done");
  });

  it("types, adds single spaces and deletes", () => {
    const space = KEYBOARD_ROWS[3].find((key) => key.kind === "space")!;
    const remove = KEYBOARD_ROWS[2].find((key) => key.kind === "delete")!;
    expect(typeKey("co", keyAt(FIRST_KEY))).toBe("coq");
    expect(typeKey("", space)).toBe("");
    expect(typeKey("call", space)).toBe("call ");
    expect(typeKey("call ", space)).toBe("call ");
    expect(typeKey("cod", remove)).toBe("co");
  });
});
