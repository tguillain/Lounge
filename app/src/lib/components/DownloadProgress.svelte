<script lang="ts">
  import { downloadLabel } from "$lib/downloads";
  import type { Download } from "$lib/types";

  type Props = { download: Download };

  let { download }: Props = $props();
</script>

<span class="badge" class:paused={download.state !== "downloading"}>
  <svg aria-hidden="true" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.6" stroke-linecap="round" stroke-linejoin="round">
    <path d="M12 4v11M7 10l5 5 5-5M5 20h14" />
  </svg>
  {downloadLabel(download)}
</span>
{#if download.percent !== null}
  <span class="track" aria-hidden="true">
    <span class="bar" class:paused={download.state !== "downloading"} style:width="{download.percent}%"></span>
  </span>
{/if}

<style>
  .badge {
    position: absolute;
    top: 10px;
    left: 10px;
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 10px;
    border-radius: 999px;
    background: rgba(10, 10, 12, 0.85);
    font-size: 12px;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
    color: var(--text);
  }

  .badge.paused {
    color: var(--text-tertiary);
  }

  .track {
    position: absolute;
    left: 0;
    right: 0;
    bottom: 0;
    height: 5px;
    background: rgba(10, 10, 12, 0.7);
  }

  .bar {
    display: block;
    height: 100%;
    background: var(--text);
    transition: width 0.6s var(--ease);
  }

  .bar.paused {
    background: var(--text-tertiary);
  }
</style>
