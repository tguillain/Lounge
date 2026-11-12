<script lang="ts">
  import { onMount } from "svelte";
  import { SvelteMap, SvelteSet } from "svelte/reactivity";
  import AchievementsView from "$lib/components/AchievementsView.svelte";
  import BatteryIndicator from "$lib/components/BatteryIndicator.svelte";
  import FriendsPanel from "$lib/components/FriendsPanel.svelte";
  import GameMenu, { type MenuItem } from "$lib/components/GameMenu.svelte";
  import Hints, { type Hint } from "$lib/components/Hints.svelte";
  import HomeHero from "$lib/components/HomeHero.svelte";
  import LaunchOverlay from "$lib/components/LaunchOverlay.svelte";
  import LibraryView from "$lib/components/LibraryView.svelte";
  import SearchKeyboard from "$lib/components/SearchKeyboard.svelte";
  import SettingsView from "$lib/components/SettingsView.svelte";
  import Shelf from "$lib/components/Shelf.svelte";
  import SteamKeyDialog from "$lib/components/SteamKeyDialog.svelte";
  import {
    backToGame,
    checkForUpdate,
    forgetSteamApiKey,
    getAudioOutputs,
    getPreferences,
    getSession,
    imageUrl,
    installUpdate,
    joinFriend,
    launchGame,
    loadAchievements,
    loadFriends,
    loadLibrary,
    onGameSession,
    onOpenSettings,
    onPowerButton,
    onSteamDownloads,
    onUpdateProgress,
    openReleasesPage,
    openSteamApiKeyPage,
    pulseDualSense,
    quitLounge,
    runPower,
    releaseLightbar,
    saveSteamApiKey,
    setAudioOutput,
    setGameHidden,
    setLightbar,
    setQuickMenuShortcut,
    setStartWithWindows,
    setToggle,
    takeSettingsRequest,
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
  import { settingsRows, type SettingsRow, type SteamKeyState } from "$lib/settings";
  import { playSound, soundFor } from "$lib/sounds";
  import { FIRST_KEY, keyAt, matchesSearch, moveOnKeyboard, typeKey, type KeyPosition } from "$lib/search";
  import { applyDownloads } from "$lib/downloads";
  import { countLabel, isSecret, orderAchievements } from "$lib/achievements";
  import { POWER_CHOICES, confirmationOf, type PowerChoice } from "$lib/power";
  import { currentOutputLabel, markCurrent, outputTitle } from "$lib/audio";
  import { captureShortcut, shortcutLabel } from "$lib/shortcut";
  import type { Achievement, AchievementsResult, AudioOutput, Friend, FriendsResult, Game, LibraryState, Preferences, Toggle, UpdateStatus } from "$lib/types";

  type View = "home" | "library" | "settings" | "achievements";

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
    Tab: "menu",
  };

  const LAUNCH_OVERLAY_TIMEOUT_MS = 20_000;
  const CLOCK_REFRESH_MS = 15_000;
  const FRIENDS_REFRESH_MS = 60_000;
  const VISIBLE_FRIENDS = 5;
  const UPDATE_CHECK_MS = 6 * 60 * 60 * 1000;
  const LIGHTBAR_DELAY_MS = 150;
  const ACHIEVEMENTS_DELAY_MS = 300;

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

  let searchQuery = $state("");
  let searchOpen = $state(false);
  let searchWithKeys = $state(true);
  let keyPosition = $state<KeyPosition>(FIRST_KEY);

  const achievementsByGame = new SvelteMap<string, AchievementsResult>();
  let achievementsGame = $state<Game | null>(null);
  let achievementIndex = $state(0);
  const revealedAchievements = new SvelteSet<string>();
  let viewBeforeAchievements: "home" | "library" = "home";

  let powerOpen = $state(false);
  let powerIndex = $state(0);
  let powerConfirm = $state<PowerChoice | null>(null);
  let powerNote = $state<string | null>(null);
  let choosingOutput = $state(false);
  let outputs = $state<AudioOutput[] | null>(null);

  let menuGame = $state<Game | null>(null);
  let menuIndex = $state(0);

  let preferences = $state<Preferences | null>(null);
  let settingsIndex = $state(0);
  let settingsNote = $state<string | null>(null);
  let capturingShortcut = $state(false);
  let viewBeforeSettings: "home" | "library" = "home";
  let update = $state<UpdateStatus>({ state: "idle" });
  const showFriends = $derived(preferences?.showFriends ?? true);

  const allGames = $derived(library.status === "ready" ? library.games : []);
  const homeGames = $derived(allGames.filter((game) => !game.hidden));
  const selected = $derived<Game | null>(homeGames[selectedIndex] ?? null);
  const selectedIsRunning = $derived(selected !== null && selected.id === runningGameId);

  const filters = $derived(availableFilters(allGames));
  const libraryGames = $derived(
    sortGames(
      filterGames(allGames, libraryFilter).filter((game) => matchesSearch(game.name, searchQuery)),
      librarySort,
    ),
  );
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

  const lightbarGame = $derived<Game | null>(
    view === "library" ? focusedLibraryGame : view === "home" ? selected : view === "achievements" ? achievementsGame : null,
  );

  const achievementsOf = (game: Game | null): AchievementsResult | null =>
    game?.store === "steam" ? (achievementsByGame.get(game.id) ?? null) : null;

  const shownAchievements = $derived<AchievementsResult>(achievementsOf(achievementsGame) ?? { status: "loading" });
  const orderedAchievements = $derived<Achievement[]>(
    shownAchievements.status === "ready" ? orderAchievements(shownAchievements.achievements) : [],
  );

  const fetchAchievements = async (game: Game, refresh = false) => {
    const known = achievementsByGame.get(game.id);
    if (game.store !== "steam" || (known && known.status !== "failed" && !refresh)) {
      return;
    }
    if (!known || known.status !== "ready") {
      achievementsByGame.set(game.id, { status: "loading" });
    }
    try {
      achievementsByGame.set(game.id, await loadAchievements(game));
    } catch (error) {
      achievementsByGame.set(game.id, { status: "failed", message: `Couldn't load achievements: ${error}` });
    }
  };

  $effect(() => {
    const game = view === "home" ? selected : null;
    if (!game || game.store !== "steam") {
      return;
    }
    const timer = setTimeout(() => fetchAchievements(game), ACHIEVEMENTS_DELAY_MS);
    return () => clearTimeout(timer);
  });

  $effect(() => {
    const game = lightbarGame;
    if (!game || !preferences?.lightbar) {
      return;
    }
    const timer = setTimeout(() => setLightbar(game).catch(() => {}), LIGHTBAR_DELAY_MS);
    return () => clearTimeout(timer);
  });

  const heroUrl = $derived(selected ? usableImage(selected.images.hero) : null);

  const feedback = () => {
    if (inputMode === "keyboard" || preferences?.vibration === false) {
      return;
    }
    pulse();
    if (inputMode === "playstation") {
      pulseDualSense();
    }
  };

  const refreshLibrary = async () => {
    try {
      const selectedId = selected?.id;
      const loaded = await loadLibrary();
      library = { status: "ready", games: loaded };
      selectedIndex = Math.max(0, homeGames.findIndex((game) => game.id === selectedId));
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
      fetchAchievements(game);
    }
  };

  const openAchievements = (game: Game) => {
    if (view === "home" || view === "library") {
      viewBeforeAchievements = view;
    }
    achievementsGame = game;
    achievementIndex = 0;
    revealedAchievements.clear();
    view = "achievements";
    fetchAchievements(game, true);
  };

  const closeAchievements = () => {
    view = viewBeforeAchievements;
    achievementsGame = null;
  };

  const activateAchievement = (achievement: Achievement) => {
    if (isSecret(achievement)) {
      revealedAchievements.add(achievement.id);
    }
  };

  const runAchievementsAction = (action: PadAction) => {
    if (action === "up" || action === "down") {
      const next = Math.max(0, Math.min(orderedAchievements.length - 1, achievementIndex + (action === "down" ? 1 : -1)));
      if (next !== achievementIndex) {
        achievementIndex = next;
        feedback();
      }
    } else if (action === "confirm") {
      const focused = orderedAchievements[achievementIndex];
      if (focused) {
        activateAchievement(focused);
      } else if (shownAchievements.status === "failed" && achievementsGame) {
        fetchAchievements(achievementsGame, true);
      }
    } else if (action === "back" || action === "options") {
      closeAchievements();
    } else if (action === "previousTab" || action === "nextTab") {
      closeAchievements();
      switchView(action === "nextTab" ? "library" : "home");
    } else if (action === "menu") {
      openSettings();
    }
  };

  const loadPreferences = async () => {
    try {
      preferences = await getPreferences();
    } catch (error) {
      settingsNote = `Couldn't read your settings: ${error}`;
    }
  };

  const lookForUpdate = async () => {
    if (!preferences?.canUpdate || update.state === "checking" || update.state === "installing") {
      return;
    }
    update = { state: "checking" };
    try {
      const found = await checkForUpdate();
      update = found ? { state: "available", version: found.version } : { state: "up-to-date" };
    } catch {
      update = { state: "failed", message: "Couldn't check for updates. Check your connection and try again." };
    }
  };

  const installAvailableUpdate = async () => {
    if (update.state !== "available") {
      return;
    }
    update = { state: "installing", version: update.version, percent: null };
    try {
      await installUpdate();
    } catch (error) {
      update = { state: "failed", message: `Couldn't install the update: ${error}` };
    }
  };

  const openSettings = () => {
    if (view === "achievements") {
      viewBeforeSettings = viewBeforeAchievements;
      achievementsGame = null;
    } else if (view !== "settings") {
      viewBeforeSettings = view;
    }
    settingsIndex = 0;
    settingsNote = null;
    view = "settings";
    loadPreferences().then(() => {
      if (update.state === "up-to-date" || update.state === "failed") {
        lookForUpdate();
      }
    });
  };

  const closeSettings = () => {
    capturingShortcut = false;
    view = viewBeforeSettings;
    homeZone = "shelf";
  };

  const flipSetting = async (key: Toggle | "startWithWindows") => {
    if (!preferences) {
      return;
    }
    const enabled = !preferences[key];
    preferences = { ...preferences, [key]: enabled };
    settingsNote = key === "startFullscreen" ? "Lounge will open in this mode next time." : null;
    try {
      await (key === "startWithWindows" ? setStartWithWindows(enabled) : setToggle(key, enabled));
      if (key === "vibration" && enabled) {
        feedback();
      }
      if (key === "lightbar" && !enabled) {
        await releaseLightbar();
      }
    } catch (error) {
      preferences = { ...preferences, [key]: !enabled };
      settingsNote = `Couldn't save this setting: ${error}`;
    }
  };

  const steamKeyState = $derived<SteamKeyState>(
    friendsResult.status === "loading" ? "unknown" : friendsResult.status === "not-configured" ? "missing" : "connected",
  );

  const rows = $derived<SettingsRow[]>(preferences ? settingsRows(preferences, steamKeyState, update, capturingShortcut) : []);

  const saveShortcut = async (shortcut: string | null) => {
    if (!preferences) {
      return;
    }
    capturingShortcut = false;
    try {
      await setQuickMenuShortcut(shortcut);
      preferences = { ...preferences, quickMenuShortcut: shortcut };
      settingsNote = shortcut
        ? `Press ${shortcutLabel(shortcut)} during a game to open the Quick Menu.`
        : "The Quick Menu shortcut is off.";
    } catch (error) {
      settingsNote = `${error}`;
    }
  };

  const listenForShortcut = (event: KeyboardEvent) => {
    event.preventDefault();
    const result = captureShortcut(event);
    if (result.kind === "waiting") {
      return;
    }
    if (result.kind === "cancel") {
      capturingShortcut = false;
      settingsNote = null;
    } else if (result.kind === "clear") {
      saveShortcut(null);
    } else if (result.kind === "invalid") {
      settingsNote = result.message;
    } else {
      saveShortcut(result.shortcut);
    }
  };

  const activateSetting = (row: SettingsRow) => {
    const wasCapturing = capturingShortcut;
    capturingShortcut = false;
    if (row.kind === "action" && row.action === "quick-menu-shortcut") {
      capturingShortcut = !wasCapturing;
      settingsNote = null;
    } else if (row.kind === "toggle") {
      if (row.available) {
        flipSetting(row.toggle);
      }
    } else if (row.action === "steam-key") {
      keyDialogOpen = true;
    } else if (row.action === "remove-key") {
      forgetKey();
      settingsIndex = Math.max(0, settingsIndex - 1);
    } else if (row.action === "update") {
      if (row.inert) {
        return;
      }
      if (update.state === "available") {
        installAvailableUpdate();
      } else {
        lookForUpdate();
      }
    } else {
      openReleasesPage().catch((error) => (settingsNote = `Couldn't open the releases page: ${error}`));
    }
  };

  const gameMenuItems = $derived.by<MenuItem[]>(() => {
    const game = menuGame;
    if (!game) {
      return [];
    }
    const achievements = achievementsOf(game);
    return [
      ...(game.store === "steam" && achievements?.status !== "none"
        ? [
            {
              id: "achievements",
              label: "Achievements",
              value: achievements ? (countLabel(achievements) ?? undefined) : undefined,
              run: () => {
                closeMenu();
                openAchievements(game);
              },
            },
          ]
        : []),
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

  const menuItems = $derived(gameMenuItems);

  const loadOutputs = async () => {
    try {
      outputs = await getAudioOutputs();
    } catch (error) {
      outputs = [];
      powerNote = `Couldn't read the sound outputs: ${error}`;
    }
  };

  const openPower = () => {
    menuGame = null;
    powerConfirm = null;
    choosingOutput = false;
    powerNote = null;
    powerIndex = 0;
    powerOpen = true;
    loadOutputs();
  };

  const closePower = () => {
    powerOpen = false;
    powerConfirm = null;
    choosingOutput = false;
  };

  const openOutputs = () => {
    choosingOutput = true;
    powerNote = null;
    powerIndex = Math.max(0, outputs?.findIndex((output) => output.current) ?? 0);
  };

  const leaveOutputs = () => {
    choosingOutput = false;
    powerNote = null;
    powerIndex = 0;
  };

  const pickOutput = async (output: AudioOutput) => {
    if (output.current || !outputs) {
      return;
    }
    powerNote = null;
    try {
      await setAudioOutput(output);
      outputs = markCurrent(outputs, output.id);
      if (preferences?.sounds !== false) {
        playSound("select");
      }
    } catch (error) {
      powerNote = String(error);
    }
  };

  const carryOut = async (choice: PowerChoice) => {
    powerNote = null;
    try {
      if (choice === "quit") {
        await quitLounge();
      } else {
        closePower();
        await runPower(choice);
      }
    } catch (error) {
      powerOpen = true;
      powerConfirm = null;
      powerNote = String(error);
    }
  };

  const choosePower = (choice: PowerChoice) => {
    if (confirmationOf(choice)) {
      powerConfirm = choice;
      powerIndex = 0;
    } else {
      carryOut(choice);
    }
  };

  const cancelPowerConfirm = () => {
    powerIndex = Math.max(0, POWER_CHOICES.findIndex((choice) => choice.id === powerConfirm)) + 1;
    powerConfirm = null;
  };

  const powerConfirmation = $derived(powerConfirm ? confirmationOf(powerConfirm) : null);

  const powerItems = $derived.by<MenuItem[]>(() => {
    const choice = powerConfirm;
    if (choice && powerConfirmation) {
      return [
        { id: "confirm", label: powerConfirmation.action, run: () => carryOut(choice) },
        { id: "cancel", label: "Cancel", run: cancelPowerConfirm },
      ];
    }
    if (choosingOutput) {
      if (outputs?.length === 0) {
        return [{ id: "no-output", label: "No sound output found", run: leaveOutputs }];
      }
      return (outputs ?? []).map((output) => ({
        id: output.id,
        label: outputTitle(output),
        value: output.current ? "In use" : undefined,
        run: () => pickOutput(output),
      }));
    }
    return [
      { id: "sound", label: "Sound output", value: currentOutputLabel(outputs) ?? "…", run: openOutputs },
      ...POWER_CHOICES.map((entry) => ({ id: entry.id, label: entry.label, run: () => choosePower(entry.id) })),
    ];
  });

  const powerTitle = $derived(choosingOutput ? "Sound output" : (powerConfirmation?.title ?? "Control Center"));
  const outputsHint = $derived(
    choosingOutput && outputs?.length === 1 ? "Turn on your TV or plug in headphones to see more outputs here." : null,
  );

  const runPowerAction = (action: PadAction) => {
    if (action === "up" || action === "down") {
      const step = action === "down" ? 1 : -1;
      powerIndex = (powerIndex + step + powerItems.length) % powerItems.length;
      feedback();
    } else if (action === "confirm") {
      powerItems[powerIndex]?.run();
    } else if (action === "back") {
      if (powerConfirm) {
        cancelPowerConfirm();
      } else if (choosingOutput) {
        leaveOutputs();
      } else {
        closePower();
      }
    } else if (action === "power" || action === "menu") {
      closePower();
    }
  };

  const switchView = (target: "home" | "library") => {
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

  const setSearch = (query: string) => {
    searchQuery = query;
    libraryIndex = 0;
  };

  const openSearch = () => {
    if (view !== "library") {
      switchView("library");
    }
    searchWithKeys = inputMode !== "keyboard";
    keyPosition = FIRST_KEY;
    searchOpen = true;
  };

  const closeSearch = () => {
    searchOpen = false;
    libraryZone = "grid";
    libraryIndex = 0;
  };

  const pressSearchKey = (position: KeyPosition) => {
    keyPosition = position;
    const key = keyAt(position);
    if (key.kind === "done") {
      closeSearch();
    } else {
      setSearch(typeKey(searchQuery, key));
    }
  };

  const runSearchAction = (action: PadAction) => {
    if (action === "left" || action === "right" || action === "up" || action === "down") {
      const next = moveOnKeyboard(keyPosition, action);
      if (keyAt(next) !== keyAt(keyPosition)) {
        keyPosition = next;
        feedback();
      }
    } else if (action === "confirm") {
      if (searchWithKeys) {
        pressSearchKey(keyPosition);
      } else {
        closeSearch();
      }
    } else if (action === "search") {
      setSearch(searchQuery.slice(0, -1));
    } else if (action === "back" || action === "menu") {
      closeSearch();
    }
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
    } else if (action === "back" || action === "options" || action === "menu") {
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
    } else if (action === "up" && showFriends && (visibleFriends.length > 0 || friendsNeedSetup)) {
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
      leaveLibrary();
    }
  };

  const leaveLibrary = () => {
    if (searchQuery) {
      setSearch("");
    } else {
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
      leaveLibrary();
    }
  };

  const runSettingsAction = (action: PadAction) => {
    if (capturingShortcut && action !== "confirm") {
      capturingShortcut = false;
      settingsNote = null;
      if (action === "back" || action === "menu") {
        return;
      }
    }
    if (action === "up" || action === "down") {
      const next = Math.max(0, Math.min(rows.length - 1, settingsIndex + (action === "down" ? 1 : -1)));
      if (next !== settingsIndex) {
        settingsIndex = next;
        feedback();
      }
    } else if (action === "confirm" && rows[settingsIndex]) {
      activateSetting(rows[settingsIndex]);
    } else if (action === "back" || action === "menu") {
      closeSettings();
    }
  };

  const runAction = (action: PadAction) => {
    if (keyDialogOpen) {
      if (action === "back") {
        keyDialogOpen = false;
      }
      return;
    }
    if (powerOpen) {
      runPowerAction(action);
      return;
    }
    if (action === "power") {
      if (!runningGameId && !launching && !searchOpen) {
        openPower();
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
    if (searchOpen) {
      runSearchAction(action);
      return;
    }
    if (view === "achievements") {
      runAchievementsAction(action);
      return;
    }
    if (action === "search" && view !== "settings") {
      openSearch();
      return;
    }
    if (action === "previousTab" || action === "nextTab") {
      switchView(action === "nextTab" ? "library" : "home");
    } else if (view === "settings") {
      runSettingsAction(action);
    } else if (action === "menu") {
      openSettings();
    } else if (view === "home") {
      runHomeAction(action);
    } else {
      runLibraryAction(action);
    }
  };

  const openLayers = () =>
    (view === "settings" || view === "achievements" ? 1 : 0) + (menuGame ? 1 : 0) + (powerOpen ? 1 : 0) + (keyDialogOpen ? 1 : 0) + (searchOpen ? 1 : 0);

  const focusKey = () =>
    [
      view,
      homeZone,
      selectedIndex,
      friendIndex,
      libraryZone,
      libraryIndex,
      controlIndex,
      libraryFilter,
      librarySort,
      settingsIndex,
      menuIndex,
      powerIndex,
      powerConfirm,
      choosingOutput,
      achievementIndex,
      revealedAchievements.size,
      searchQuery,
      keyPosition.row,
      keyPosition.column,
    ].join("|");

  const act = (action: PadAction) => {
    const layersBefore = openLayers();
    const focusBefore = focusKey();
    const launchingBefore = launching;
    runAction(action);
    if (preferences?.sounds === false) {
      return;
    }
    const sound = soundFor(action, {
      layersBefore,
      layersAfter: openLayers(),
      focusMoved: focusKey() !== focusBefore,
      launched: launching !== null && launching !== launchingBefore,
    });
    if (sound) {
      playSound(sound);
    }
  };

  const handleKeydown = (event: KeyboardEvent) => {
    inputMode = "keyboard";
    if (keyDialogOpen) {
      return;
    }
    if (capturingShortcut && view === "settings") {
      listenForShortcut(event);
      return;
    }
    if (searchOpen) {
      searchWithKeys = false;
      if (event.key === "Escape" || event.key === "Enter") {
        event.preventDefault();
        act(event.key === "Escape" ? "back" : "confirm");
      }
      return;
    }
    if (event.key === "/" || ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "f")) {
      event.preventDefault();
      act("search");
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
      act(action);
    }
  };

  const hints = $derived.by<Hint[]>(() => {
    if (searchOpen) {
      return searchWithKeys
        ? [
            { action: "navigate", label: "Move" },
            { action: "confirm", label: "Type" },
            { action: "search", label: "Delete" },
            { action: "menu", label: "Done" },
            { action: "back", label: "Close" },
          ]
        : [
            { action: "confirm", label: "Show results" },
            { action: "back", label: "Close" },
          ];
    }
    if (view === "achievements") {
      const focused = orderedAchievements[achievementIndex];
      return [
        ...(orderedAchievements.length > 1 ? [{ action: "navigate" as const, label: "Scroll" }] : []),
        ...(focused && isSecret(focused) && !revealedAchievements.has(focused.id) ? [{ action: "confirm" as const, label: "Reveal" }] : []),
        ...(shownAchievements.status === "failed" ? [{ action: "confirm" as const, label: "Try again" }] : []),
        { action: "back", label: "Back" },
      ];
    }
    if (view === "settings" && capturingShortcut) {
      return [{ action: "back", label: "Cancel" }];
    }
    if (view === "settings") {
      const row = rows[settingsIndex];
      return [
        { action: "navigate", label: "Navigate" },
        ...(row && !(row.kind === "action" && row.inert)
          ? [{ action: "confirm" as const, label: row.kind === "toggle" ? "Turn on or off" : row.action === "update" ? (row.value ?? "Select") : "Select" }]
          : []),
        { action: "back", label: "Back" },
      ];
    }
    if (view === "library") {
      return [
        { action: "navigate", label: "Navigate" },
        {
          action: "confirm",
          label: libraryZone === "controls" ? "Select" : focusedLibraryGame?.id === runningGameId ? "Back to game" : "Play",
        },
        { action: "options", label: "Options" },
        { action: "search", label: "Search" },
        ...(searchQuery ? [{ action: "back" as const, label: "Clear search" }] : [{ action: "tabs" as const, label: "Home" }]),
        { action: "menu", label: "Settings" },
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
      { action: "menu", label: "Settings" },
      ...(runningGameId ? [] : [{ action: "power" as const, label: "Control Center" }]),
      { action: "fullscreen", label: "Full screen" },
    ];
  });

  onMount(() => {
    refreshLibrary();
    refreshFriends();
    loadPreferences().then(lookForUpdate);

    const stopGamepads = watchGamepads((action, family) => {
      inputMode = family;
      act(action);
    });

    const clock = setInterval(() => {
      if (document.visibilityState === "visible") {
        now = new Date();
      }
    }, CLOCK_REFRESH_MS);

    const friendsTimer = setInterval(() => {
      if (document.visibilityState === "visible" && showFriends && friendsResult.status === "ready") {
        refreshFriends();
      }
    }, FRIENDS_REFRESH_MS);

    const updateTimer = setInterval(() => {
      if (update.state !== "available") {
        lookForUpdate();
      }
    }, UPDATE_CHECK_MS);

    const stopDownloads = onSteamDownloads((downloads) => {
      if (library.status !== "ready") {
        return;
      }
      const result = applyDownloads(library.games, downloads);
      library = { status: "ready", games: result.games };
      if (result.needsRescan) {
        refreshLibrary();
      }
    });

    const stopPowerButton = onPowerButton(() => {
      inputMode = "playstation";
      act("power");
    });

    const openSettingsIfAsked = () =>
      takeSettingsRequest()
        .then((asked) => {
          if (asked && view !== "settings") {
            openSettings();
          }
        })
        .catch(() => {});
    openSettingsIfAsked();
    const stopOpenSettings = onOpenSettings(openSettingsIfAsked);

    const stopUpdateProgress = onUpdateProgress((percent) => {
      if (update.state === "installing") {
        update = { ...update, percent };
      }
    });

    getSession()
      .then((session) => (runningGameId = session ? `steam:${session.appId}` : null))
      .catch(() => (runningGameId = null));

    const stopListening = onGameSession(
      (gameId) => {
        runningGameId = gameId;
        closeLaunchOverlay();
      },
      (gameId) => {
        runningGameId = null;
        refreshLibrary();
        if (achievementsByGame.has(gameId)) {
          const played = allGames.find((game) => game.id === gameId);
          achievementsByGame.delete(gameId);
          if (played) {
            fetchAchievements(played);
          }
        }
      },
    );

    return () => {
      clearInterval(clock);
      clearInterval(friendsTimer);
      clearInterval(updateTimer);
      stopUpdateProgress.then((stop) => stop());
      stopPowerButton.then((stop) => stop());
      stopOpenSettings.then((stop) => stop());
      stopDownloads.then((stop) => stop());
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
  {#if view === "library" || view === "settings"}
    <div class="ambient" aria-hidden="true"></div>
  {/if}
  {#if view === "achievements" && achievementsGame}
    {@const art = usableImage(achievementsGame.images.hero)}
    <div class="hero-art faded" style:background={placeholderArt(achievementsGame.name)}>
      {#if art}
        <img src={art} alt="" decoding="async" onerror={() => markBroken(art)} />
      {/if}
    </div>
    <div class="scrim deep"></div>
  {/if}
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
      <div class="top-right">
        <BatteryIndicator />
        <span class="clock">{clockLabel(now)}</span>
        <button type="button" class="settings-button" class:active={powerOpen} aria-label="Control Center" onclick={openPower}>
          <svg aria-hidden="true" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 3v9" /><path d="M6.4 6.4a8 8 0 1 0 11.2 0" /></svg>
        </button>
        <button
          type="button"
          class="settings-button"
          class:active={view === "settings"}
          class:has-update={update.state === "available"}
          aria-label={update.state === "available" ? "Settings, update available" : "Settings"}
          aria-current={view === "settings" ? "page" : undefined}
          onclick={() => (view === "settings" ? closeSettings() : openSettings())}
        >
          <svg aria-hidden="true" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6z" />
            <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
          </svg>
        </button>
      </div>
    </header>

    {#if view === "settings"}
      <SettingsView
        {rows}
        focusedIndex={settingsIndex}
        note={settingsNote}
        onFocus={(index) => (settingsIndex = index)}
        onActivate={activateSetting}
      />
    {:else if view === "achievements" && achievementsGame}
      <AchievementsView
        game={achievementsGame}
        result={shownAchievements}
        achievements={orderedAchievements}
        focusedIndex={achievementIndex}
        revealed={revealedAchievements}
        onFocus={(index) => (achievementIndex = index)}
        onActivate={activateAchievement}
        onRetry={() => achievementsGame && fetchAchievements(achievementsGame, true)}
      />
    {:else if library.status === "loading"}
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
        search={searchQuery}
        {now}
        {isBroken}
        onBroken={markBroken}
        onFilter={selectFilter}
        onCycleSort={cycleSort}
        onSearch={openSearch}
        onClearSearch={() => setSearch("")}
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
          achievements={achievementsOf(selected)}
          onAchievements={() => selected && openAchievements(selected)}
          {isBroken}
          onBroken={markBroken}
          onPrimary={() => selected && playOrReturn(selected)}
        />
        {#if showFriends}
          <FriendsPanel
            result={friendsResult}
            focusedIndex={homeZone === "friends" ? friendIndex : null}
            panelFocused={homeZone === "friends" && friendsNeedSetup}
            onConnect={() => (keyDialogOpen = true)}
            onRetry={refreshFriends}
            onJoin={join}
          />
        {/if}
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

  {#if searchOpen}
    <SearchKeyboard
      query={searchQuery}
      position={keyPosition}
      showKeys={searchWithKeys}
      resultCount={libraryGames.length}
      onInput={setSearch}
      onPressKey={pressSearchKey}
    />
  {/if}

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

  {#if powerOpen}
    <GameMenu
      title={powerTitle}
      items={powerItems}
      focusedIndex={powerIndex}
      mode={inputMode}
      note={powerNote ?? powerConfirmation?.note ?? outputsHint}
      onFocus={(index) => (powerIndex = index)}
      onClose={closePower}
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

  .ambient {
    position: absolute;
    inset: 0 0 auto 0;
    height: 70vh;
    background:
      radial-gradient(40% 70% at 10% 0%, rgba(94, 92, 230, 0.4) 0%, transparent 100%),
      radial-gradient(36% 60% at 48% 0%, rgba(255, 55, 95, 0.2) 0%, transparent 100%),
      radial-gradient(40% 70% at 90% 0%, rgba(100, 210, 255, 0.26) 0%, transparent 100%),
      radial-gradient(30% 50% at 70% 20%, rgba(191, 90, 242, 0.22) 0%, transparent 100%);
    mask-image: linear-gradient(180deg, #000 0%, rgba(0, 0, 0, 0.5) 50%, transparent 100%);
    animation: fade-in 0.5s var(--ease);
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

  .hero-art.faded {
    height: 100vh;
    filter: blur(28px) saturate(1.2);
    transform: scale(1.08);
    opacity: 0.55;
  }

  .scrim.deep {
    background: linear-gradient(180deg, rgba(10, 10, 12, 0.55) 0%, rgba(10, 10, 12, 0.85) 45%, var(--bg) 100%);
  }

  .scrim {
    position: absolute;
    inset: 0;
    background:
      linear-gradient(90deg, rgba(10, 10, 12, 0.92) 0%, rgba(10, 10, 12, 0.6) 38%, rgba(10, 10, 12, 0) 70%),
      linear-gradient(0deg, var(--bg) 0%, var(--bg) 22%, rgba(10, 10, 12, 0.7) 40%, rgba(10, 10, 12, 0) 65%);
  }

  .layout {
    --page-gutter: 64px;
    position: relative;
    height: 100%;
    padding: 36px var(--page-gutter) 32px;
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

  .top-right {
    justify-self: end;
    display: flex;
    align-items: center;
    gap: 16px;
  }

  .clock {
    font-variant-numeric: tabular-nums;
  }

  .settings-button {
    width: 40px;
    height: 40px;
    border: 0;
    border-radius: 999px;
    background: transparent;
    display: grid;
    place-items: center;
    color: var(--text-secondary);
    cursor: pointer;
    position: relative;
    transition:
      background 0.15s ease,
      color 0.15s ease;
  }

  .settings-button.has-update::after {
    content: "";
    position: absolute;
    top: 7px;
    right: 7px;
    width: 9px;
    height: 9px;
    border-radius: 999px;
    background: var(--live);
    box-shadow: 0 0 0 2px var(--bg);
  }

  .settings-button:hover {
    background: var(--glass);
    color: var(--text);
  }

  .settings-button.active {
    background: var(--text);
    color: var(--bg);
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
