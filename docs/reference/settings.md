# Settings & preferences

Open the Preferences dialog from **PackFile → Settings** (or via its shortcut, `Ctrl+,`).

Settings are stored on the server side and cached client-side, so they're consistent across `rpfm_ui` runs and across multiple UIs talking to the same `rpfm_server`. Most changes take effect immediately; a few (notably language and other UI-layout settings) require a restart.

<!-- IMAGE: Preferences dialog open with the seven-button navigation pane on the left and the General section on the right. -->

The dialog is a single scrollable form with a left-hand navigation pane that jumps to one of seven sections. The list below is a tour, not an exhaustive reference — for the full list of setting keys see the [`rpfm_ipc::settings_keys` API docs](../../api/rpfm_ipc/settings_keys/index.html).

## Paths

The first section you'll touch as a new user. Holds two groups:

- **Game paths** — for each supported game, a collapsible "spoiler" with the game install path and (where applicable) the Assembly Kit install path.
- **Extra paths** — **MyMod base path** (root folder for [MyMod](../mymod/overview.md) projects) and **Secondary path** (extra mods folder in addition to the game's `data/`).

## General

- **Language** — RPFM's UI language. Requires a restart.
- **Default game** — which Total War game RPFM starts on.
- **Update channel** — `Stable` or `Beta`.
- **Autosave amount** and **Autosave interval** (in minutes).
- **Check for X on start** toggles for program, schema, lua-autogen and old-AK updates.
- **Allow editing of CA Packfiles**, **Disable file previews**, **Start maximized**.
- Pack-tree behaviour toggles: expand on add, include base folder on add-from-folder, delete empty folders on delete, ignore game files in AK, multifolder file picker, drag-and-drop in the pack contents tree.

## Table

UI behaviour for the table editors:

- **Adjust columns to content**, **Disable combos on tables**, **Extend last column**, **Tight table mode**, **Resize on edit**.
- **Tables use old column order** (and a TSV variant) — restores the legacy column ordering.
- **Disable UUID regeneration on DB tables** — don't auto-generate a new UUID on table save.
- **Use right-side markers**, **Enable lookups**, **Enable icons**, **Enable diff markers**.
- Colour pickers for **Added**, **Modified**, **Error**, **Warning** and **Info** marks (separate light/dark colour for each).

## Debug

Hidden behind a Debug section header rather than a feature flag, but most of these only matter to developers:

- **Check for missing table definitions**, **Enable debug menu** (exposes the hidden Debug menu in the menu bar), **Enable unit editor**, **Enable ESF editor**, **Use debug view for Unit Variant**, **Enable renderer** (only when built with `support_model_renderer`).
- **Use lazy loading** — defer decoding files until they're opened.
- Action buttons: **Clear Dependencies Cache Folder**, **Clear Autosave Folder**, **Clear Schema Folder**, **Clear Layout Settings**, **Add RPFM to Runcher Tools**.

## Diagnostics

- **Trigger diagnostics on Pack open**.
- **Trigger diagnostics on table edit**.

There are no per-check toggles in the Preferences dialog — per-diagnostic ignores are configured per Pack via the Pack Settings tab and via the Diagnostics panel's right-click "Ignore…" actions.

## Telemetry

- **Enable Usage Telemetry** — opt-out anonymous action counters.
- **Enable Crash Reports** — opt-out automatic crash report upload to Sentry.

Both default to on. Both take effect immediately. See [Telemetry & crash reports](./telemetry.md) for the full picture.

## AI

API keys for the AI-backed features (used by the [Translator](../tools/translator.md), among others):

- **OpenAI API key**.
- **DeepL API key**.

## Shortcuts

The Preferences dialog has a **Shortcuts** button at the bottom that opens a separate KDE-style shortcuts dialog where every action can be rebound. See [Keyboard shortcuts](./shortcuts.md).

## Where settings live on disk

- **Server-side authoritative copy:** `<config>/settings.json`. The exact `<config>` folder depends on platform (e.g. `~/.config/rpfm/` on Linux, `%AppData%\rpfm\` on Windows). On debug builds, it's the working directory.
- **Backup:** a single `.bak` file is written next to it when the main file fails to load, so a corrupted settings file can be recovered manually.
- **UI-side cache:** in-memory only — refreshed from the server on launch and on each preferences-saved event.
