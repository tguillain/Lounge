<script lang="ts">
  import { onMount } from "svelte";
  import { SvelteSet } from "svelte/reactivity";
  import FriendsPanel from "$lib/components/FriendsPanel.svelte";
  import GameMenu, { type MenuItem } from "$lib/components/GameMenu.svelte";
  import Hints, { type Hint } from "$lib/components/Hints.svelte";
  import HomeHero from "$lib/components/HomeHero.svelte";
  import LaunchOverlay from "$lib/components/LaunchOverlay.svelte";
  import LibraryView from "$lib/components/LibraryView.svelte";
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
  import { clockLabel, greeting, placeholderArt } from "$lib/format";
  import { pulse, watchGamepads } from "$lib/input/gamepad";
  import type { InputMode, PadAction } from "$lib/input/gamepad-mapping";
  import {
    availableFilters,
    DEFAULT_SORT,
    filterGames,
    moveInGrid,
    nextSort,
    sortGames,
    type LibraryFilter,
    type LibrarySort,
  } from "$lib/library";
  import type { Friend, FriendsResult, Game, LibraryState } from "$lib/types";

  type View = "home" | "library";

  const KEY_ACTIONS: Record<string, PadAction> = {
    ArrowLeft: "left",
    ArrowRight: "right",
    ArrowUp: "up",
    ArrowDown: "down",
    Enter: "confirm",
    Escape: "back",
    ContextMenu: "options",
    q: "previousTab",
    Q: "previousTab",
    e: "nextTab",
    E: "nextTab",
  };

  const LAUNCH_OVERLAY_TIMEOUT_MS = 20_000;
  const CLOCK_REFRESH_MS = 15_000;
  const FRIENDS_REFRESH_MS = 60_000;
  const VISIBLE_FRIENDS = 5;

  let library = $state<LibraryState>({ status: "loading" });
  let view = $state<View>("home");
  let now = $state(new Date());
  let inputMode = $state<InputMode>("keyboard");
  let launching = $state<Game | null>(null);
  let launchError = $state<string | null>(null);
  let runningGameId = $state<string | null>(null);
  let launchTimer: ReturnType<typeof setTimeout> | undefined;
  const brokenImages = new SvelteSet<string>();

  let selectedIndex = $state(0);
  let homeZone = $state<"shelf" | "friends">("shelf");
  let friendsResult = $state<FriendsResult>({ status: "loading" });
  let friendIndex = $state(0);
  let keyDialogOpen = $state(false);

  let libraryFilter = $state<LibraryFilter>("all");
  let librarySort = $state<LibrarySort>(DEFAULT_SORT);
  let libraryZone = $state<"grid" | "controls">("grid");
  let libraryIndex = $state(0);
  let controlIndex = $state(0);
  let libraryColumns = $state(6);

  let menuGame = $state<Game | null>(null);
  let menuIndex = $state(0);

  const allGames = $derived(library.status === "ready" ? library.games : []);
  const homeGames = $derived(allGames.filter((game) => !game.hidden));
  const selected = $derived<Game | null>(homeGames[selectedIndex] ?? null);
  const selectedIsRunning = $derived(selected !== null && selected.id === runningGameId);

  const filters = $derived(availableFilters(allGames));
  const libraryGames = $derived(sortGames(filterGames(allGames, libraryFilter), librarySort));
  const focusedLibraryGame = $derived<Game | null>(libraryGames[libraryIndex] ?? null);

  const visibleFriends = $derived(friendsResult.status === "ready" ? friendsResult.friends.slice(0, VISIBLE_FRIENDS) : []);
  const friendsNeedSetup = $derived(friendsResult.status === "not-configured" || friendsResult.status === "failed");
  const focusedFriend = $derived<Friend | null>(homeZone === "friends" ? (visibleFriends[friendIndex] ?? null) : null);

  const isBroken = (url: string) => brokenImages.has(url);
  const markBroken = (url: string) => brokenImages.add(url);

  const usableImage = (source: string | null): string | null => {
    const url = imageUrl(source);
    return url && !brokenImages.has(url) ? url : null;
  };

  const heroUrl = $derived(selected ? usableImage(selected.images.hero) : null);

  const feedback = () => {
    if (inputMode !== "keyboard") {
      pulse();
    }
  };

  const refreshLibrary = async () => {
    try {
      const loaded = await loadLibrary();
      library = { status: "ready", games: loaded };
      selectedIndex = 0;
      libraryIndex = Math.min(libraryIndex, Math.max(0, libraryGames.length - 1));
    } catch (error) {
      library = { status: "error", message: String(error) };
    }
  };

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

  const join = async (friend: Friend) => {
    launchError = null;
    try {
      await joinFriend(friend);
    } catch (error) {
      launchError = `Couldn't join ${friend.name}: ${error}`;
    }
  };

  const toggleHidden = async (game: Game) => {
    const hidden = !game.hidden;
    if (library.status === "ready") {
      library = { status: "ready", games: library.games.map((g) => (g.id === game.id ? { ...g, hidden } : g)) };
    }
    selectedIndex = Math.max(0, Math.min(selectedIndex, homeGames.length - 1));
    libraryIndex = Math.max(0, Math.min(libraryIndex, libraryGames.length - 1));
    if (libraryGames.length === 0 && libraryFilter === "hidden") {
      libraryFilter = "all";
    }
    try {
      await setGameHidden(game, hidden);
    } catch (error) {
      launchError = `Couldn't save your choice for ${game.name}: ${error}`;
    }
  };

  const closeMenu = () => {
    menuGame = null;
  };

  const openMenu = (game: Game | null) => {
    if (game) {
      menuIndex = 0;
      menuGame = game;
    }
  };

  const menuItems = $derived.by<MenuItem[]>(() => {
    const game = menuGame;
    if (!game) {
      return [];
    }
    return [
      {
        id: "hide",
        label: game.hidden ? "Show on Home" : "Hide from Home",
        run: () => {
          closeMenu();
          toggleHidden(game);
        },
      },
      { id: "close", label: "Close", run: closeMenu },
    ];
  });

  const switchView = (target: View) => {
    if (view === target) {
      return;
    }
    view = target;
    homeZone = "shelf";
    libraryZone = "grid";
    feedback();
  };

  const selectFilter = (filter: LibraryFilter) => {
    libraryFilter = filter;
    libraryIndex = 0;
  };

  const cycleSort = () => {
    librarySort = nextSort(librarySort);
    libraryIndex = 0;
  };

  const moveSelection = (step: number) => {
    const next = Math.min(homeGames.length - 1, Math.max(0, selectedIndex + step));
    if (homeGames.length > 0 && next !== selectedIndex) {
      selectedIndex = next;
      feedback();
    }
  };

  const runMenuAction = (action: PadAction) => {
    if (action === "up" || action === "down") {
      const step = action === "down" ? 1 : -1;
      menuIndex = (menuIndex + step + menuItems.length) % menuItems.length;
      feedback();
    } else if (action === "confirm") {
      menuItems[menuIndex]?.run();
    } else if (action === "back" || action === "options") {
      closeMenu();
    }
  };

  const runFriendsAction = (action: PadAction) => {
    if (action === "up") {
      friendIndex = Math.max(0, friendIndex - 1);
    } else if (action === "down") {
      if (friendIndex >= visibleFriends.length - 1) {
        homeZone = "shelf";
      } else {
        friendIndex += 1;
      }
    } else if (action === "left" || action === "back") {
      homeZone = "shelf";
    } else if (action === "confirm") {
      if (friendsNeedSetup) {
        keyDialogOpen = true;
      } else if (focusedFriend?.joinUri) {
        join(focusedFriend);
      }
    }
  };

  const runHomeAction = (action: PadAction) => {
    if (homeZone === "friends") {
      runFriendsAction(action);
    } else if (action === "up" && (visibleFriends.length > 0 || friendsNeedSetup)) {
      homeZone = "friends";
      friendIndex = 0;
    } else if (action === "left" || action === "right") {
      moveSelection(action === "right" ? 1 : -1);
    } else if (action === "confirm" && selected) {
      playOrReturn(selected);
    } else if (action === "options") {
      openMenu(selected);
    }
  };

  const runControlsAction = (action: PadAction) => {
    const lastControl = filters.length;
    if (action === "left" || action === "right") {
      controlIndex = Math.max(0, Math.min(lastControl, controlIndex + (action === "right" ? 1 : -1)));
    } else if (action === "down") {
      libraryZone = "grid";
    } else if (action === "confirm") {
      if (controlIndex === lastControl) {
        cycleSort();
      } else {
        selectFilter(filters[controlIndex].id);
      }
    } else if (action === "back") {
      switchView("home");
    }
  };

  const runLibraryAction = (action: PadAction) => {
    if (libraryZone === "controls") {
      runControlsAction(action);
      return;
    }
    if (action === "left" || action === "right" || action === "up" || action === "down") {
      const move = moveInGrid(libraryIndex, action, libraryColumns, libraryGames.length);
      if (move.leftTop || libraryGames.length === 0) {
        libraryZone = "controls";
        controlIndex = Math.max(0, filters.findIndex((filter) => filter.id === libraryFilter));
      } else if (move.index !== libraryIndex) {
        libraryIndex = move.index;
        feedback();
      }
    } else if (action === "confirm" && focusedLibraryGame) {
      playOrReturn(focusedLibraryGame);
    } else if (action === "options") {
      openMenu(focusedLibraryGame);
    } else if (action === "back") {
      switchView("home");
    }
  };

  const runAction = (action: PadAction) => {
    if (keyDialogOpen) {
      if (action === "back") {
        keyDialogOpen = false;
      }
      return;
    }
    if (menuGame) {
      runMenuAction(action);
      return;
    }
    if (launching) {
      if (action === "back") {
        closeLaunchOverlay();
      }
      return;
    }
    if (action === "previousTab" || action === "nextTab") {
      switchView(action === "nextTab" ? "library" : "home");
    } else if (view === "home") {
      runHomeAction(action);
    } else {
      runLibraryAction(action);
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

  const hints = $derived.by<Hint[]>(() => {
    if (view === "library") {
      return [
        { action: "navigate", label: "Navigate" },
        {
          action: "confirm",
          label: libraryZone === "controls" ? "Select" : focusedLibraryGame?.id === runningGameId ? "Back to game" : "Play",
        },
        { action: "options", label: "Options" },
        { action: "tabs", label: "Home" },
      ];
    }
    if (homeZone === "friends") {
      return [
        { action: "navigate", label: "Navigate" },
        ...(friendsNeedSetup ? [{ action: "confirm" as const, label: "Connect" }] : []),
        ...(focusedFriend?.joinUri ? [{ action: "confirm" as const, label: "Join" }] : []),
        { action: "back", label: "Back to games" },
      ];
    }
    return [
      { action: "navigate", label: "Navigate" },
      { action: "confirm", label: selectedIsRunning ? "Back to game" : "Play" },
      { action: "options", label: "Options" },
      { action: "tabs", label: "Library" },
      { action: "fullscreen", label: "Full screen" },
    ];
  });

  onMount(() => {
    refreshLibrary();
    refreshFriends();

    const stopGamepads = watchGamepads((action, family) => {
      inputMode = family;
      runAction(action);
    });

    const clock = setInterval(() => {
      if (document.visibilityState === "visible") {
        now = new Date();
      }
    }, CLOCK_REFRESH_MS);

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
  {#if view === "home" && selected}
    {#key selected.id}
      <div class="hero-art" style:background={placeholderArt(selected.name)}>
        {#if heroUrl}
          <img src={heroUrl} alt="" decoding="async" onerror={() => markBroken(heroUrl)} />
        {/if}
      </div>
    {/key}
    <div class="scrim"></div>
  {/if}

  <div class="layout">
    <header class="top-bar">
      <span class="greeting">{greeting(now)}</span>
      <nav aria-label="Main navigation">
        <button type="button" class="tab" class:active={view === "home"} aria-current={view === "home" ? "page" : undefined} onclick={() => switchView("home")}>
          Home
        </button>
        <button type="button" class="tab" class:active={view === "library"} aria-current={view === "library" ? "page" : undefined} onclick={() => switchView("library")}>
          Library
        </button>
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
    {:else if view === "library"}
      <LibraryView
        games={libraryGames}
        {filters}
        activeFilter={libraryFilter}
        sort={librarySort}
        focusedIndex={libraryZone === "grid" ? libraryIndex : null}
        focusedControl={libraryZone === "controls" ? controlIndex : null}
        {runningGameId}
        {now}
        {isBroken}
        onBroken={markBroken}
        onFilter={selectFilter}
        onCycleSort={cycleSort}
        onFocusGame={(index) => {
          libraryZone = "grid";
          libraryIndex = index;
        }}
        onActivate={playOrReturn}
        onOptions={openMenu}
        onColumns={(columns) => (libraryColumns = columns)}
      />
    {:else if !selected && allGames.length > 0}
      <section class="message">
        <p class="message-title">All your games are hidden</p>
        <p class="message-body">Open the Library to bring some back to Home.</p>
        <button type="button" class="secondary" onclick={() => switchView("library")}>Open Library</button>
      </section>
    {:else if !selected}
      <section class="message">
        <p class="message-title">No installed games found</p>
        <p class="message-body">Install a game on Steam or Epic Games, then refresh.</p>
        <button type="button" class="secondary" onclick={refreshLibrary}>Refresh</button>
      </section>
    {:else}
      <div class="main-row">
        <HomeHero
          game={selected}
          running={selectedIsRunning}
          {now}
          error={launchError}
          {isBroken}
          onBroken={markBroken}
          onPrimary={() => selected && playOrReturn(selected)}
        />
        <FriendsPanel
          result={friendsResult}
          focusedIndex={homeZone === "friends" ? friendIndex : null}
          panelFocused={homeZone === "friends" && friendsNeedSetup}
          onConnect={() => (keyDialogOpen = true)}
          onRetry={refreshFriends}
          onJoin={join}
        />
      </div>

      <section class="shelf" aria-label="Recently played">
        <h2>Recently played</h2>
        <Shelf
          games={homeGames}
          {selectedIndex}
          {runningGameId}
          {isBroken}
          onBroken={markBroken}
          onSelect={(index) => (selectedIndex = index)}
          onLaunch={playOrReturn}
          onOptions={(index) => {
            selectedIndex = index;
            openMenu(homeGames[index] ?? null);
          }}
        />
      </section>
    {/if}

    {#if view === "library" && launchError}
      <p class="error" role="alert">{launchError}</p>
    {/if}

    <Hints mode={inputMode} {hints} />
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

  {#if menuGame}
    <GameMenu
      title={menuGame.name}
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
    border: 0;
    border-radius: 999px;
    background: transparent;
    font-size: 15px;
    font-weight: 500;
    cursor: pointer;
    transition:
      background 0.2s ease,
      color 0.2s ease;
  }

  .tab.active {
    background: var(--text);
    color: var(--bg);
    font-weight: 600;
  }

  .main-row {
    flex: 1;
    min-height: 0;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 48px;
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

  .secondary {
    height: 58px;
    padding: 0 34px;
    border-radius: 999px;
    background: var(--glass);
    border: 1px solid var(--glass-border);
    font-size: 18px;
    font-weight: 600;
    cursor: pointer;
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

  .error {
    margin: 0 0 12px;
    font-size: 15px;
    color: #ff8a80;
  }

  @keyframes fade-in {
    from {
      opacity: 0;
    }
  }
</style>
