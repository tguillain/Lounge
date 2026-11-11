<script lang="ts">
  import { onMount } from "svelte";
  import { watchControllerBattery } from "$lib/api";
  import { batteryDescription, batteryTone } from "$lib/battery";
  import type { Battery } from "$lib/types";

  let battery = $state<Battery | null>(null);

  onMount(() => {
    const stop = watchControllerBattery((next) => (battery = next));
    return () => {
      stop.then((unwatch) => unwatch());
    };
  });
</script>

{#if battery}
  {@const tone = batteryTone(battery)}
  <span class="battery {tone}" role="img" aria-label={batteryDescription(battery)} title={batteryDescription(battery)}>
    <svg class="controller" aria-hidden="true" viewBox="0 0 24 24" fill="currentColor">
      <path d="M7.5 6h9a5 5 0 0 1 4.9 4l1 5.2a2.8 2.8 0 0 1-5 2.1L15.6 15H8.4l-1.8 2.3a2.8 2.8 0 0 1-5-2.1l1-5.2a5 5 0 0 1 4.9-4zm-.2 3a.7.7 0 0 0-.7.7v1h-1a.7.7 0 0 0 0 1.4h1v1a.7.7 0 0 0 1.4 0v-1h1a.7.7 0 0 0 0-1.4H8v-1a.7.7 0 0 0-.7-.7zm9.2.2a.9.9 0 1 0 0 1.8.9.9 0 0 0 0-1.8zm-2 2a.9.9 0 1 0 0 1.8.9.9 0 0 0 0-1.8z" />
    </svg>
    <span class="cell" aria-hidden="true">
      <span class="fill" style:width="{battery.percent}%"></span>
      {#if battery.charging}
        <svg class="bolt" viewBox="0 0 10 16" fill="currentColor"><path d="M6 0L0 9h3.5L3 16l7-10H6.2L6 0z" /></svg>
      {/if}
    </span>
    <span class="percent">{battery.percent}%</span>
  </span>
{/if}

<style>
  .battery {
    --cell-width: 25px;
    --cell-height: 12px;
    display: inline-flex;
    align-items: center;
    gap: 7px;
    color: var(--text-secondary);
    font-size: 15px;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }

  .controller {
    width: 20px;
    height: 20px;
  }

  .cell {
    position: relative;
    width: var(--cell-width);
    height: var(--cell-height);
    padding: 2px;
    border: 1.5px solid currentColor;
    border-radius: 4px;
    opacity: 0.95;
  }

  .cell::after {
    content: "";
    position: absolute;
    top: 50%;
    right: -4px;
    width: 2px;
    height: 40%;
    border-radius: 0 2px 2px 0;
    background: currentColor;
    transform: translateY(-50%);
  }

  .fill {
    display: block;
    height: 100%;
    min-width: 2px;
    border-radius: 1.5px;
    background: var(--text);
  }

  .charging .fill {
    background: var(--live);
  }

  .low {
    color: #ff6961;
  }

  .low .fill {
    background: #ff453a;
  }

  .bolt {
    position: absolute;
    top: 50%;
    left: 50%;
    width: auto;
    height: calc(var(--cell-height) + 2px);
    color: var(--text);
    filter: drop-shadow(0 0 1.5px rgba(0, 0, 0, 0.8));
    transform: translate(-50%, -50%);
  }

  .percent {
    color: var(--text);
  }

  .low .percent {
    color: #ff6961;
  }
</style>
