import type { PadAction } from "./input/gamepad-mapping";

export const normalize = (text: string): string =>
  text
    .normalize("NFD")
    .replace(/\p{Diacritic}/gu, "")
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, " ")
    .trim();

const initialsOf = (words: string[]): string => words.map((word) => (/^\d+$/.test(word) ? word : word[0])).join("");

export const matchesSearch = (name: string, query: string): boolean => {
  const wanted = normalize(query);
  if (!wanted) {
    return true;
  }
  const normalizedName = normalize(name);
  const compactWanted = wanted.replaceAll(" ", "");
  return (
    normalizedName.includes(wanted) ||
    normalizedName.replaceAll(" ", "").includes(compactWanted) ||
    initialsOf(normalizedName.split(" ")).startsWith(compactWanted)
  );
};

export type KeyboardKey =
  | { kind: "char"; id: string; label: string; value: string; span: 1 }
  | { kind: "space" | "delete" | "done"; id: string; label: string; span: number };

export type KeyPosition = { row: number; column: number };

const characters = (letters: string): KeyboardKey[] =>
  [...letters].map((letter) => ({ kind: "char", id: letter, label: letter.toUpperCase(), value: letter, span: 1 }));

export const KEYBOARD_COLUMNS = 10;

export const KEYBOARD_ROWS: KeyboardKey[][] = [
  characters("1234567890"),
  characters("qwertyuiop"),
  [...characters("asdfghjkl"), { kind: "delete", id: "delete", label: "Delete", span: 1 }],
  [
    ...characters("zxcvbnm"),
    { kind: "space", id: "space", label: "Space", span: 2 },
    { kind: "done", id: "done", label: "Done", span: 1 },
  ],
];

export const FIRST_KEY: KeyPosition = { row: 1, column: 0 };

const keyStart = (row: KeyboardKey[], index: number): number =>
  row.slice(0, index).reduce((start, key) => start + key.span, 0);

const keyIndexAt = (row: KeyboardKey[], column: number): number => {
  let start = 0;
  for (const [index, key] of row.entries()) {
    if (column < start + key.span) {
      return index;
    }
    start += key.span;
  }
  return row.length - 1;
};

export const keyAt = ({ row, column }: KeyPosition): KeyboardKey => {
  const keys = KEYBOARD_ROWS[row];
  return keys[keyIndexAt(keys, column)];
};

export const moveOnKeyboard = (position: KeyPosition, action: PadAction): KeyPosition => {
  if (action === "up" || action === "down") {
    const row = Math.max(0, Math.min(KEYBOARD_ROWS.length - 1, position.row + (action === "down" ? 1 : -1)));
    return { row, column: position.column };
  }
  if (action === "left" || action === "right") {
    const keys = KEYBOARD_ROWS[position.row];
    const index = Math.max(0, Math.min(keys.length - 1, keyIndexAt(keys, position.column) + (action === "right" ? 1 : -1)));
    return { row: position.row, column: keyStart(keys, index) };
  }
  return position;
};

export const typeKey = (query: string, key: KeyboardKey): string => {
  if (key.kind === "char") {
    return query + key.value;
  }
  if (key.kind === "space") {
    return query === "" || query.endsWith(" ") ? query : `${query} `;
  }
  if (key.kind === "delete") {
    return query.slice(0, -1);
  }
  return query;
};
