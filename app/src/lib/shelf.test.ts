import { describe, expect, it } from "vitest";
import { PEEK, SHELF_GAP, firstVisibleCard, shelfLayout } from "./shelf";

describe("shelfLayout", () => {
  it("shows more cards on wider screens", () => {
    expect(shelfLayout(1900).fullCards).toBe(4);
    expect(shelfLayout(1340).fullCards).toBe(3);
    expect(shelfLayout(900).fullCards).toBe(2);
  });

  it("centers the full cards, with the neighbours peeking out equally on both sides", () => {
    for (const width of [900, 1340, 1900, 2400]) {
      const { fullCards, tileWidth, step, start } = shelfLayout(width);
      const blockEnd = start + fullCards * step - SHELF_GAP;
      expect(width - blockEnd).toBeCloseTo(start, 6);
      expect((start - SHELF_GAP) / tileWidth).toBeCloseTo(PEEK, 6);
    }
  });
});

describe("firstVisibleCard", () => {
  it("keeps the row still while the selection stays in view", () => {
    expect(firstVisibleCard(0, 2, 4)).toBe(0);
    expect(firstVisibleCard(3, 5, 4)).toBe(3);
  });

  it("scrolls one card at a time past either edge", () => {
    expect(firstVisibleCard(0, 4, 4)).toBe(1);
    expect(firstVisibleCard(3, 2, 4)).toBe(2);
  });
});
