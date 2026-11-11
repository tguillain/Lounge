<script lang="ts" module>
  export type MenuItem = { id: string; label: string; run: () => void };
</script>

<script lang="ts">
  import Hints from "./Hints.svelte";
  import type { InputMode } from "$lib/input/gamepad-mapping";

  type Props = {
    title: string;
    items: MenuItem[];
    focusedIndex: number;
    mode: InputMode;
    onFocus: (index: number) => void;
    onClose: () => void;
  };

  let { title, items, focusedIndex, mode, onFocus, onClose }: Props = $props();
</script>

<div class="backdrop" role="presentation" onclick={onClose}></div>
<div class="sheet" role="menu" aria-label={`Options — ${title}`}>
  <p class="title">{title}</p>
  {#each items as item, index (item.id)}
    <button
      type="button"
      role="menuitem"
      class:focused={index === focusedIndex}
      onpointerenter={() => onFocus(index)}
      onclick={item.run}
    >
      {item.label}
    </button>
  {/each}
  <div class="hints">
    <Hints
      {mode}
      hints={[
        { action: "confirm", label: "Select" },
        { action: "back", label: "Close" },
      ]}
    />
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 20;
    background: rgba(10, 10, 12, 0.55);
    animation: appear 0.2s var(--ease);
  }

  .sheet {
    position: fixed;
    z-index: 21;
    left: 50%;
    top: 50%;
    width: 420px;
    padding: 20px;
    border-radius: 24px;
    background: #1c1c20;
    border: 1px solid rgba(255, 255, 255, 0.12);
    box-shadow: 0 30px 80px rgba(0, 0, 0, 0.55);
    display: flex;
    flex-direction: column;
    gap: 6px;
    transform: translate(-50%, -50%);
    animation: rise 0.22s var(--ease);
  }

  .title {
    margin: 0 4px 10px;
    font-size: 15px;
    font-weight: 600;
    color: var(--text-tertiary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  button {
    height: 52px;
    padding: 0 16px;
    border: 0;
    border-radius: 14px;
    background: transparent;
    font-size: 16px;
    font-weight: 500;
    text-align: left;
    cursor: pointer;
    transition:
      background 0.15s ease,
      color 0.15s ease;
  }

  button.focused {
    background: var(--text);
    color: var(--bg);
    font-weight: 600;
  }

  .hints {
    margin-top: 12px;
  }

  @keyframes appear {
    from {
      opacity: 0;
    }
  }

  @keyframes rise {
    from {
      opacity: 0;
      transform: translate(-50%, -46%);
    }
  }
</style>
