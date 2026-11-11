<script lang="ts">
  import { untrack } from "svelte";
  import GameTile from "./GameTile.svelte";
  import type { Game } from "$lib/types";

  type Props = {
    games: Game[];
    selectedIndex: number;
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

  const TILE_WIDTH = 288;
  const TILE_STEP = TILE_WIDTH + 28;
  const SIDE_PADDING = 28;
  const EDGE_MARGIN = 40;
  const DRAG_THRESHOLD_PX = 6;
  const GLIDE_MS = 180;

  let { games, selectedIndex, isBroken, onBroken, onSelect, onLaunch, onOptions }: Props = $props();

  let viewportWidth = $state(0);
  let row = $state<HTMLDivElement>();
  let offset = $state(0);
  let dragging = $state(false);
  let drag: DragState | null = null;
  let suppressNextClick = false;

  const visibleWidth = () => viewportWidth - 2 * SIDE_PADDING;
  const maxOffset = () => Math.max(0, (row?.scrollWidth ?? 0) - visibleWidth());
  const clampOffset = (value: number) => Math.round(Math.min(maxOffset(), Math.max(0, value)));

  $effect(() => {
    const index = selectedIndex;
    if (viewportWidth === 0) {
      return;
    }
    untrack(() => {
      const left = index * TILE_STEP;
      const right = left + TILE_WIDTH;
      if (left - EDGE_MARGIN < offset) {
        offset = clampOffset(left - EDGE_MARGIN);
      } else if (right + EDGE_MARGIN > offset + visibleWidth()) {
        offset = clampOffset(right + EDGE_MARGIN - visibleWidth());
      }
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
  aria-label="Liste des jeux"
  bind:clientWidth={viewportWidth}
  onpointerdown={handlePointerDown}
  onpointermove={handlePointerMove}
  onpointerup={(event) => endDrag(event, true)}
  onpointercancel={(event) => endDrag(event, false)}
  onclickcapture={handleClickCapture}
  onwheel={handleWheel}
>
  <div class="row" bind:this={row} style:transform="translateX(-{offset}px)">
    {#each games as game, index (game.id)}
      <GameTile
        {game}
        selected={index === selectedIndex}
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
    margin: 0 -28px;
    padding: 22px 28px;
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
    gap: 28px;
    width: max-content;
    transition: transform 0.35s var(--ease);
  }

  .dragging .row {
    transition: none;
  }
</style>
