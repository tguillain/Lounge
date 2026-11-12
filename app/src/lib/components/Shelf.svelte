<script lang="ts">
  import { untrack } from "svelte";
  import GameTile from "./GameTile.svelte";
  import { SHELF_GAP, firstVisibleCard, shelfLayout } from "$lib/shelf";
  import type { Game } from "$lib/types";

  type Props = {
    games: Game[];
    selectedIndex: number;
    runningGameId: string | null;
    isBroken: (url: string) => boolean;
    onBroken: (url: string) => void;
    onSelect: (index: number) => void;
    onLaunch: (game: Game) => void;
    onOptions: (index: number) => void;
  };

  type DragState = {
    pointerId: number;
    startX: number;
    startOffset: number;
    lastX: number;
    lastTime: number;
    velocity: number;
    moved: boolean;
  };

  const DRAG_THRESHOLD_PX = 6;
  const GLIDE_MS = 180;

  let { games, selectedIndex, runningGameId, isBroken, onBroken, onSelect, onLaunch, onOptions }: Props = $props();

  let viewportWidth = $state(0);
  let offset = $state(0);
  let dragging = $state(false);
  let drag: DragState | null = null;
  let suppressNextClick = false;

  const layout = $derived(shelfLayout(viewportWidth));
  const maxOffset = () => Math.max(0, (games.length - layout.fullCards) * layout.step);
  const clampOffset = (value: number) => Math.round(Math.min(maxOffset(), Math.max(0, value)));

  $effect(() => {
    const index = selectedIndex;
    const { fullCards, step } = layout;
    if (viewportWidth === 0 || step === 0) {
      return;
    }
    untrack(() => {
      const first = firstVisibleCard(Math.round(offset / step), index, fullCards);
      offset = Math.round(first * step);
    });
  });

  const handlePointerDown = (event: PointerEvent) => {
    if (event.button !== 0) {
      return;
    }
    suppressNextClick = false;
    drag = {
      pointerId: event.pointerId,
      startX: event.clientX,
      startOffset: offset,
      lastX: event.clientX,
      lastTime: event.timeStamp,
      velocity: 0,
      moved: false,
    };
  };

  const handlePointerMove = (event: PointerEvent) => {
    if (!drag || event.pointerId !== drag.pointerId) {
      return;
    }
    const distance = event.clientX - drag.startX;
    if (!drag.moved) {
      if (Math.abs(distance) < DRAG_THRESHOLD_PX) {
        return;
      }
      drag.moved = true;
      dragging = true;
      (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
    }
    const elapsed = Math.max(1, event.timeStamp - drag.lastTime);
    drag.velocity = (event.clientX - drag.lastX) / elapsed;
    drag.lastX = event.clientX;
    drag.lastTime = event.timeStamp;
    offset = clampOffset(drag.startOffset - distance);
  };

  const endDrag = (event: PointerEvent, glide: boolean) => {
    if (!drag || event.pointerId !== drag.pointerId) {
      return;
    }
    if (drag.moved) {
      suppressNextClick = true;
      dragging = false;
      if (glide) {
        offset = clampOffset(offset - drag.velocity * GLIDE_MS);
      }
    }
    drag = null;
  };

  const handleClickCapture = (event: MouseEvent) => {
    if (!suppressNextClick) {
      return;
    }
    suppressNextClick = false;
    event.stopPropagation();
    event.preventDefault();
  };

  const handleWheel = (event: WheelEvent) => {
    const delta = Math.abs(event.deltaX) > Math.abs(event.deltaY) ? event.deltaX : event.deltaY;
    offset = clampOffset(offset + delta);
  };
</script>

<div
  class="viewport"
  class:dragging
  role="group"
  aria-label="Game list"
  bind:clientWidth={viewportWidth}
  onpointerdown={handlePointerDown}
  onpointermove={handlePointerMove}
  onpointerup={(event) => endDrag(event, true)}
  onpointercancel={(event) => endDrag(event, false)}
  onclickcapture={handleClickCapture}
  onwheel={handleWheel}
>
  <div
    class="row"
    style:transform="translateX({layout.start - offset}px)"
    style:--tile-width="{layout.tileWidth}px"
    style:gap="{SHELF_GAP}px"
  >
    {#each games as game, index (game.id)}
      <GameTile
        {game}
        selected={index === selectedIndex}
        running={game.id === runningGameId}
        {isBroken}
        {onBroken}
        onSelect={() => onSelect(index)}
        onLaunch={() => onLaunch(game)}
        onOptions={() => onOptions(index)}
      />
    {/each}
  </div>
</div>

<style>
  .viewport {
    margin: 0 calc(-1 * var(--page-gutter, 28px));
    padding: 22px 0;
    overflow: hidden;
    cursor: grab;
    touch-action: pan-y;
  }

  .viewport.dragging,
  .viewport.dragging :global(*) {
    cursor: grabbing;
  }

  .row {
    display: flex;
    width: max-content;
    transition: transform 0.35s var(--ease);
  }

  .dragging .row {
    transition: none;
  }
</style>
