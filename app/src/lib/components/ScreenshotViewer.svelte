<script lang="ts">
  import { imageUrl } from "$lib/api";
  import type { Picture } from "$lib/types";

  type Props = {
    title: string;
    pictures: Picture[];
    index: number;
    onStep: (step: 1 | -1) => void;
    onClose: () => void;
  };

  let { title, pictures, index, onStep, onClose }: Props = $props();

  const picture = $derived(pictures[index]);
  const full = $derived(picture ? imageUrl(picture.full) : null);
  const preview = $derived(picture ? imageUrl(picture.thumbnail) : null);
</script>

<div class="viewer" role="dialog" aria-modal="true" aria-label="{title}, {index + 1} of {pictures.length}">
  <button type="button" class="backdrop" aria-label="Close" onclick={onClose}></button>
  {#key index}
    <div class="frame" style:background-image={preview ? `url("${preview}")` : undefined}>
      {#if full}
        <img src={full} alt="Screenshot {index + 1} of {pictures.length}" decoding="async" />
      {/if}
    </div>
  {/key}
  <button type="button" class="arrow left" aria-label="Previous screenshot" disabled={index === 0} onclick={() => onStep(-1)}>
    <svg aria-hidden="true" width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round"><path d="M15 5l-7 7 7 7" /></svg>
  </button>
  <button type="button" class="arrow right" aria-label="Next screenshot" disabled={index === pictures.length - 1} onclick={() => onStep(1)}>
    <svg aria-hidden="true" width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round"><path d="M9 5l7 7-7 7" /></svg>
  </button>
  <p class="counter">{title} · {index + 1} / {pictures.length}</p>
</div>

<style>
  .viewer {
    position: fixed;
    inset: 0;
    z-index: 30;
    display: grid;
    place-items: center;
    animation: appear 0.2s var(--ease);
  }

  .backdrop {
    position: absolute;
    inset: 0;
    border: 0;
    background: rgba(5, 5, 7, 0.94);
    cursor: default;
  }

  .frame {
    position: relative;
    width: min(86vw, calc(74vh * 16 / 9));
    margin-bottom: 24px;
    aspect-ratio: 16 / 9;
    border-radius: 18px;
    overflow: hidden;
    background-size: cover;
    background-position: center;
    box-shadow: 0 30px 80px rgba(0, 0, 0, 0.6);
    animation: appear 0.25s var(--ease);
  }

  .frame img {
    width: 100%;
    height: 100%;
    object-fit: contain;
    background: #000;
  }

  .arrow {
    position: absolute;
    top: 50%;
    width: 52px;
    height: 52px;
    margin-top: -26px;
    border-radius: 999px;
    border: 1px solid rgba(255, 255, 255, 0.16);
    background: rgba(255, 255, 255, 0.12);
    color: var(--text);
    display: grid;
    place-items: center;
    cursor: pointer;
  }

  .arrow:disabled {
    opacity: 0.25;
    cursor: default;
  }

  .arrow.left {
    left: 32px;
  }

  .arrow.right {
    right: 32px;
  }

  .counter {
    position: absolute;
    left: 0;
    right: 0;
    bottom: 40px;
    margin: 0;
    text-align: center;
    font-size: 15px;
    color: var(--text-tertiary);
    font-variant-numeric: tabular-nums;
  }

  @keyframes appear {
    from {
      opacity: 0;
    }
  }
</style>
