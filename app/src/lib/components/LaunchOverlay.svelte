<script lang="ts">
  import Hints from "./Hints.svelte";
  import { storeLabel } from "$lib/format";
  import type { InputMode } from "$lib/input/gamepad-mapping";
  import type { Game } from "$lib/types";

  type Props = {
    game: Game;
    heroUrl: string | null;
    logoUrl: string | null;
    background: string;
    mode: InputMode;
  };

  let { game, heroUrl, logoUrl, background, mode }: Props = $props();
</script>

<div class="overlay" role="status" aria-live="polite">
  <div class="art" style:background>
    {#if heroUrl}
      <img src={heroUrl} alt="" />
    {/if}
  </div>
  <div class="scrim"></div>

  <div class="content">
    {#if logoUrl}
      <img class="logo" class:light={game.images.logoIsDark} src={logoUrl} alt={game.name} />
    {:else}
      <h1>{game.name}</h1>
    {/if}
    <div class="progress" aria-hidden="true"><span></span></div>
    <p>Lancement via {storeLabel(game.store)}…</p>
  </div>

  <div class="footer"><Hints hints={[{ action: "back", label: "Fermer" }]} {mode} /></div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    z-index: 10;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    background: var(--bg);
    animation: appear 0.3s var(--ease);
  }

  .art,
  .scrim {
    position: absolute;
    inset: 0;
  }

  .art img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    animation: breathe 3s ease-in-out infinite;
  }

  .scrim {
    background: radial-gradient(60% 60% at 50% 50%, rgba(10, 10, 12, 0.5) 0%, rgba(10, 10, 12, 0.92) 100%);
  }

  .content {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 28px;
    text-align: center;
  }

  .logo {
    max-width: 560px;
    max-height: 200px;
    object-fit: contain;
    filter: drop-shadow(0 6px 30px rgba(0, 0, 0, 0.45));
  }

  .logo.light {
    filter: brightness(0) invert(1) drop-shadow(0 6px 30px rgba(0, 0, 0, 0.45));
  }

  h1 {
    margin: 0;
    font-size: 64px;
    font-weight: 700;
    letter-spacing: -0.03em;
  }

  .progress {
    width: 220px;
    height: 3px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.18);
    overflow: hidden;
  }

  .progress span {
    display: block;
    width: 40%;
    height: 100%;
    border-radius: 999px;
    background: var(--text);
    animation: slide 1.4s ease-in-out infinite;
  }

  p {
    margin: 0;
    font-size: 17px;
    font-weight: 500;
  }

  .footer {
    position: absolute;
    right: 64px;
    bottom: 32px;
  }

  @keyframes appear {
    from {
      opacity: 0;
    }
  }

  @keyframes slide {
    from {
      transform: translateX(-100%);
    }
    to {
      transform: translateX(260%);
    }
  }

  @keyframes breathe {
    0%,
    100% {
      opacity: 0.85;
    }
    50% {
      opacity: 1;
    }
  }
</style>
