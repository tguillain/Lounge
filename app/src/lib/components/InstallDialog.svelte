<script lang="ts">
  import { imageUrl } from "$lib/api";
  import { placeholderArt, sizeLabel } from "$lib/format";
  import type { Game, SteamLibrary } from "$lib/types";

  type Props = {
    game: Game;
    libraries: SteamLibrary[] | null;
    focusedIndex: number;
    isBroken: (url: string) => boolean;
    onBroken: (url: string) => void;
    onFocus: (index: number) => void;
    onContinue: () => void;
    onCancel: () => void;
  };

  let { game, libraries, focusedIndex, isBroken, onBroken, onFocus, onContinue, onCancel }: Props = $props();

  const header = $derived(imageUrl(game.images.header));
  const art = $derived(header && !isBroken(header) ? header : null);
  const usedShare = (library: SteamLibrary): number =>
    library.totalBytes > 0 ? Math.round(((library.totalBytes - library.freeBytes) / library.totalBytes) * 100) : 0;
</script>

<div class="backdrop" role="presentation" onclick={onCancel}></div>
<div class="dialog" role="dialog" aria-modal="true" aria-labelledby="install-title" tabindex="-1">
  <div class="heading">
    <span class="art" style:background={art ? undefined : placeholderArt(game.name)}>
      {#if art}
        <img src={art} alt="" onerror={() => onBroken(art)} />
      {/if}
    </span>
    <div>
      <span class="eyebrow">Install</span>
      <h2 id="install-title">{game.name}</h2>
    </div>
  </div>

  <div class="libraries">
    <span class="label">Your Steam libraries</span>
    {#if libraries === null}
      <span class="library loading" aria-busy="true"></span>
    {:else if libraries.length === 0}
      <p class="note">Lounge couldn't read your Steam libraries. Steam still lets you pick one.</p>
    {:else}
      {#each libraries as library (library.path)}
        <div class="library">
          <div class="library-line">
            <span class="path">{library.path}</span>
            <span class="free">
              {#if library.totalBytes > 0}
                {sizeLabel(library.freeBytes) ?? "0 GB"} free of {sizeLabel(library.totalBytes)}
              {/if}
            </span>
          </div>
          <span class="track"><span class="fill" style:width="{usedShare(library)}%"></span></span>
        </div>
      {/each}
    {/if}
    <p class="note">Steam asks which drive to use and shows the game's size. Lounge comes back on its own once the download starts.</p>
  </div>

  <div class="buttons">
    <button type="button" class="primary" class:focused={focusedIndex === 0} onpointerenter={() => onFocus(0)} onclick={onContinue}>
      <svg aria-hidden="true" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"><path d="M7 18a4.5 4.5 0 0 1-.5-9A6 6 0 0 1 18 8a4 4 0 0 1 0 8" /><path d="M12 12v8M9 17l3 3 3-3" /></svg>
      Continue in Steam
    </button>
    <button type="button" class="secondary" class:focused={focusedIndex === 1} onpointerenter={() => onFocus(1)} onclick={onCancel}>Cancel</button>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 30;
    background: rgba(5, 5, 7, 0.62);
  }

  .dialog {
    position: fixed;
    z-index: 31;
    left: 50%;
    top: 50%;
    width: min(620px, calc(100vw - 32px));
    max-height: calc(100vh - 48px);
    overflow-y: auto;
    padding: 30px;
    box-sizing: border-box;
    border-radius: 26px;
    background: #1c1c20;
    border: 1px solid rgba(255, 255, 255, 0.12);
    box-shadow: 0 30px 80px rgba(0, 0, 0, 0.6);
    transform: translate(-50%, -50%);
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  .heading {
    display: flex;
    align-items: center;
    gap: 16px;
  }

  .art {
    flex: 0 0 auto;
    width: 112px;
    height: 52px;
    border-radius: 10px;
    overflow: hidden;
  }

  .art img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .eyebrow {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-tertiary);
  }

  h2 {
    margin: 2px 0 0;
    font-size: 26px;
    font-weight: 700;
    letter-spacing: -0.02em;
  }

  .libraries {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .label {
    font-size: 14px;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .library {
    padding: 14px 16px;
    border-radius: 16px;
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.1);
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .library.loading {
    height: 52px;
    animation: pulse 1.4s ease-in-out infinite;
  }

  .library-line {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 12px;
  }

  .path {
    min-width: 0;
    font-size: 15px;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .free {
    flex-shrink: 0;
    font-size: 14px;
    color: var(--text-secondary);
    font-variant-numeric: tabular-nums;
  }

  .track {
    height: 6px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.1);
    overflow: hidden;
  }

  .fill {
    display: block;
    height: 100%;
    background: rgba(255, 255, 255, 0.45);
  }

  .note {
    margin: 0;
    font-size: 13px;
    line-height: 1.45;
    color: var(--text-tertiary);
  }

  .buttons {
    display: flex;
    gap: 10px;
  }

  .buttons button {
    height: 52px;
    padding: 0 24px;
    border-radius: 999px;
    font-size: 16px;
    font-weight: 600;
    display: flex;
    align-items: center;
    gap: 10px;
    cursor: pointer;
    transition: box-shadow 0.15s ease;
  }

  .primary {
    border: 0;
    background: var(--text);
    color: var(--bg);
  }

  .secondary {
    border: 1px solid rgba(255, 255, 255, 0.16);
    background: rgba(255, 255, 255, 0.08);
    color: var(--text);
  }

  .buttons button.focused {
    box-shadow:
      0 0 0 3px #1c1c20,
      0 0 0 5.5px var(--text);
  }

  .buttons button:focus-visible {
    outline: none;
  }

  @keyframes pulse {
    50% {
      opacity: 0.5;
    }
  }
</style>
