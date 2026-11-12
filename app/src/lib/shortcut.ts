export type KeyPress = {
  key: string;
  code: string;
  ctrlKey: boolean;
  altKey: boolean;
  shiftKey: boolean;
  metaKey: boolean;
};

export type CaptureResult =
  | { kind: "waiting" }
  | { kind: "cancel" }
  | { kind: "clear" }
  | { kind: "invalid"; message: string }
  | { kind: "shortcut"; shortcut: string };

const MODIFIER_KEYS = new Set(["Control", "Shift", "Alt", "AltGraph", "Meta", "OS"]);
const WORKS_ALONE = /^(F([1-9]|1[01]|1[3-9]|2[0-4])|Pause|ScrollLock|Insert|Home|End|PageUp|PageDown)$/;
const NEEDS_MODIFIER = /^(Key[A-Z]|Digit[0-9]|Numpad[0-9]|Space|Enter|Tab|Arrow(Up|Down|Left|Right))$/;
const RESERVED = new Set(["Alt+Tab", "Alt+F4", "Alt+Enter", "Alt+Space"]);

const KEY_NAMES: Record<string, string> = {
  PageUp: "Page Up",
  PageDown: "Page Down",
  ScrollLock: "Scroll Lock",
  ArrowUp: "↑",
  ArrowDown: "↓",
  ArrowLeft: "←",
  ArrowRight: "→",
};

const keyName = (part: string): string => {
  const letterOrDigit = /^(?:Key|Digit)(.)$/.exec(part);
  if (letterOrDigit) {
    return letterOrDigit[1];
  }
  const numpad = /^Numpad([0-9])$/.exec(part);
  if (numpad) {
    return `Num ${numpad[1]}`;
  }
  return KEY_NAMES[part] ?? part;
};

export const shortcutLabel = (shortcut: string): string => shortcut.split("+").map(keyName).join(" + ");

const mainKeyOf = (press: KeyPress): string | null => {
  if (/^Key[A-Z]$/.test(press.code)) {
    return /^[a-z]$/i.test(press.key) ? `Key${press.key.toUpperCase()}` : press.code;
  }
  if (press.code === "F12") {
    return null;
  }
  return WORKS_ALONE.test(press.code) || NEEDS_MODIFIER.test(press.code) ? press.code : null;
};

export const captureShortcut = (press: KeyPress): CaptureResult => {
  if (MODIFIER_KEYS.has(press.key)) {
    return { kind: "waiting" };
  }
  const withModifier = press.ctrlKey || press.altKey || press.shiftKey;
  if (press.key === "Escape" && !withModifier) {
    return { kind: "cancel" };
  }
  if ((press.key === "Backspace" || press.key === "Delete") && !withModifier) {
    return { kind: "clear" };
  }
  if (press.metaKey) {
    return { kind: "invalid", message: "Windows keeps the Windows key for its own shortcuts. Pick another combination." };
  }
  if (press.code === "F12") {
    return { kind: "invalid", message: "Windows keeps F12 for itself. Pick another key." };
  }
  const mainKey = mainKeyOf(press);
  if (!mainKey) {
    return { kind: "invalid", message: "Pick a letter, a number or a function key." };
  }
  const modifiers = [press.ctrlKey && "Ctrl", press.altKey && "Alt", press.shiftKey && "Shift"].filter(
    (modifier): modifier is string => Boolean(modifier),
  );
  if (!WORKS_ALONE.test(mainKey) && !press.ctrlKey && !press.altKey) {
    return { kind: "invalid", message: "Add Ctrl or Alt, so the key keeps working in your games." };
  }
  const shortcut = [...modifiers, mainKey].join("+");
  if (RESERVED.has(shortcut)) {
    return { kind: "invalid", message: `Windows or your games already use ${shortcutLabel(shortcut)}. Pick another combination.` };
  }
  return { kind: "shortcut", shortcut };
};
