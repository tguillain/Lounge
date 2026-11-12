<script lang="ts">
  import { KEYBOARD_ROWS, keyAt, type KeyPosition } from "$lib/search";

  type Props = {
    query: string;
    position: KeyPosition;
    showKeys: boolean;
    resultCount: number;
    onInput: (query: string) => void;
    onPressKey: (position: KeyPosition) => void;
  };

  let { query, position, showKeys, resultCount, onInput, onPressKey }: Props = $props();

  let field: HTMLInputElement | undefined = $state();
  const focusedKey = $derived(keyAt(position));

  $effect(() => {
    if (!showKeys) {
      field?.focus();
    }
  });

  const startOf = (row: number, index: number): number =>
    KEYBOARD_ROWS[row].slice(0, index).reduce((start, key) => start + key.span, 0);
</script>

<div class="search" role="dialog" aria-label="Search your library">
  <label class="field">
    <svg aria-hidden="true" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round">
      <circle cx="11" cy="11" r="7" /><path d="M20 20l-3.5-3.5" />
    </svg>
    <input
      bind:this={field}
      type="text"
      value={query}
      placeholder="Search your games"
      spellcheck="false"
      autocomplete="off"
      readonly={showKeys}
      oninput={(event) => onInput(event.currentTarget.value)}
    />
    <span class="count">{resultCount} {resultCount === 1 ? "game" : "games"}</span>
  </label>
  {#if showKeys}
    <div class="keys">
      {#each KEYBOARD_ROWS as row, rowIndex (rowIndex)}
        {#each row as key, keyIndex (key.id)}
          {@const column = startOf(rowIndex, keyIndex)}
          <button
            type="button"
            class="key {key.kind}"
            class:focused={rowIndex === position.row && key.id === focusedKey.id}
            style:grid-column="{column + 1} / span {key.span}"
            style:grid-row={rowIndex + 1}
            onclick={() => onPressKey({ row: rowIndex, column })}
          >
            {#if key.kind === "delete"}
              <svg aria-label="Delete" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M9 5h11v14H9l-6-7z" /><path d="M12.5 9.5l5 5M17.5 9.5l-5 5" />
              </svg>
            {:else}
              {key.label}
            {/if}
          </button>
        {/each}
      {/each}
    </div>
  {/if}
</div>

<style>
  .search {
    position: fixed;
    left: 50%;
    bottom: 72px;
    z-index: 20;
    width: min(760px, calc(100vw - 48px));
    padding: 18px;
    border-radius: 26px;
    background: rgba(28, 28, 32, 0.92);
    border: 1px solid var(--glass-border);
    box-shadow: 0 30px 80px rgba(0, 0, 0, 0.6);
    backdrop-filter: blur(24px);
    transform: translateX(-50%);
    animation: rise 0.22s var(--ease);
  }

  .field {
    display: flex;
    align-items: center;
    gap: 12px;
    height: 56px;
    padding: 0 18px;
    border-radius: 16px;
    background: rgba(255, 255, 255, 0.08);
    color: var(--text-tertiary);
  }

  input {
    flex: 1;
    min-width: 0;
    border: 0;
    outline: none;
    background: transparent;
    font: inherit;
    font-size: 22px;
    font-weight: 600;
    color: var(--text);
    caret-color: var(--text);
  }

  input::placeholder {
    color: var(--text-tertiary);
    font-weight: 500;
  }

  .count {
    flex-shrink: 0;
    font-size: 14px;
    font-variant-numeric: tabular-nums;
  }

  .keys {
    display: grid;
    grid-template-columns: repeat(10, 1fr);
    gap: 8px;
    margin-top: 14px;
  }

  .key {
    height: 52px;
    border: 0;
    border-radius: 12px;
    background: rgba(255, 255, 255, 0.07);
    display: flex;
    align-items: center;
    justify-content: center;
    font: inherit;
    font-size: 18px;
    font-weight: 600;
    color: var(--text);
    cursor: pointer;
    transition:
      background 0.12s ease,
      color 0.12s ease,
      transform 0.12s var(--ease);
  }

  .key.space,
  .key.done {
    font-size: 15px;
  }

  .key.done {
    background: rgba(255, 255, 255, 0.16);
  }

  .key.focused {
    background: var(--text);
    color: var(--bg);
    transform: scale(1.06);
  }

  .key:focus-visible {
    outline: none;
  }

  @keyframes rise {
    from {
      opacity: 0;
      transform: translate(-50%, 16px);
    }
  }
</style>
