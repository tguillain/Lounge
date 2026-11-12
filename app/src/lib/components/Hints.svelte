<script lang="ts" module>
  export type HintAction = "navigate" | "confirm" | "back" | "options" | "search" | "menu" | "tabs" | "fullscreen";
  export type Hint = { action: HintAction; label: string };
</script>

<script lang="ts">
  import type { InputMode } from "$lib/input/gamepad-mapping";

  type Glyph =
    | { kind: "shape"; path: string }
    | { kind: "letter"; text: string }
    | { kind: "key"; text: string; round?: boolean }
    | { kind: "dpad" };

  const CROSS = "M6 6l12 12M18 6L6 18";
  const CIRCLE = "M5 12a7 7 0 1 0 14 0a7 7 0 1 0 -14 0";
  const TRIANGLE = "M12 4.5l8 14H4z";
  const SQUARE = "M6 6h12v12H6z";

  const GLYPHS: Record<InputMode, Partial<Record<HintAction, Glyph>>> = {
    playstation: {
      navigate: { kind: "dpad" },
      confirm: { kind: "shape", path: CROSS },
      back: { kind: "shape", path: CIRCLE },
      options: { kind: "shape", path: TRIANGLE },
      search: { kind: "shape", path: SQUARE },
      menu: { kind: "key", text: "Options" },
      tabs: { kind: "key", text: "L1 R1" },
    },
    xbox: {
      navigate: { kind: "dpad" },
      confirm: { kind: "letter", text: "A" },
      back: { kind: "letter", text: "B" },
      options: { kind: "letter", text: "Y" },
      search: { kind: "letter", text: "X" },
      menu: { kind: "letter", text: "≡" },
      tabs: { kind: "key", text: "LB RB" },
    },
    keyboard: {
      navigate: { kind: "key", text: "Arrows" },
      confirm: { kind: "key", text: "Enter" },
      back: { kind: "key", text: "Esc" },
      options: { kind: "key", text: "Right-click" },
      search: { kind: "key", text: "/" },
      menu: { kind: "key", text: "Tab" },
      tabs: { kind: "key", text: "Q E" },
      fullscreen: { kind: "key", text: "F11" },
    },
  };

  let { hints, mode }: { hints: Hint[]; mode: InputMode } = $props();

  const visible = $derived(
    hints.flatMap((hint) => {
      const glyph = GLYPHS[mode][hint.action];
      return glyph ? [{ ...hint, glyph }] : [];
    }),
  );
</script>

<footer class="hints" aria-label="Controls">
  {#each visible as hint (hint.action)}
    <span class="hint">
      {#if hint.glyph.kind === "shape"}
        <span class="button"><svg aria-hidden="true" viewBox="0 0 24 24"><path d={hint.glyph.path} /></svg></span>
      {:else if hint.glyph.kind === "letter"}
        <span class="button">{hint.glyph.text}</span>
      {:else if hint.glyph.kind === "dpad"}
        <span class="button dpad">
          <svg aria-hidden="true" viewBox="0 0 24 24"><path d="M9 3h6v6h6v6h-6v6H9v-6H3V9h6z" /></svg>
        </span>
      {:else}
        <kbd class:round={hint.glyph.round}>{hint.glyph.text}</kbd>
      {/if}
      {hint.label}
    </span>
  {/each}
</footer>

<style>
  .hints {
    display: flex;
    justify-content: flex-end;
    gap: 28px;
    font-size: 14px;
    color: var(--text-tertiary);
  }

  .hint {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .button {
    width: 24px;
    height: 24px;
    border-radius: 999px;
    border: 1.5px solid var(--line);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 11px;
    font-weight: 700;
    color: var(--text);
  }

  .button svg {
    width: 12px;
    height: 12px;
    fill: none;
    stroke: currentColor;
    stroke-width: 3;
    stroke-linecap: round;
    stroke-linejoin: round;
  }

  .dpad {
    border: 0;
  }

  .dpad svg {
    width: 20px;
    height: 20px;
    stroke-width: 1.6;
  }

  kbd {
    min-width: 24px;
    height: 24px;
    padding: 0 8px;
    border-radius: 6px;
    border: 1.5px solid var(--line);
    border-bottom-width: 3px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-family: inherit;
    font-size: 12px;
    font-weight: 600;
    color: var(--text);
  }

  kbd.round {
    border-radius: 999px;
  }
</style>
