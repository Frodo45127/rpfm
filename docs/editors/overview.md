# Editors overview

Every supported file type opens in a dedicated view that knows how to decode, present and re-encode that format. Views live in tabs in the central panel; the [Pack tree](../packs/pack-tree.md) is where you launch them from.

## How editors behave

- **Open by double-click.** Double-clicking a file in the Pack tree opens it in a new tab. Re-opening an already-open file just brings its tab forward.
- **Tabs do not survive Pack switches.** Editors stay open until you close them, or you close the Pack underneath. RPFM warns before destroying unsaved changes.
- **Save are usually on-edit, except in some editors.** Editors save on edit the changes to the in-memory Pack, except some editors which have a save button at the bottom of the screen. You still need to **save the Pack** for changes to hit the `.pack` file on disk.
- **External editing.** From the tree's context menu, **Open with External Program** extracts the file into a temp folder and hands it to your OS's default app for that file type. Edits round-trip back into the Pack when you save them externally. There is no per-extension configuration in RPFM Preferences — the OS decides which app opens what.

## Supported file types

| File type            | Editor chapter                                                          | UI today        |
|----------------------|-------------------------------------------------------------------------|-----------------|
| DB table             | [DB tables](./db.md)                                                    | Full grid editor |
| Loc                  | [Loc files](./loc.md)                                                   | Full grid editor |
| Text & scripts       | [Text & scripts](./text.md)                                             | KTextEditor (full) |
| Image (DDS, PNG…)    | [Images & DDS](./images.md)                                             | Viewer only (replace via tree) |
| Video (CA_VP8)       | [Video](./video.md)                                                     | Metadata + IVF round-trip |
| AnimPack             | [AnimPack](./animpack.md)                                               | Inner tree + per-file editors |
| AnimsTable           | [Animations](./animations.md)                                           | JSON debug view |
| AnimFragmentBattle   | [Animations](./animations.md)                                           | Structured form view |
| MatchedCombat        | [Animations](./animations.md)                                           | JSON debug view |
| Portrait Settings    | [Portrait Settings](./portrait-settings.md)                             | Full structured editor |
| Audio (`.wav`, `.ogg`…) | [Audio](./audio.md)                                                  | Play/stop player |
| RigidModel           | [RigidModel](./rigidmodel.md)                                           | Metadata + glTF export (read-only) |
| UnitVariant          | [Specialised editors](./specialised.md)                                 | Full structured editor |
| Atlas                | [Specialised editors](./specialised.md)                                 | Full grid editor |
| ESF (saves, startpos)| [Specialised editors](./specialised.md)                                 | Tree editor (debug-gated) |
| BMD                  | [Specialised editors](./specialised.md)                                 | JSON text editor |
| UIC                  | [Specialised editors](./specialised.md)                                 | Read-only text dump |
| Group formations     | [Specialised editors](./specialised.md)                                 | JSON debug view |

> **JSON debug view** means the file decodes via the lib, gets serialised to pretty JSON, opens in a text editor, and you save it by hitting Save (the JSON is parsed back into the typed structure). It's editable but it's not a UI — handle with care.

For the live list of formats `rpfm_lib` understands and their lib-side capability, see the [`rpfm_lib::files` API docs](../../api/rpfm_lib/files/index.html). A format with full lib support may still only have a debug view in the UI.

## The DB Decoder

A separate, lower-level tool for reverse-engineering binary table layouts. Used when a game patch changes a table format and the schema needs an update. See [The DB Decoder](./decoder.md).
