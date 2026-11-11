<script lang="ts">
  import { imageUrl } from "$lib/api";
  import { placeholderArt } from "$lib/format";
  import { SORT_LABELS, sortDetail, type FilterOption, type LibraryFilter, type LibrarySort } from "$lib/library";
  import type { Game } from "$lib/types";

  type Props = {
    games: Game[];
    filters: FilterOption[];
    activeFilter: LibraryFilter;
    sort: LibrarySort;
    focusedIndex: number | null;
    focusedControl: number | null;
    runningGameId: string | null;
    now: Date;
    isBroken: (url: string) => boolean;
    onBroken: (url: string) => void;
    onFilter: (filter: LibraryFilter) => void;
    onCycleSort: () => void;
    onFocusGame: (index: number) => void;
    onActivate: (game: Game) => void;
    onOptions: (game: Game) => void;
    onColumns: (columns: number) => void;
  };

  let {
    games,
    filters,
    activeFilter,
    sort,
    focusedIndex,
    focusedControl,
    runningGameId,
    now,
    isBroken,
    onBroken,
    onFilter,
    onCycleSort,
    onFocusGame,
    onActivate,
    onOptions,
    onColumns,
  }: Props = $props();

  let grid = $state<HTMLDivElement>();
  let gridWidth = $state(0);
  const cards: HTMLButtonElement[] = [];

  const coverUrl = (game: Game): string | null => {
    const url = imageUrl(game.images.cover);
    return url && !isBroken(url) ? url : null;
  };

  const detail = (game: Game): string =>
    game.id === runningGameId ? "Now playing" : sortDetail(game, sort, now.getTime() / 1000);

  const handleClick = (game: Game, index: number) => (index === focusedIndex ? onActivate(game) : onFocusGame(index));

  const handleContextMenu = (event: MouseEvent, game: Game, index: number) => {
    event.preventDefault();
    onFocusGame(index);
    onOptions(game);
  };

  $effect(() => {
    if (!grid || gridWidth === 0) {
      return;
    }
    onColumns(Math.max(1, getComputedStyle(grid).gridTemplateColumns.split(" ").length));
  });

  $effect(() => {
    if (focusedIndex !== null) {
      cards[focusedIndex]?.scrollIntoView({ block: "nearest", behavior: "smooth" });
    }
  });
</script>

<section class="library" aria-label="Library">
  <div class="toolbar">
    <div class="title">
      <h1>Library</h1>
      <span class="count">{games.length} {games.length === 1 ? "game" : "games"}</span>
    </div>
    <div class="controls" role="group" aria-label="Filter and sort">
      {#each filters as filter, index (filter.id)}
        <button
          type="button"
          class="chip"
          class:active={filter.id === activeFilter}
          class:focused={index === focusedControl}
          aria-pressed={filter.id === activeFilter}
          onclick={() => onFilter(filter.id)}
        >
          {filter.label}
        </button>
      {/each}
      <button type="button" class="chip sort" class:focused={focusedControl === filters.length} onclick={onCycleSort}>
        Sort: {SORT_LABELS[sort]}
      </button>
    </div>
  </div>

  {#if games.length === 0}
    <p class="empty">{activeFilter === "hidden" ? "You haven't hidden any games." : "No games here."}</p>
  {:else}
    <div class="scroller">
      <div class="grid" bind:this={grid} bind:clientWidth={gridWidth}>
        {#each games as game, index (game.id)}
          {@const cover = coverUrl(game)}
          <button
            type="button"
            class="card"
            class:focused={index === focusedIndex}
            class:hidden-game={game.hidden}
            bind:this={cards[index]}
            aria-label={game.name}
            onclick={() => handleClick(game, index)}
            oncontextmenu={(event) => handleContextMenu(event, game, index)}
          >
            <span class="cover" style:background={cover ? undefined : placeholderArt(game.name)}>
              {#if cover}
                <img src={cover} alt="" draggable="false" loading="lazy" decoding="async" onerror={() => onBroken(cover)} />
              {:else}
                <span class="cover-name">{game.name}</span>
              {/if}
              {#if game.id === runningGameId}
                <span class="badge"><span class="live-dot"></span>Now playing</span>
              {/if}
            </span>
            <span class="name">{game.name}</span>
            <span class="detail" class:live={game.id === runningGameId}>{detail(game)}</span>
          </button>
        {/each}
      </div>
    </div>
  {/if}
</section>

<style>
  .library {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    gap: 24px;
    margin-top: 36px;
  }

  .toolbar {
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    flex-wrap: wrap;
    gap: 16px 32px;
  }

  .title {
    display: flex;
    align-items: baseline;
    gap: 14px;
  }

  h1 {
    margin: 0;
    font-size: 44px;
    font-weight: 700;
    letter-spacing: -0.03em;
  }

  .count {
    font-size: 17px;
    color: var(--text-tertiary);
  }

  .controls {
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
  }

  .chip {
    height: 40px;
    padding: 0 18px;
    border-radius: 999px;
    border: 1px solid rgba(255, 255, 255, 0.12);
    background: rgba(255, 255, 255, 0.08);
    font-size: 15px;
    font-weight: 500;
    cursor: pointer;
    transition:
      background 0.15s ease,
      color 0.15s ease,
      box-shadow 0.15s ease;
  }

  .chip.active {
    background: var(--text);
    border-color: var(--text);
    color: var(--bg);
    font-weight: 600;
  }

  .chip.focused {
    box-shadow:
      0 0 0 3px var(--bg),
      0 0 0 5px var(--text);
  }

  .sort {
    margin-left: 12px;
  }

  .scroller {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    margin: 0 -28px;
    padding: 28px 28px 48px;
    scrollbar-width: none;
  }

  .scroller::-webkit-scrollbar {
    display: none;
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(176px, 1fr));
    gap: 36px 24px;
  }

  .card {
    padding: 0;
    border: 0;
    background: none;
    text-align: left;
    display: flex;
    flex-direction: column;
    gap: 18px;
    cursor: pointer;
    scroll-margin: 32px;
  }

  .card:focus-visible {
    outline: none;
  }

  .cover {
    position: relative;
    aspect-ratio: 2 / 3;
    border-radius: 14px;
    overflow: hidden;
    display: flex;
    align-items: flex-end;
    background: #1c1c1e;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
    transform-origin: center bottom;
    transition:
      transform 0.25s var(--ease),
      box-shadow 0.25s var(--ease);
  }

  .card.focused .cover {
    transform: scale(1.05);
    box-shadow:
      0 0 0 3px var(--bg),
      0 0 0 6px var(--text),
      0 20px 40px rgba(0, 0, 0, 0.55);
  }

  .card.hidden-game {
    opacity: 0.55;
  }

  .cover img {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .cover-name {
    position: relative;
    padding: 16px;
    font-size: 17px;
    font-weight: 800;
    line-height: 1.15;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: #fff;
  }

  .badge {
    position: absolute;
    top: 10px;
    left: 10px;
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 10px;
    border-radius: 999px;
    background: rgba(10, 10, 12, 0.8);
    font-size: 12px;
    font-weight: 600;
    color: var(--live-text);
  }

  .live-dot {
    width: 6px;
    height: 6px;
    border-radius: 999px;
    background: var(--live);
  }

  .name {
    font-size: 15px;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .detail {
    margin-top: -14px;
    font-size: 13px;
    color: var(--text-tertiary);
  }

  .detail.live {
    color: var(--live-text);
  }

  .empty {
    margin: 48px 0;
    font-size: 17px;
    color: var(--text-tertiary);
  }
</style>
