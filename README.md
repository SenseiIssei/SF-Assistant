# SF Assistant — Shakes & Fidget bot (SF bot / S&F bot)

[<img src="https://storage.ko-fi.com/cdn/kofi3.png?v=3" height="20" alt="Buy me a coffee">](https://ko-fi.com/senseiissei)

An automation assistant for Shakes & Fidget that streamlines your daily routine with smart, safe defaults and a smooth, modern UI.

## About

SF Assistant is a lightweight automation companion for Shakes & Fidget. It runs the right task at the right time, prioritizes expedition rewards the way you want, and avoids spending sandwatches/glasses unless you explicitly allow it. With event-driven updates, multi-account support, and a clean dark theme, it keeps your daily routine fast and fuss‑free across accounts.

Also known as: SF bot, S&F bot, Shakes and Fidget bot, Shakes & Fidget bot, Shakes&Fidget bot.

> Note: S&F has introduced rate limiting that impacts manual server-wide crawling. SF Assistant focuses on in-game automation and quality-of-life features rather than full-server crawls.

## Search keywords

shakes and fidget bot, shakes & fidget bot, sf bot, s&f bot, shakes and fidget automation, shakes & fidget automation, sf automation, sf automation bot, s&f automation bot, shakes fidget bot

## Highlights

- Primary task exclusivity: Only one of Expeditions, Tavern, or City Guard runs at a time to avoid conflicts and waste.
- Expedition reward priority: Choose between sensible presets (default: Mushrooms > Gold > Eggs) to guide automation decisions.
- Safety first: Sandwatches/glasses are off by default. Explicit toggles are available for Tavern and Expeditions.
- Event-driven UI: No periodic refresh timers—snappy and low-flicker updates.
- Custom Charcoal/Orange theme: Clean dark aesthetic with an accent; theme picker still supported.
- Handy toggles: A lightning icon in the Expeditions header quickly enables/disables auto-expeditions.
- Multi-account aware: Per-character, per-server settings are stored when you log in with “Remember me”.

## What it automates

SF Assistant can assist with:

- Tavern: Quest handling, optional beer purchase via mushrooms (within your set budget), optional glasses usage when enabled.
- Expeditions: Reward-aware expedition selection, optional glasses usage, and quick lightning toggle in the UI.
- Dungeons, Pets, Guild: Optional automation toggles for routine checks and fights where applicable.
- Side-actions: Opportunistic actions can run while a primary task is active, where safe and available.

The app enforces that only one primary activity (Tavern, Expeditions, City Guard) runs at any moment to prevent conflicts.

## UI overview

- Automation page
	- Automations: Top-level toggles for battle, lure, tavern, expeditions, dungeons, pets, and guild.
	- Strategy: Select your mission strategy for quests.
	- Expeditions section: Includes a header lightning toggle, a “Use glasses to skip waits” safety toggle, and a “Reward priority” selector.
	- Tavern options: “Buy beer with mushrooms” and “Use glasses to skip waits” toggles.
	- Mushroom budgets: Sliders for daily beer, dungeon skip, and pet skip caps.

## Installation

SF Assistant is a Rust application and can be built from source on Windows, macOS, or Linux.

Prerequisites:
- Rust toolchain (stable)

Build from source:

```powershell
# Windows PowerShell
cargo build --release
```

The executable will be in `target/release/`.
Official release archives include platform-specific names (Windows: `SFAssistant.exe`; macOS/Linux: `SFAssistant`).

## Using SF Assistant

1. Launch the app and log in to your account(s).
2. Check “Remember me” on the login screen to persist per-character, per-server settings.
3. Open the Automation page and enable the tasks you want.
4. Configure Expedition reward priority and glasses usage to match your preferences.
5. Set mushroom budgets to cap spending on beer, dungeon skips, and pet skips.

Your settings are stored per character and server automatically (after using “Remember me”).

## Safety defaults and controls

- No glasses by default: The app will not spend sandwatches/glasses unless you explicitly enable the corresponding toggle for Tavern or Expeditions.
- Reward priorities: Expeditions follow your selected priority preset so you get the rewards you value most.
- Exclusivity: Only one primary activity at a time, to avoid clashing timers or wasted resources.

## Troubleshooting

- Rendering/UI: The app uses wgpu via Iced; if you encounter rendering issues, update your GPU drivers and try again.
- Rate limiting: Manual full-server crawls are limited by S&F rate-limiting. Automation features remain available and recommended.
- Logs: The app uses log4rs; if you report issues, please share logs from your session if available.

## License

This project is licensed under the MIT License. See `LICENSE` for details.

## Credits

- Original S&F Scrapbook Helper by the-marenga: https://github.com/the-marenga/sf-scrapbook-helper
	- Support the original author: [<img src="https://storage.ko-fi.com/cdn/kofi3.png?v=3" height="20" alt="Buy me a coffee">](https://ko-fi.com/J3J0ULD4J)
- sf-api by the-marenga (protocol and game API used by this app): https://github.com/the-marenga/sf-api

SF Assistant builds on the fantastic groundwork laid by the original Scrapbook Helper and sf-api projects. Thank you!