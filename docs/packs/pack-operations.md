# Opening, creating and saving Packs

RPFM can have **multiple Packs open at once**. Most operations are scoped to a single Pack — the one currently selected. The **Pack** menu is your starting point for everything in this chapter.

## Opening a Pack

The **Pack** menu has several entry points depending on where the Pack lives:

| Action | When to use it |
|--------|----------------|
| **Open Packs…** | Standard file picker. Pick one or several Packs from anywhere on disk. |
| **Open and Merge Packs…** | Pick several Packs and merge their contents into a single new in-memory Pack. Useful when you want to see what a stack of mods looks like as a whole. |
| **Load All CA Packs** | Open every vanilla Pack of the active game in one go. Read-only browsing — you can't save into a vanilla Pack. |
| **Open Recent ▸** | Quick access to the last few Packs you opened. |
| **Open from Content ▸** | Browse Packs in your `…/Steam/steamapps/workshop/content/` for the active game. |
| **Open from Data ▸** | Browse Packs in the active game's `data/` folder. |
| **Open from Secondary ▸** | Browse Packs in the [secondary mods folder](../intro/first-time-config.md#3-set-the-mymod-and-secondary-paths) you configured. |
| **Open from Autosave ▸** | Recover a Pack from RPFM's autosave folder. |

You can also drag-and-drop `.pack` files onto the main window to open them.

> **Multi-pack at once.** When several Packs are open, the tree shows them as siblings. Diagnostics, search and dependencies all work across the open set, which is how you debug interactions between mods or compare a mod against vanilla.

## Creating a new Pack

**Pack → New Pack** creates an empty Pack in memory. It isn't on disk until you save it. The new Pack defaults to type `Mod` for the active game; to change the type or compression, right-click the Pack's root in the Pack tree and use the **Change PackFile Type** and **Compression Format** submenus before saving.

## Pack types

RPFM supports every Pack type the engine understands. The most common are:

- **Mod** — the standard type for player-made mods. This is what you want 99 % of the time.
- **Movie** — official types used by Creative Assembly themselves. Only used in mods to load certain stuff.
- **Boot** — loads first, before everything else. Used by official content; rarely appropriate for player mods.
- **Release**, **Patch** — official types used by Creative Assembly. Don't use these for your own work unless you know exactly why.

The active type can be changed from the **Change PackFile Type** submenu in the Pack tree's right-click menu (on the Pack's root).

## Saving

| Action | Effect |
|--------|--------|
| **Save Pack ▸ \<name\>** | Save the named Pack back to its current path. |
| **Save Pack As ▸ \<name\>** | Save the named Pack to a new path. The original path is forgotten. |
| **Save Pack For Release ▸ \<name\>** | Runs the optimizer against the Pack and then saves it. Use this for the Pack you actually upload to the Workshop. |
| **Save All** | Save every open Pack that has unsaved changes. |
| **Close Pack ▸ \<name\>** | Close the named Pack. Closes silently — there's no "save first?" prompt, so save before closing if you have unsaved changes you want to keep. |

If you have many Packs open, use **Save All** rather than going through them one at a time.

## Compression

Packs can be compressed with **LZMA1**, **LZ4** or **ZSTD** (game-dependent), or left uncompressed. Pick the format from the **Compression Format** submenu in the Pack tree's right-click menu (on the Pack's root). RPFM will only auto-compress tables for Warhammer 3 and newer — older games crash on compressed table data, so the option isn't offered for them.

## Sessions

`rpfm_server` (the headless backend the UI talks to) holds all of your currently open Packs — together with their dependencies — inside a single server-side session. **Pack → Select Session…** lets you reattach to that session if the UI was disconnected, getting every Pack back in one go — useful after a `rpfm_ui` crash where the server was still running. See [Server overview](../server/overview.md) for the details.

## Autosave

RPFM autosaves the open Packs at the interval configured in **Preferences → General → Autosave Interval**. Recover from **Pack → Open from Autosave**. Setting the interval to `0` disables autosave.

## What's next

Now that you can move Packs in and out, the next chapter covers what to actually do with them: [The Pack tree](./pack-tree.md).
