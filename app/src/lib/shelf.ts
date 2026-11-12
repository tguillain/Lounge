export const SHELF_GAP = 28;
export const PEEK = 0.2;

const FULL_CARD_STEPS: { minWidth: number; cards: number }[] = [
  { minWidth: 1500, cards: 4 },
  { minWidth: 1000, cards: 3 },
  { minWidth: 0, cards: 2 },
];

export type ShelfLayout = { fullCards: number; tileWidth: number; step: number; start: number };

export const shelfLayout = (width: number): ShelfLayout => {
  const fullCards = FULL_CARD_STEPS.find((step) => width >= step.minWidth)?.cards ?? 2;
  const tileWidth = Math.max(0, (width - SHELF_GAP * (fullCards + 1)) / (fullCards + 2 * PEEK));
  return { fullCards, tileWidth, step: tileWidth + SHELF_GAP, start: SHELF_GAP + PEEK * tileWidth };
};

export const firstVisibleCard = (first: number, selected: number, fullCards: number): number => {
  if (selected < first) {
    return selected;
  }
  if (selected > first + fullCards - 1) {
    return selected - fullCards + 1;
  }
  return first;
};
