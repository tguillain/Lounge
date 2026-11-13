<script lang="ts">
  import { imageUrl } from "$lib/api";
  import DownloadProgress from "./DownloadProgress.svelte";
  import { placeholderArt } from "$lib/format";
  import type { Game } from "$lib/types";

  type Props = {
    game: Game;
    selected: boolean;
    running: boolean;
    isBroken: (url: string) => boolean;
    onBroken: (url: string) => void;
    onSelect: () => void;
    onLaunch: () => void;
    onOptions: () => void;
  };

  let { game, selected, running, isBroken, onBroken, onSelect, onLaunch, onOptions }: Props = $props();

  const header = $derived(imageUrl(game.images.header));
  const showHeader = $derived(header !== null && !isBroken(header));

  const handleClick = () => (selected ? onLaunch() : onSelect());

  const handleContextMenu = (event: MouseEvent) => {
    event.preventDefault();
    onOptions();
  };
</script>

<button
  type="button"
  class="tile"
  class:selected
  class:hidden-game={game.hidden}
  aria-label={game.hidden ? `${game.name} (hidden)` : game.name}
  aria-pressed={selected}
  style:background={showHeader ? undefined : placeholderArt(game.name)}
  onclick={handleClick}
  oncontextmenu={handleContextMenu}
>
  {#if showHeader && header}
    <img src={header} alt="" draggable="false" decoding="async" onerror={() => onBroken(header)} />
  {:else}
    <span class="name">{game.name}</span>
  {/if}
  {#if running}
    <span class="badge live"><span class="live-dot"></span>Now playing</span>
  {:else if game.download}
    <DownloadProgress download={game.download} />
  {:else if game.hidden}
    <span class="badge">Hidden</span>
  {/if}
</button>

<style>
  .tile {
    position: relative;
    flex: 0 0 auto;
    width: var(--tile-width, 288px);
    aspect-ratio: 460 / 215;
    padding: 0;
    border: 0;
    border-radius: var(--radius-tile);
    overflow: hidden;
    cursor: pointer;
    display: flex;
    align-items: flex-end;
    background: #1c1c1e;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
    transition:
      transform 0.25s var(--ease),
      box-shadow 0.25s var(--ease),
      opacity 0.25s var(--ease);
  }

  .tile.selected {
    z-index: 1;
    transform: scale(1.08);
    box-shadow:
      0 0 0 3px var(--bg),
      0 0 0 6px var(--text),
      0 24px 48px rgba(0, 0, 0, 0.6);
  }

  .tile.hidden-game {
    opacity: 0.45;
  }

  .tile.hidden-game.selected {
    opacity: 0.8;
  }

  .tile:focus-visible {
    outline: none;
  }

  img {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .name {
    position: relative;
    padding: 14px 16px;
    font-size: 15px;
    font-weight: 800;
    line-height: 1.1;
    letter-spacing: 0.06em;
    text-align: left;
    text-transform: uppercase;
    color: #fff;
    text-shadow: 0 2px 12px rgba(0, 0, 0, 0.5);
  }

  .badge {
    position: absolute;
    top: 10px;
    left: 10px;
    padding: 4px 10px;
    border-radius: 999px;
    background: rgba(10, 10, 12, 0.8);
    font-size: 12px;
    font-weight: 600;
    color: var(--text);
  }

  .badge.live {
    display: flex;
    align-items: center;
    gap: 6px;
    color: var(--live-text);
  }

  .live-dot {
    width: 6px;
    height: 6px;
    border-radius: 999px;
    background: var(--live);
  }
</style>
