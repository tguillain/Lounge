<script lang="ts">
  import { sectionsOf, type SettingsRow } from "$lib/settings";

  type Props = {
    rows: SettingsRow[];
    focusedIndex: number;
    note: string | null;
    onFocus: (index: number) => void;
    onActivate: (row: SettingsRow) => void;
  };

  let { rows, focusedIndex, note, onFocus, onActivate }: Props = $props();

  const groups = $derived(sectionsOf(rows));
  const buttons: HTMLButtonElement[] = [];

  $effect(() => {
    buttons[focusedIndex]?.scrollIntoView({ block: "nearest", behavior: "smooth" });
  });
</script>

<section class="settings" aria-label="Settings">
  <h1>Settings</h1>
  <div class="scroller">
    {#each groups as group (group.section)}
      <h2>{group.section}</h2>
      <div class="group">
        {#each group.rows as { row, index } (row.id)}
          <button
            type="button"
            class="row"
            class:focused={index === focusedIndex}
            class:danger={row.kind === "action" && row.danger}
            class:unavailable={row.kind === "toggle" && !row.available}
            role={row.kind === "toggle" ? "switch" : undefined}
            aria-checked={row.kind === "toggle" ? row.on : undefined}
            bind:this={buttons[index]}
            onpointerenter={() => onFocus(index)}
            onclick={() => onActivate(row)}
          >
            <span class="text">
              <span class="label">{row.label}</span>
              <span class="description">{row.description}</span>
            </span>
            {#if row.kind === "toggle"}
              <span class="switch" class:on={row.on} aria-hidden="true"><span class="knob"></span></span>
            {:else}
              <span class="value">
                {row.value ?? ""}
                {#if row.action === "releases"}
                  <svg aria-hidden="true" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round"><path d="M7 17L17 7M8 7h9v9" /></svg>
                {:else if !row.danger}
                  <svg aria-hidden="true" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round"><path d="M9 6l6 6-6 6" /></svg>
                {/if}
              </span>
            {/if}
          </button>
        {/each}
      </div>
    {/each}
    {#if note}
      <p class="note" role="status">{note}</p>
    {/if}
  </div>
</section>

<style>
  .settings {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    gap: 12px;
    margin-top: 36px;
    width: 100%;
    max-width: 760px;
    align-self: center;
  }

  h1 {
    margin: 0;
    font-size: 44px;
    font-weight: 700;
    letter-spacing: -0.03em;
  }

  .scroller {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    margin: 0 -16px;
    padding: 8px 16px 48px;
    scrollbar-width: none;
  }

  .scroller::-webkit-scrollbar {
    display: none;
  }

  h2 {
    margin: 28px 4px 10px;
    font-size: 14px;
    font-weight: 600;
    color: var(--text-tertiary);
  }

  .group {
    border-radius: 18px;
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.08);
    overflow: hidden;
  }

  .row {
    width: 100%;
    min-height: 72px;
    padding: 14px 20px;
    border: 0;
    background: transparent;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 24px;
    text-align: left;
    cursor: pointer;
    scroll-margin: 24px;
    transition:
      background 0.15s ease,
      color 0.15s ease;
  }

  .row + .row {
    border-top: 1px solid rgba(255, 255, 255, 0.08);
  }

  .row:focus-visible {
    outline: none;
  }

  .row.focused {
    background: var(--text);
    color: var(--bg);
  }

  .row.focused + .row {
    border-top-color: transparent;
  }

  .text {
    display: flex;
    flex-direction: column;
    gap: 3px;
    min-width: 0;
  }

  .label {
    font-size: 17px;
    font-weight: 500;
  }

  .description {
    font-size: 13px;
    color: var(--text-tertiary);
  }

  .row.focused .description {
    color: rgba(10, 10, 12, 0.6);
  }

  .row.danger .label {
    color: #ff6961;
  }

  .row.danger.focused .label {
    color: #d70015;
  }

  .row.unavailable .label {
    opacity: 0.5;
  }

  .value {
    flex-shrink: 0;
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 15px;
    color: var(--text-tertiary);
  }

  .row.focused .value {
    color: rgba(10, 10, 12, 0.6);
  }

  .switch {
    flex-shrink: 0;
    position: relative;
    width: 52px;
    height: 32px;
    border-radius: 999px;
    background: rgba(120, 120, 128, 0.36);
    transition: background 0.2s ease;
  }

  .row.focused .switch {
    background: rgba(120, 120, 128, 0.32);
  }

  .switch.on,
  .row.focused .switch.on {
    background: var(--live);
  }

  .row.unavailable .switch {
    opacity: 0.4;
  }

  .knob {
    position: absolute;
    top: 3px;
    left: 3px;
    width: 26px;
    height: 26px;
    border-radius: 999px;
    background: #fff;
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.3);
    transition: transform 0.2s var(--ease);
  }

  .switch.on .knob {
    transform: translateX(20px);
  }

  .note {
    margin: 20px 4px 0;
    font-size: 14px;
    color: var(--text-tertiary);
  }
</style>
