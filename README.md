<div align="center">

<img src="app/app-icon.svg" width="120" alt="Lounge icon" />

# Lounge

**Your PC games, on the couch.**

A minimal, controller-first launcher that turns your Windows library into a console-style home screen, then gets out of the way the moment you hit Play.

![Windows](https://img.shields.io/badge/Windows-10%20%7C%2011-0078D4?style=flat-square)
![Tauri](https://img.shields.io/badge/Tauri-2-24C8DB?style=flat-square&logo=tauri&logoColor=white)
![Rust](https://img.shields.io/badge/Rust-core-000000?style=flat-square&logo=rust)
![Svelte](https://img.shields.io/badge/Svelte-5-FF3E00?style=flat-square&logo=svelte&logoColor=white)
![Installer](https://img.shields.io/badge/installer-~2.5%20MB-30d158?style=flat-square)

</div>

<p align="center">
  <img src="docs/screenshots/home.jpg" alt="Lounge's Home screen: BeamNG.drive hero art with its logo and a Play button, Steam friends online on the right, and a Recently played shelf below" width="100%" />
</p>

---

Steam's Big Picture mode is heavy. Store launchers are noisy. Lounge is neither: one calm screen with your games, your friends and a big **Play** button, designed for a DualSense in your hands and the PC across the room.

## Highlights

### 🎮 Built for the controller
- Full navigation with a **PS5 DualSense**, an Xbox pad, or keyboard and mouse. Switch any time.
- On-screen hints follow what you're holding: ✕ ○ △ for PlayStation, A B Y for Xbox, keys for the keyboard.
- Haptic tick as you move between games.

### 🏠 A home screen worth landing on
- Full-bleed hero art and the game's official logo, straight from Steam's local cache.
- A *Recently played* shelf you can scroll with the d-pad, the mouse wheel or a click-and-drag.
- **Steam friends** online right now, with what they're playing and a one-press **Join**.

### 📚 A real library
- Every installed game as a grid of covers.
- Filter by store (Steam, Epic Games) and **sort by play time**, recently played, name or size.
- The line under each cover follows the sort: *145 h played*, *2 days ago*, *52 GB*.
- Hide the clutter (tools, demos, benchmarks) from Home without losing it.

<p align="center">
  <img src="docs/screenshots/library.jpg" alt="Lounge's Library: a grid of game covers sorted by play time, with store filters and a Hidden games filter" width="100%" />
</p>

### ⏯️ A Quick Menu in game
- Press the **PS button** mid-game and a slim overlay slides in: *Resume game*, *Back to Lounge*, *Quit game*.
- Lounge tracks the session on its own: it knows when a game starts, brings it back to the front, and returns to Home when you quit.

### 🪶 Light on purpose
- The main window is **destroyed** while you play, not just hidden. What's left is a tray icon and a tiny watcher.
- Native Rust core on the system WebView2. No bundled Chromium, a ~2.5 MB installer.
- Every background effect is a static gradient. No animated wallpapers eating your GPU.

### ⚙️ Feels like a console
- **Start with Windows** and **open in full screen**, both opt-in from Settings.
- Press Options (≡), Tab, or the gear icon to open Settings.

## Controls

| | DualSense | Xbox | Keyboard and mouse |
|---|---|---|---|
| Move | D-pad / left stick | D-pad / left stick | Arrow keys |
| Play / select | ✕ | A | Enter / click |
| Back | ○ | B | Esc |
| Game options | △ | Y | Right-click |
| Home ⇄ Library | L1 / R1 | LB / RB | Q / E |
| Settings | Options | Menu | Tab |
| Quick Menu (in game) | PS | | |
| Full screen | | | F11 |

## Stores

| Store | Library | Artwork | Play time | Launch | Session tracking |
|---|---|---|---|---|---|
| Steam | ✅ | ✅ local cache, CDN fallback | ✅ | ✅ | ✅ |
| Epic Games | ✅ | | | ✅ | 🔜 |
| Rockstar Games | 🔜 | | | | |

Lounge reads what your launchers already keep on disk: Steam library folders, app manifests, the artwork cache and your local play time. No sign-in, no scraping.

## Privacy

- Everything about your library is read **locally**.
- Steam friends are optional. They need a free Steam Web API key, which Lounge stores in **Windows Credential Manager**, never in a file or in this repo.
- Lounge only opens `steam://` and `com.epicgames.launcher://` launch links. Anything else is refused.

## Install

Lounge doesn't ship prebuilt releases yet. Build the installer yourself (see below), then run `Lounge_x.y.z_x64-setup.exe`. It installs for your user account only, no admin rights needed.

> The installer isn't code-signed, so Windows SmartScreen may warn about an unknown publisher. Choose **More info → Run anyway**.

## Build from source

**You'll need** Windows 10 or 11, [Node.js](https://nodejs.org/) 20+, the [Rust toolchain](https://rustup.rs/), and the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) (the Microsoft C++ Build Tools and WebView2, which Windows 11 already has).

```bash
cd app
npm install
```

Run it in development, with hot reload:

```bash
npm run tauri dev
```

Build the release app and its installer, in `app/src-tauri/target/release/bundle/nsis/`:

```bash
npm run tauri build
```

Run the tests:

```bash
npm test
```

```bash
cd src-tauri && cargo test
```

## How it's built

```
app/
├─ src/                    SvelteKit front end (Svelte 5, TypeScript)
│  ├─ routes/+page.svelte    Home and Library
│  ├─ routes/quick-menu/     The in-game overlay
│  └─ lib/                   Components, gamepad input, library logic
└─ src-tauri/src/          Rust core
   ├─ steam.rs, epic.rs      Store detection (with a small VDF parser)
   ├─ watcher.rs, session.rs Game sessions: start, focus, quit
   ├─ pad.rs, quick_menu.rs  DualSense over HID and the PS button overlay
   ├─ friends.rs             Steam friends via the Web API
   └─ window.rs, autostart.rs Window lifecycle and Start with Windows
```

- **Rust** does the heavy lifting: scanning stores, trimming and caching logos, reading the DualSense over HID, tracking the running game, and managing the window.
- **Svelte** draws the interface and reads controllers through the Gamepad API while Lounge is on screen.
- They talk through a small set of Tauri commands and events.

## Roadmap

- [ ] Session tracking and *Now playing* for Epic games
- [ ] Rockstar Games Launcher support
- [ ] Single instance: reopening Lounge brings back the running one
- [ ] Subtle navigation sounds
- [ ] Prebuilt releases

---

<div align="center">
<sub>Made for playing PC games from the couch. Built with Tauri, Rust and Svelte.</sub>
</div>
