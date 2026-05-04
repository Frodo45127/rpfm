![rpfm](https://github.com/Frodo45127/rpfm/assets/15714929/3a820a6a-f7c9-4b15-8c2d-44605e751f5d)
# Rusted PackFile Manager

[![Tests](https://github.com/Frodo45127/rpfm/actions/workflows/test.yml/badge.svg)](https://github.com/Frodo45127/rpfm/actions/workflows/test.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)
[![Latest release](https://img.shields.io/github/v/release/Frodo45127/rpfm)](https://github.com/Frodo45127/rpfm/releases/latest)
[![AUR](https://img.shields.io/aur/version/rpfm-bin)](https://aur.archlinux.org/packages/rpfm-bin)
[![Downloads](https://img.shields.io/github/downloads/Frodo45127/rpfm/total)](https://github.com/Frodo45127/rpfm/releases)

***Rusted PackFile Manager*** (RPFM) is a Rust + ***Qt6*** reimplementation of ***PackFile Manager*** (PFM), one of the best modding tools for Total War games. It opens, inspects, edits and saves PackFiles for every Total War since *Empire: Total War*, and ships integrated editors for DB tables, Loc files, scripts, animations, portrait settings, rigid models, videos and a long list of other formats.

**Downloads:** [https://github.com/Frodo45127/rpfm/releases][Downloads]

**Manual (read it before asking!):** [HERE][Manual].

[![become_a_patron_button](https://user-images.githubusercontent.com/15714929/40394531-2130b9ce-5e24-11e8-91a2-bbf8e6e75d21.png)][Patreon]

## Requirements (to use)

* ***Windows***: just download, extract and run.
* ***Linux***:
    - ***Arch Linux and derivatives***: it's in the AUR as **rpfm-bin**.
    - ***Other distros***: install Qt6, xz and 7zip — or use the Flatpak.
* ***macOS***: you'll know it when I manage to compile it for Mac.

## Requirements (to build)

See the [compilation instructions][CompInst].

## Architecture

RPFM is split into several crates so the same code can power the desktop app, a headless server, and any third-party tool that wants to read or write Total War files.

### Libraries

| Crate                                    | Purpose                                                                       |
|------------------------------------------|-------------------------------------------------------------------------------|
| [`rpfm_lib`](./rpfm_lib)                 | Core file-format library: packs, schemas, DB, Loc, RigidModel, audio, video… |
| [`rpfm_extensions`](./rpfm_extensions)   | Higher-level workflows: dependencies, diagnostics, search, optimizer, translator, glTF export. |
| [`rpfm_ipc`](./rpfm_ipc)                 | Command/response protocol shared between UI and server.                       |
| [`rpfm_telemetry`](./rpfm_telemetry)     | Logging, crash reporting and opt-out action telemetry.                        |
| [`rpfm_ui_common`](./rpfm_ui_common)     | Shared Qt6 helpers used by every UI consumer.                                 |

### Executables

| Crate                                | Purpose                                                                                            |
|--------------------------------------|----------------------------------------------------------------------------------------------------|
| [`rpfm_ui`](./rpfm_ui)               | The Qt6 desktop application most people interact with.                                             |
| [`rpfm_server`](./rpfm_server)       | Backend that does the heavy file/schema/filesystem work. Exposes WebSocket + MCP for AI tools. The UI spawns it automatically. |

### Companion data

| Folder                                          | Purpose                                                                            |
|-------------------------------------------------|------------------------------------------------------------------------------------|
| [`schemas`](./schemas)                          | RON schema files for every supported game, plus runtime patches and animation IDs. |
| [`old_ak_files`](./old_ak_files)                | Archived Assembly Kit definitions for Empire and Napoleon (no AK was ever shipped). |
| [`install`](./install)                          | Per-platform packaging scripts (Linux tar.zst, Flatpak, AUR, Windows zip).         |

## FAQ

- **How can I translate the UI to my own language?** Go to the `locale` folder, copy `English_en.ftl`, rename it to `NameYouWantInTheUI_xx.ftl` (for Spanish: `Español_es.ftl`), translate it. Done.
- **Why is there no .exe in the download?** Because you downloaded the source code, not the program. See ***Downloads*** at the top.
- **How do I contribute a mod translation?** Use RPFM's translator tool to produce a translation JSON, then submit it to the [Total War Translation Hub](https://github.com/Frodo45127/total_war_translation_hub). Runcher will pick it up automatically for any user with translations enabled.

## Credits

* Created and programmed by: **Frodo45127**.
* Extra programming work by: **Vandy**.
* Modern DDS read support by: **Phazer**.
* App icons until v1.6.2 by: **Maruka**.
* App icons since v2.0.0 by: **Jake Armitage**.
* AnimPack research: **Marthenil** and **Frodo45127**.
* Ca\_vp8 research: **John Sirett**.
* LUA functions by: **Aexrael Dex**.
* LUA Types for Kailua: **DrunkFlamingo**.
* RigidModel research by: **Mr.Jox**, **Der Spaten**, **Maruka**, **phazer** and **Frodo45127**.
* RigidModel module until v1.6.2 by: **Frodo45127**.
* RigidModel module since v2.4.99 by: **Phazer**.
* TW: Arena research and coding: **Trolldemorted**.

## Support

[Rustup download]: https://www.rustup.rs/ "Here you can download it :)"
[Patreon]: https://www.patreon.com/RPFM
[Manual]: https://frodo45127.github.io/rpfm/manual/
[Downloads]: https://github.com/Frodo45127/rpfm/releases
[CompInst]: https://frodo45127.github.io/rpfm/manual/building.html
