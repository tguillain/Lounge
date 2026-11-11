<script lang="ts">
  import { onMount } from "svelte";
  import { SvelteSet } from "svelte/reactivity";
  import FriendsPanel from "$lib/components/FriendsPanel.svelte";
  import GameMenu, { type MenuItem } from "$lib/components/GameMenu.svelte";
  import Hints from "$lib/components/Hints.svelte";
  import LaunchOverlay from "$lib/components/LaunchOverlay.svelte";
  import Shelf from "$lib/components/Shelf.svelte";
  import SteamKeyDialog from "$lib/components/SteamKeyDialog.svelte";
  import {
    backToGame,
    forgetSteamApiKey,
    getSession,
    imageUrl,
    joinFriend,
    launchGame,
    loadFriends,
    loadLibrary,
    onGameSession,
    openSteamApiKeyPage,
    saveSteamApiKey,
    setGameHidden,
    toggleFullscreen,
  } from "$lib/api";
  import { clockLabel, greeting, lastPlayedLabel, placeholderArt, sizeLabel, storeLabel } from "$lib/format";
  import { pulse, watchGamepads } from "$lib/input/gamepad";
  import type { InputMode, PadAction } from "$lib/input/gamepad-mapping";
  import type { Friend, FriendsResult, Game, LibraryState } from "$lib/types";

  const KEY_ACTIONS: Record<string, PadAction> = {
    ArrowLeft: "left",
    ArrowRight: "right",
    ArrowUp: "up",
    ArrowDown: "down",
    Enter: "confirm",
    Escape: "back",
    ContextMenu: "options",
  };

  const LAUNCH_OVERLAY_TIMEOUT_MS = 20_000;
  const CLOCK_REFRESH_MS = 15_000;
  const FRIENDS_REFRESH_MS = 60_000;
  const VISIBLE_FRIENDS = 5;

  let library = $state<LibraryState>({ status: "loading" });
  let selectedIndex = $state(0);
  let now = $state(new Date());
  let launching = $state<Game | null>(null);
  let launchError = $state<string | null>(null);
  let inputMode = $state<InputMode>("keyboard");
  let showHidden = $state(false);
  let menuOpen = $state(false);
  let menuIndex = $state(0);
  let runningGameId = $state<string | null>(null);
  let friendsResult = $state<FriendsResult>({ status: "loading" });
  let focusZone = $state<"shelf" | "friends">("shelf");
  let friendIndex = $state(0);
  let keyDialogOpen = $state(false);
  const brokenImages = new SvelteSet<string>();
  let launchTimer: ReturnType<typeof setTimeout> | undefined;

  const allGames = $derived(library.status === "ready" ? library.games : []);
  const hiddenCount = $derived(allGames.filter((game) => game.hidden).length);
  const games = $derived(showHidden ? allGames : allGames.filter((game) => !game.hidden));
  const selected = $derived<Game | null>(games[selectedIndex] ?? null);
  const selectedIsRunning = $derived(selected !== null && selected.id === runningGameId);
  const visibleFriends = $derived(friendsResult.status === "ready" ? friendsResult.friends.slice(0, VISIBLE_FRIENDS) : []);
  const friendsNeedSetup = $derived(friendsResult.status === "not-configured" || friendsResult.status === "failed");
  const focusedFriend = $derived<Friend | null>(focusZone === "friends" ? (visibleFriends[friendIndex] ?? null) : null);

  const refreshFriends = async () => {
    try {
      friendsResult = await loadFriends();
    } catch (error) {
      friendsResult = { status: "failed", message: String(error) };
    }
    friendIndex = Math.min(friendIndex, Math.max(0, visibleFriends.length - 1));
  };

  const saveKey = async (key: string) => {
    await saveSteamApiKey(key);
    keyDialogOpen = false;
    friendsResult = { status: "loading" };
    await refreshFriends();
  };

  const forgetKey = async () => {
    await forgetSteamApiKey();
    keyDialogOpen = false;
    friendsResult = { status: "not-configured" };
  };

  const join = async (friend: Friend) => {
    launchError = null;
    try {
      await joinFriend(friend);
    } catch (error) {
      launchError = `Couldn't join ${friend.name}: ${error}`;
    }
  };

  const canFocusFriends = () => visibleFriends.length > 0 || friendsNeedSetup;

  const runFriendsAction = (action: PadAction) => {
    if (action === "up") {
      friendIndex = Math.max(0, friendIndex - 1);
    } else if (action === "down") {
      if (friendIndex >= visibleFriends.length - 1) {
        focusZone = "shelf";
      } else {
        friendIndex += 1;
      }
    } else if (action === "left" || action === "back") {
      focusZone = "shelf";
    } else if (action === "confirm") {
      if (friendsNeedSetup) {
        keyDialogOpen = true;
      } else if (focusedFriend?.joinUri) {
        join(focusedFriend);
      }
    }
  };

  const toggleHidden = async (game: Game) => {
    const hidden = !game.hidden;
    if (library.status === "ready") {
      library = { status: "ready", games: library.games.map((g) => (g.id === game.id ? { ...g, hidden } : g)) };
    }
    selectedIndex = Math.max(0, Math.min(selectedIndex, games.length - 1));
    try {
      await setGameHidden(game, hidden);
    } catch (error) {
      launchError = `Couldn't save your choice for ${game.name}: ${error}`;
    }
  };

  const toggleShowHidden = () => {
    const keepId = selected?.id;
    showHidden = !showHidden;
    selectedIndex = Math.max(0, games.findIndex((game) => game.id === keepId));
  };

  const closeMenu = () => {
    menuOpen = false;
  };

  const menuItems = $derived.by<MenuItem[]>(() => {
    if (!selected) {
      return [];
    }
    const game = selected;
    const items: MenuItem[] = [
      {
        id: "hide",
        label: game.hidden ? "Show on Home" : "Hide from Home",
        run: () => {
          closeMenu();
          toggleHidden(game);
        },
      },
    ];
    if (hiddenCount > 0 || showHidden) {
      items.push({
        id: "show-hidden",
        label: showHidden ? "Stop showing hidden games" : `Show hidden games (${hiddenCount})`,
        run: () => {
          closeMenu();
          toggleShowHidden();
        },
      });
    }
    items.push({ id: "close", label: "Close", run: closeMenu });
    return items;
  });

  const openMenu = () => {
    if (!selected) {
      return;
    }
    menuIndex = 0;
    menuOpen = true;
  };

  const usableImage = (source: string | null): string | null => {
    const url = imageUrl(source);
    return url && !brokenImages.has(url) ? url : null;
  };

  const heroUrl = $derived(selected ? usableImage(selected.images.hero) : null);
  const logoUrl = $derived(selected ? usableImage(selected.images.logo) : null);
  const lastPlayed = $derived(selected ? lastPlayedLabel(selected.lastPlayed, now.getTime() / 1000) : null);
  const size = $derived(selected ? sizeLabel(selected.sizeOnDisk) : null);

  const refreshLibrary = async () => {
    try {
      const loaded = await loadLibrary();
      library = { status: "ready", games: loaded };
      selectedIndex = 0;
    } catch (error) {
      library = { status: "error", message: String(error) };
    }
  };

  const closeLaunchOverlay = () => {
    clearTimeout(launchTimer);
    launching = null;
  };

  const play = async (game: Game) => {
    launchError = null;
    launching = game;
    clearTimeout(launchTimer);
    launchTimer = setTimeout(closeLaunchOverlay, LAUNCH_OVERLAY_TIMEOUT_MS);
    try {
      await launchGame(game);
    } catch (error) {
      closeLaunchOverlay();
      launchError = `Couldn't launch ${game.name}: ${error}`;
    }
  };

  const returnToGame = async () => {
    launchError = null;
    try {
      await backToGame();
    } catch (error) {
      launchError = `Couldn't switch back to the game: ${error}`;
    }
  };

  const playOrReturn = (game: Game) => (game.id === runningGameId ? returnToGame() : play(game));

  const moveSelection = (step: number): boolean => {
    const next = Math.min(games.length - 1, Math.max(0, selectedIndex + step));
    if (games.length === 0 || next === selectedIndex) {
      return false;
    }
    selectedIndex = next;
    return true;
  };

  const runMenuAction = (action: PadAction) => {
    if (action === "up" || action === "down") {
      const step = action === "down" ? 1 : -1;
      menuIndex = (menuIndex + step + menuItems.length) % menuItems.length;
      if (inputMode !== "keyboard") {
        pulse();
      }
    } else if (action === "confirm") {
      menuItems[menuIndex]?.run();
    } else if (action === "back" || action === "options") {
      closeMenu();
    }
  };

  const runAction = (action: PadAction) => {
    if (keyDialogOpen) {
      if (action === "back") {
        keyDialogOpen = false;
      }
      return;
    }
    if (menuOpen) {
      runMenuAction(action);
      return;
    }
    if (launching) {
      if (action === "back") {
        closeLaunchOverlay();
      }
      return;
    }
    if (focusZone === "friends") {
      runFriendsAction(action);
      return;
    }
    if (action === "up" && canFocusFriends()) {
      focusZone = "friends";
      friendIndex = 0;
      return;
    }
    if (action === "left" || action === "right") {
      if (moveSelection(action === "right" ? 1 : -1) && inputMode !== "keyboard") {
        pulse();
      }
    } else if (action === "confirm" && selected) {
      playOrReturn(selected);
    } else if (action === "options") {
      openMenu();
    }
  };

  const handleKeydown = (event: KeyboardEvent) => {
    inputMode = "keyboard";
    if (keyDialogOpen) {
      return;
    }
    if (event.key === "F11") {
      event.preventDefault();
      toggleFullscreen();
      return;
    }
    const action = KEY_ACTIONS[event.key];
    if (action) {
      event.preventDefault();
      runAction(action);
    }
  };

  onMount(() => {
    refreshLibrary();

    const stopGamepads = watchGamepads((action, family) => {
      inputMode = family;
      runAction(action);
    });

    const clock = setInterval(() => {
      if (document.visibilityState === "visible") {
        now = new Date();
      }
    }, CLOCK_REFRESH_MS);

    refreshFriends();
    const friendsTimer = setInterval(() => {
      if (document.visibilityState === "visible" && friendsResult.status === "ready") {
        refreshFriends();
      }
    }, FRIENDS_REFRESH_MS);

    getSession()
      .then((session) => (runningGameId = session ? `steam:${session.appId}` : null))
      .catch(() => (runningGameId = null));

    const stopListening = onGameSession(
      (gameId) => {
        runningGameId = gameId;
        closeLaunchOverlay();
      },
      () => {
        runningGameId = null;
        refreshLibrary();
      },
    );

    return () => {
      clearInterval(clock);
      clearInterval(friendsTimer);
      clearTimeout(launchTimer);
      stopGamepads();
      stopListening.then((stop) => stop());
    };
  });
</script>

<svelte:window
  onkeydown={handleKeydown}
  onpointerdown={() => (inputMode = "keyboard")}
  oncontextmenu={(event) => event.preventDefault()}
/>

<div class="screen">
  {#if selected}
    {#key selected.id}
      <div class="hero-art" style:background={placeholderArt(selected.name)}>
        {#if heroUrl}
          <img src={heroUrl} alt="" decoding="async" onerror={() => brokenImages.add(heroUrl)} />
        {/if}
      </div>
    {/key}
  {/if}
  <div class="scrim"></div>

  <div class="layout">
    <header class="top-bar">
      <span class="greeting">{greeting(now)}</span>
      <nav aria-label="Main navigation">
        <span class="tab active" aria-current="page">Home</span>
        <span class="tab" aria-disabled="true" title="Coming soon">Library</span>
      </nav>
      <span class="clock">{clockLabel(now)}</span>
    </header>

    {#if library.status === "loading"}
      <section class="message" aria-live="polite">
        <p class="message-title">Loading your library…</p>
      </section>
    {:else if library.status === "error"}
      <section class="message" role="alert">
        <p class="message-title">Couldn't read your library</p>
        <p class="message-body">{library.message}</p>
        <button type="button" class="secondary" onclick={refreshLibrary}>Try again</button>
      </section>
    {:else if !selected && allGames.length > 0}
      <section class="message">
        <p class="message-title">All your games are hidden</p>
        <p class="message-body">Show them to bring some back to Home.</p>
        <button type="button" class="secondary" onclick={toggleShowHidden}>Show hidden games</button>
      </section>
    {:else if !selected}
      <section class="message">
        <p class="message-title">No installed games found</p>
        <p class="message-body">Install a game on Steam or Epic Games, then refresh.</p>
        <button type="button" class="secondary" onclick={refreshLibrary}>Refresh</button>
      </section>
    {:else}
      <div class="main-row">
      <section class="hero" aria-label="Selected game">
        <div class="title-slot">
          {#key selected.id}
            {#if logoUrl}
              <img class="logo" class:light={selected.images.logoIsDark} src={logoUrl} alt={selected.name} onerror={() => brokenImages.add(logoUrl)} />
            {:else}
              <h1>{selected.name}</h1>
            {/if}
          {/key}
        </div>
        <div class="meta">
          <span class="pill">{storeLabel(selected.store)}</span>
          {#if selectedIsRunning}
            <span class="live"><span class="live-dot"></span>Now playing</span>
          {:else if lastPlayed}
            <span>Last played {lastPlayed}</span>
          {:else}
            <span>Never played</span>
          {/if}
          {#if size}
            <span class="dot" aria-hidden="true">·</span>
            <span>{size}</span>
          {/if}
        </div>
        <div class="actions">
          <button type="button" class="primary" onclick={() => selected && playOrReturn(selected)}>
            {#if selectedIsRunning}
              <svg aria-hidden="true" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round"><path d="M5 12h14M13 6l6 6-6 6" /></svg>
              Back to game
            {:else}
              <svg aria-hidden="true" width="18" height="18" viewBox="0 0 24 24"><path d="M6 3.8v16.4a1 1 0 0 0 1.5.9l13.2-8.2a1 1 0 0 0 0-1.8L7.5 2.9A1 1 0 0 0 6 3.8z" fill="currentColor" /></svg>
              Play
            {/if}
          </button>
        </div>
        {#if launchError}
          <p class="error" role="alert">{launchError}</p>
        {/if}
      </section>

      <FriendsPanel
        result={friendsResult}
        focusedIndex={focusZone === "friends" ? friendIndex : null}
        panelFocused={focusZone === "friends" && friendsNeedSetup}
        onConnect={() => (keyDialogOpen = true)}
        onRetry={refreshFriends}
        onJoin={join}
      />
      </div>

      <section class="shelf" aria-label="Recently played">
        <h2>Recently played</h2>
        <Shelf
          {games}
          {selectedIndex}
          isBroken={(url) => brokenImages.has(url)}
          onBroken={(url) => brokenImages.add(url)}
          {runningGameId}
          onSelect={(index) => (selectedIndex = index)}
          onLaunch={playOrReturn}
          onOptions={(index) => {
            selectedIndex = index;
            openMenu();
          }}
        />
      </section>
    {/if}

    <Hints
      mode={inputMode}
      hints={focusZone === "friends"
        ? [
            { action: "navigate", label: "Navigate" },
            ...(friendsNeedSetup ? [{ action: "confirm" as const, label: "Connect" }] : []),
            ...(focusedFriend?.joinUri ? [{ action: "confirm" as const, label: "Join" }] : []),
            { action: "back", label: "Back to games" },
          ]
        : [
            { action: "navigate", label: "Navigate" },
            { action: "confirm", label: selectedIsRunning ? "Back to game" : "Play" },
            { action: "options", label: "Options" },
            { action: "fullscreen", label: "Full screen" },
          ]}
    />
  </div>

  {#if keyDialogOpen}
    <SteamKeyDialog
      connected={friendsResult.status !== "not-configured"}
      onSave={saveKey}
      onForget={forgetKey}
      onOpenKeyPage={openSteamApiKeyPage}
      onClose={() => (keyDialogOpen = false)}
    />
  {/if}

  {#if menuOpen && selected}
    <GameMenu
      title={selected.name}
      items={menuItems}
      focusedIndex={menuIndex}
      mode={inputMode}
      onFocus={(index) => (menuIndex = index)}
      onClose={closeMenu}
    />
  {/if}

  {#if launching}
    <LaunchOverlay
      game={launching}
      heroUrl={usableImage(launching.images.hero)}
      logoUrl={usableImage(launching.images.logo)}
      background={placeholderArt(launching.name)}
      mode={inputMode}
    />
  {/if}
</div>

<style>
  .screen {
    position: relative;
    height: 100vh;
    overflow: hidden;
  }

  .hero-art {
    position: absolute;
    inset: 0 0 auto 0;
    height: 78vh;
    animation: fade-in 0.5s var(--ease);
  }

  .hero-art img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    object-position: center top;
  }

  .scrim {
    position: absolute;
    inset: 0;
    background:
      linear-gradient(90deg, rgba(10, 10, 12, 0.92) 0%, rgba(10, 10, 12, 0.6) 38%, rgba(10, 10, 12, 0) 70%),
      linear-gradient(0deg, var(--bg) 0%, var(--bg) 22%, rgba(10, 10, 12, 0.7) 40%, rgba(10, 10, 12, 0) 65%);
  }

  .layout {
    position: relative;
    height: 100%;
    padding: 36px 64px 32px;
    display: flex;
    flex-direction: column;
  }

  .top-bar {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto minmax(0, 1fr);
    align-items: center;
  }

  .greeting,
  .clock {
    font-size: 17px;
    font-weight: 600;
  }

  .clock {
    justify-self: end;
    font-variant-numeric: tabular-nums;
  }

  nav {
    display: flex;
    gap: 4px;
    padding: 5px;
    border-radius: 999px;
    background: rgba(28, 28, 32, 0.7);
    border: 1px solid rgba(255, 255, 255, 0.08);
  }

  .tab {
    padding: 10px 22px;
    border-radius: 999px;
    font-size: 15px;
    font-weight: 500;
  }

  .tab.active {
    background: var(--text);
    color: var(--bg);
    font-weight: 600;
  }

  .tab[aria-disabled="true"] {
    color: var(--text-tertiary);
  }

  .main-row {
    flex: 1;
    min-height: 0;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 48px;
  }

  .hero {
    flex: 0 1 640px;
    min-width: 0;
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 20px;
  }

  .title-slot {
    height: 180px;
    display: flex;
    align-items: flex-end;
  }

  .logo {
    max-width: 520px;
    max-height: 100%;
    object-fit: contain;
    object-position: left bottom;
    filter: drop-shadow(0 6px 30px rgba(0, 0, 0, 0.45));
    animation: fade-in 0.4s var(--ease);
  }

  .logo.light {
    filter: brightness(0) invert(1) drop-shadow(0 6px 30px rgba(0, 0, 0, 0.45));
  }

  h1 {
    margin: 0;
    font-size: 64px;
    line-height: 1.05;
    font-weight: 700;
    letter-spacing: -0.035em;
    display: -webkit-box;
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    overflow: hidden;
    animation: fade-in 0.4s var(--ease);
  }

  .meta {
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: 16px;
    color: #d1d1d6;
  }

  .pill {
    padding: 5px 12px;
    border-radius: 999px;
    background: var(--glass);
    border: 1px solid var(--glass-border);
    font-size: 13px;
    font-weight: 600;
    color: var(--text);
  }

  .dot {
    color: var(--line);
  }

  .live {
    display: flex;
    align-items: center;
    gap: 8px;
    font-weight: 600;
    color: var(--live-text);
  }

  .live-dot {
    width: 8px;
    height: 8px;
    border-radius: 999px;
    background: var(--live);
    box-shadow: 0 0 0 4px rgba(48, 209, 88, 0.25);
  }

  .actions {
    display: flex;
    gap: 14px;
    margin-top: 12px;
  }

  .primary,
  .secondary {
    height: 58px;
    padding: 0 34px;
    border-radius: 999px;
    font-size: 18px;
    font-weight: 600;
    display: flex;
    align-items: center;
    gap: 10px;
    cursor: pointer;
  }

  .primary {
    border: 0;
    background: var(--text);
    color: var(--bg);
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.35);
  }

  .secondary {
    background: var(--glass);
    border: 1px solid var(--glass-border);
  }

  .error {
    margin: 0;
    font-size: 15px;
    color: #ff8a80;
  }

  .message {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    text-align: center;
  }

  .message-title {
    margin: 0;
    font-size: 24px;
    font-weight: 600;
  }

  .message-body {
    margin: 0 0 12px;
    max-width: 480px;
    font-size: 16px;
    color: var(--text-tertiary);
  }

  .shelf {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  h2 {
    margin: 0;
    font-size: 20px;
    font-weight: 600;
  }

  @keyframes fade-in {
    from {
      opacity: 0;
    }
  }
</style>
