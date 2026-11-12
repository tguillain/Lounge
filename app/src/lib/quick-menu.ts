export type QuickItem = "settings" | "lounge" | "quit" | "mute" | "output" | "volume" | "sleep" | "restart" | "shut-down";

export type Spot = { row: number; column: number };

export const QUICK_ROWS: QuickItem[][] = [
  ["settings"],
  ["lounge", "quit"],
  ["mute", "output"],
  ["volume"],
  ["sleep", "restart", "shut-down"],
];

export const FIRST_SPOT: Spot = { row: 1, column: 0 };

const VOLUME_STEP = 5;

export const itemAt = (spot: Spot): QuickItem => QUICK_ROWS[spot.row][spot.column];

export const spotOf = (item: QuickItem): Spot => {
  const row = QUICK_ROWS.findIndex((items) => items.includes(item));
  return { row, column: QUICK_ROWS[row].indexOf(item) };
};

const matchingColumn = (column: number, from: number, to: number): number =>
  from <= 1 || to <= 1 ? Math.min(column, to - 1) : Math.round((column * (to - 1)) / (from - 1));

export const moveSpot = (spot: Spot, direction: "up" | "down" | "left" | "right"): Spot => {
  const items = QUICK_ROWS[spot.row];
  if (direction === "left" || direction === "right") {
    const column = Math.max(0, Math.min(items.length - 1, spot.column + (direction === "right" ? 1 : -1)));
    return { row: spot.row, column };
  }
  const row = Math.max(0, Math.min(QUICK_ROWS.length - 1, spot.row + (direction === "down" ? 1 : -1)));
  if (row === spot.row) {
    return spot;
  }
  return { row, column: matchingColumn(spot.column, items.length, QUICK_ROWS[row].length) };
};

export const stepVolume = (level: number, direction: "left" | "right"): number => {
  const snapped = direction === "right" ? Math.floor(level / VOLUME_STEP) + 1 : Math.ceil(level / VOLUME_STEP) - 1;
  return Math.max(0, Math.min(100, snapped * VOLUME_STEP));
};
