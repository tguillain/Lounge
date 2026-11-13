<script lang="ts">
  import { imageUrl } from "$lib/api";
  import { downloadLabel } from "$lib/downloads";
  import { lastPlayedLabel, placeholderArt, playTimeValue, sizeLabel, storeLabel } from "$lib/format";
  import { creditsLine, genresLine, libraryFolder, picturesOf, type DetailAction, type DetailSpot } from "$lib/game-detail";
  import type { AchievementsResult, DetailsResult, Friend, Game } from "$lib/types";

  type Props = {
    game: Game;
    details: DetailsResult;
    achievements: AchievementsResult | null;
    friends: Friend[];
    running: boolean;
    now: Date;
    error: string | null;
    actions: DetailAction[];
    spot: DetailSpot | null;
    isBroken: (url: string) => boolean;
    onBroken: (url: string) => void;
    onFocus: (spot: DetailSpot) => void;
    onAction: (action: DetailAction) => void;
    onOpenPicture: (index: number) => void;
  };

  let { game, details, achievements, friends, running, now, error, actions, spot, isBroken, onBroken, onFocus, onAction, onOpenPicture }: Props =
    $props();

  const MAX_AVATARS = 3;

  const logo = $derived(imageUrl(game.images.logo));
  const logoUrl = $derived(logo && !isBroken(logo) ? logo : null);
  const ready = $derived(details.status === "ready" ? details : null);
  const store = $derived(ready?.store ?? null);
  const genres = $derived(genresLine(store));
  const credits = $derived(creditsLine(store));
  const gallery = $derived(picturesOf(ready));
  const lastPlayed = $derived(lastPlayedLabel(game.lastPlayed, now.getTime() / 1000));
  const playTime = $derived(playTimeValue(game.playTime));
  const size = $derived(sizeLabel(game.sizeOnDisk));
  const folder = $derived(libraryFolder(ready?.installDir ?? null));
  const trophies = $derived(achievements?.status === "ready" ? achievements : null);

  const pictureButtons: HTMLButtonElement[] = [];

  $effect(() => {
    if (spot?.row === "pictures") {
      pictureButtons[spot.index]?.scrollIntoView({ block: "nearest", inline: "nearest", behavior: "smooth" });
    }
  });

  const isFocused = (row: DetailSpot["row"], index: number) => spot?.row === row && spot.index === index;

  const actionLabel = (action: DetailAction): string => {
    switch (action) {
      case "play":
        return running ? "Back to game" : "Play";
      case "achievements":
        return trophies ? `${trophies.unlocked} of ${trophies.total}` : "Achievements";
      case "folder":
        return "Game folder";
      case "options":
        return "More";
    }
  };
</script>

<section class="detail" aria-label="{game.name} details">
  <div class="intro">
    <div class="chips">
      <span class="chip">{storeLabel(game.store)}</span>
      {#if running}
        <span class="chip live"><span class="live-dot"></span>Now playing</span>
      {:else if game.download}
        <span class="chip busy">{downloadLabel(game.download)}</span>
      {:else}
        <span class="chip installed">Installed</span>
      {/if}
      {#if game.hidden}
        <span class="chip">Hidden from Home</span>
      {/if}
    </div>

    <div class="title-slot">
      {#if logoUrl}
        <img class="logo" class:light={game.images.logoIsDark} src={logoUrl} alt={game.name} onerror={() => onBroken(logoUrl)} />
      {:else}
        <h1>{game.name}</h1>
      {/if}
    </div>

    {#if genres}
      <p class="genres">{genres}</p>
    {/if}
    {#if store?.description}
      <p class="description">{store.description}</p>
    {:else if details.status === "loading" && game.store === "steam"}
      <p class="description muted">Loading the store page…</p>
    {/if}
    {#if credits}
      <p class="credits">{credits}</p>
    {/if}

    <div class="actions">
      {#each actions as action, index (action)}
        <button
          type="button"
          class="action"
          class:primary={action === "play"}
          class:focused={isFocused("actions", index)}
          class:complete={action === "achievements" && trophies !== null && trophies.unlocked === trophies.total}
          onpointerenter={() => onFocus({ row: "actions", index })}
          onclick={() => onAction(action)}
        >
          {#if action === "play"}
            {#if running}
              <svg aria-hidden="true" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round"><path d="M5 12h14M13 6l6 6-6 6" /></svg>
            {:else}
              <svg aria-hidden="true" width="18" height="18" viewBox="0 0 24 24"><path d="M6 3.8v16.4a1 1 0 0 0 1.5.9l13.2-8.2a1 1 0 0 0 0-1.8L7.5 2.9A1 1 0 0 0 6 3.8z" fill="currentColor" /></svg>
            {/if}
          {:else if action === "achievements"}
            <svg aria-hidden="true" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M8 21h8M12 17v4M7 4h10v5a5 5 0 0 1-10 0z" /><path d="M17 5h3v2a3 3 0 0 1-3 3M7 5H4v2a3 3 0 0 0 3 3" /></svg>
          {:else if action === "folder"}
            <svg aria-hidden="true" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linejoin="round"><path d="M3 7a2 2 0 0 1 2-2h4l2 2.5h8a2 2 0 0 1 2 2V17a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z" /></svg>
          {:else}
            <svg aria-hidden="true" width="18" height="18" viewBox="0 0 24 24" fill="currentColor"><circle cx="5" cy="12" r="1.8" /><circle cx="12" cy="12" r="1.8" /><circle cx="19" cy="12" r="1.8" /></svg>
          {/if}
          {actionLabel(action)}
        </button>
      {/each}
    </div>
    {#if error}
      <p class="error" role="alert">{error}</p>
    {/if}
  </div>

  <div class="stats" aria-label="About this game">
    <div class="stat">
      <span class="stat-label">Play time</span>
      <span class="stat-value">{playTime ?? (game.playTime === 0 ? "Not played yet" : "—")}</span>
    </div>
    <div class="stat">
      <span class="stat-label">Last played</span>
      <span class="stat-value">{running ? "Right now" : (lastPlayed ?? "Never")}</span>
    </div>
    <div class="stat">
      <span class="stat-label">Friends playing</span>
      <span class="stat-row">
        <span class="stat-value">{friends.length > 0 ? friends.length : "None"}</span>
        {#if friends.length > 0}
          <span class="avatars" aria-label={friends.map((friend) => friend.name).join(", ")}>
            {#each friends.slice(0, MAX_AVATARS) as friend (friend.steamId)}
              <span class="avatar" style:background={placeholderArt(friend.name)}>
                {#if friend.avatar}
                  <img src={friend.avatar} alt="" />
                {:else}
                  {friend.name.slice(0, 1).toUpperCase()}
                {/if}
              </span>
            {/each}
          </span>
        {/if}
      </span>
    </div>
    <div class="stat">
      <span class="stat-label">Storage</span>
      <span class="stat-value">{size ?? "—"}</span>
      {#if folder}
        <span class="stat-note" title={ready?.installDir ?? undefined}>{folder}</span>
      {/if}
    </div>
  </div>

  {#if gallery}
    <section class="pictures" aria-label={gallery.title}>
      <h2>{gallery.title}</h2>
      <div class="strip">
        {#each gallery.pictures as picture, index (picture.full)}
          {@const thumb = imageUrl(picture.thumbnail)}
          <button
            type="button"
            class="picture"
            class:focused={isFocused("pictures", index)}
            aria-label="Open screenshot {index + 1} of {gallery.pictures.length}"
            bind:this={pictureButtons[index]}
            onpointerenter={() => onFocus({ row: "pictures", index })}
            onclick={() => onOpenPicture(index)}
          >
            {#if thumb && !isBroken(thumb)}
              <img src={thumb} alt="" loading="lazy" decoding="async" onerror={() => onBroken(thumb)} />
            {/if}
          </button>
        {/each}
      </div>
    </section>
  {/if}
</section>

<style>
  .detail {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    gap: 22px;
    padding-bottom: 8px;
    animation: rise 0.3s var(--ease);
  }

  .intro {
    flex: 1;
    min-height: 0;
    max-width: 680px;
    display: flex;
    flex-direction: column;
    justify-content: flex-end;
    gap: 12px;
  }

  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .chip {
    padding: 5px 12px;
    border-radius: 999px;
    background: var(--glass);
    border: 1px solid rgba(255, 255, 255, 0.12);
    font-size: 13px;
    font-weight: 600;
    display: flex;
    align-items: center;
    gap: 7px;
  }

  .chip.installed {
    background: rgba(48, 209, 88, 0.16);
    border-color: rgba(48, 209, 88, 0.35);
    color: #7ee2a0;
  }

  .chip.live {
    color: var(--live-text);
  }

  .chip.busy {
    color: #64d2ff;
  }

  .live-dot {
    width: 7px;
    height: 7px;
    border-radius: 999px;
    background: var(--live);
  }

  .title-slot {
    flex: 0 1 110px;
    min-height: 60px;
    display: flex;
    align-items: flex-end;
  }

  .logo {
    max-width: 520px;
    max-height: 100%;
    object-fit: contain;
    object-position: left bottom;
    filter: drop-shadow(0 6px 30px rgba(0, 0, 0, 0.45));
  }

  .logo.light {
    filter: brightness(0) invert(1) drop-shadow(0 6px 30px rgba(0, 0, 0, 0.45));
  }

  h1 {
    margin: 0;
    font-size: 60px;
    line-height: 1.02;
    font-weight: 800;
    letter-spacing: -0.03em;
    text-shadow: 0 6px 40px rgba(0, 0, 0, 0.35);
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .genres {
    margin: 0;
    font-size: 17px;
    font-weight: 600;
    color: #e5e5ea;
  }

  .description {
    margin: 0;
    max-width: 600px;
    font-size: 16px;
    line-height: 1.5;
    color: #d1d1d6;
    display: -webkit-box;
    -webkit-line-clamp: 3;
    line-clamp: 3;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .description.muted,
  .credits {
    color: var(--text-tertiary);
  }

  .credits {
    margin: 0;
    font-size: 14px;
  }

  .actions {
    margin-top: 8px;
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
  }

  .action {
    height: 52px;
    padding: 0 22px;
    border-radius: 999px;
    border: 1px solid rgba(255, 255, 255, 0.16);
    background: rgba(255, 255, 255, 0.12);
    color: var(--text);
    font-size: 15px;
    font-weight: 600;
    display: flex;
    align-items: center;
    gap: 10px;
    cursor: pointer;
    transition:
      background 0.15s ease,
      color 0.15s ease,
      box-shadow 0.15s ease,
      transform 0.15s ease;
  }

  .action.primary {
    height: 56px;
    padding: 0 34px;
    border: 0;
    background: rgba(245, 245, 247, 0.92);
    color: var(--bg);
    font-size: 17px;
  }

  .action.complete svg {
    color: #ffd60a;
  }

  .action.focused {
    transform: scale(1.04);
    box-shadow:
      0 0 0 3px var(--bg),
      0 0 0 5.5px var(--text);
  }

  .action.primary.focused {
    background: #fff;
  }

  .action:focus-visible {
    outline: none;
  }

  .error {
    margin: 0;
    font-size: 15px;
    color: #ff8a80;
  }

  .stats {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 16px;
  }

  .stat {
    min-width: 0;
    padding: 16px 20px;
    border-radius: 18px;
    background: rgba(255, 255, 255, 0.07);
    border: 1px solid rgba(255, 255, 255, 0.1);
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .stat-label {
    font-size: 13px;
    font-weight: 500;
    color: var(--text-tertiary);
  }

  .stat-value {
    font-size: 26px;
    font-weight: 700;
    letter-spacing: -0.02em;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .stat-row {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .stat-note {
    font-size: 13px;
    color: var(--text-tertiary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .avatars {
    display: flex;
  }

  .avatar {
    width: 28px;
    height: 28px;
    border-radius: 999px;
    border: 2px solid #1f1f23;
    overflow: hidden;
    display: grid;
    place-items: center;
    font-size: 12px;
    font-weight: 700;
  }

  .avatar + .avatar {
    margin-left: -8px;
  }

  .avatar img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .pictures {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  h2 {
    margin: 0;
    font-size: 17px;
    font-weight: 600;
  }

  .strip {
    display: flex;
    gap: 16px;
    overflow-x: auto;
    margin: -8px calc(var(--page-gutter, 64px) * -1);
    padding: 8px var(--page-gutter, 64px);
    scroll-padding: 0 var(--page-gutter, 64px);
    scrollbar-width: none;
  }

  .strip::-webkit-scrollbar {
    display: none;
  }

  .picture {
    flex: 0 0 calc((100% - 80px) / 5.4);
    aspect-ratio: 16 / 9;
    padding: 0;
    border: 0;
    border-radius: 14px;
    overflow: hidden;
    background: rgba(255, 255, 255, 0.08);
    cursor: pointer;
    transition:
      transform 0.15s ease,
      box-shadow 0.15s ease;
  }

  .picture img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .picture.focused {
    transform: scale(1.04);
    box-shadow:
      0 0 0 3px var(--bg),
      0 0 0 5.5px var(--text);
  }

  .picture:focus-visible {
    outline: none;
  }

  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(24px);
    }
  }
</style>
