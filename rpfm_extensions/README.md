# rpfm_extensions

High-level modding features built on top of `rpfm_lib`.

While `rpfm_lib` handles low-level file format parsing, `rpfm_extensions` implements the higher-level workflows RPFM exposes to users: dependency resolution, diagnostics, global search, pack optimization, mod translation and 3D model export. It's the "smart" layer that knows about cross-file relationships, vanilla data, and modding conventions.

> For user-facing project info (installation, building instructions, FAQ, contributing), see the [workspace README](../README.md) and the [manual][manual].
>
> This README targets developers consuming or working on the crate.

[manual]: https://frodo45127.github.io/rpfm/manual/

## Module layout

Top-level modules in `src/`:

- `dependencies/` — `Dependencies` cache. Loads vanilla game files, parent mods and Assembly Kit data, then exposes lookups for foreign-key resolution, reference building, ITM/ITNR detection, and startpos generation.
- `diagnostics/` — Pack validation engine. Runs a battery of checks (table, pack, portrait settings, animation fragments, text, dependency, config) and returns structured `DiagnosticType` results consumers can render or auto-fix.
- `search/` — `GlobalSearch` over one or more packs, with optional regex, case sensitivity, replace, and source filters (current pack, parent mods, vanilla, Assembly Kit).
- `optimizer/` — `OptimizableContainer` trait + `OptimizerOptions`. Strips ITM/ITNR rows, datacore-imports vanilla deletes, removes empty files, drops unused portrait-settings entries, and cleans up modding-tool byproducts.
- `translator/` — `PackTranslation`. Extracts translatable strings, tracks source-text drift, auto-translates from vanilla loc data, and generates the override Loc files Runcher applies at launch.
- `gltf/` — RigidModel → glTF 2.0 export with per-LOD scenes.

## Diagnostics

`Diagnostics::check(...)` produces a `Vec<DiagnosticType>`. The available check categories live in `src/diagnostics/`:

| Module                   | Covers                                                                                          |
|--------------------------|-------------------------------------------------------------------------------------------------|
| `table.rs`               | DB & Loc tables: outdated definitions, invalid references, empty rows/keys, duplicates, datacoring, naming, escape sequences, orphan loc keys. |
| `pack.rs`                | Pack-level: invalid pack/file names, missing loc data, ITM files, overwrites, duplicates.       |
| `portrait_settings.rs`   | Portrait Settings: invalid art sets and variants, missing texture files.                        |
| `anim_fragment_battle.rs`| AnimFragmentBattle: missing locomotion graphs, file paths, metadata, sound files.               |
| `text.rs`                | Text: invalid loc-key references inside scripts.                                                |
| `dependency.rs`          | Pack dependency declarations.                                                                   |
| `config.rs`              | Setup health: missing/outdated cache, load failures, incorrect game path.                       |

## Optimizer

`OptimizableContainer::optimize` is implemented for `Pack`. `OptimizerOptions` toggles each step independently; the defaults are conservative. Steps include:

- DB/Loc: drop duplicates, ITM and ITNR rows, drop empty tables.
- Datacore: import vanilla deletes into the `twad_key_deletes` table; optionally optimize datacored tables themselves.
- Portrait Settings: drop variants/art sets unused by referenced tables, drop empty masks, drop empty files.
- Text: drop XML byproducts in `map/` and `prefabs/`, `.agf` and `.model_statistics` files left over from the modeling pipeline.
- Pack: remove files identical to vanilla/parent (ITM at the file level).

## Translator

`PackTranslation` is the entry point. Translations are persisted as JSON in RPFM's config directory and uploaded to the [Total War Translation Hub](https://github.com/Frodo45127/total_war_translation_hub) so Runcher can apply them at launch without modifying the original packs. Generated translated Loc files use the `!!!!!!translated_locs.loc` naming convention from Warhammer onward.

## glTF export

`gltf::gltf_from_rigid` converts a `RigidModel` to glTF 2.0, exporting mesh data, materials and textures. Each LOD becomes its own scene.

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
rpfm_extensions = "4.7"
```

### Running diagnostics

```rust
use std::collections::BTreeMap;
use rpfm_extensions::diagnostics::Diagnostics;

let mut packs: BTreeMap<String, Pack> = ...;
let mut diagnostics = Diagnostics::default();
diagnostics.check(
    &mut packs,
    &mut dependencies,
    &schema,
    &game_info,
    game_path,
    &[],   // empty = check all paths
    false, // skip Assembly-Kit-only references
);

for result in diagnostics.results() {
    println!("{:?}", result);
}
```

### Global search

```rust
use rpfm_extensions::search::{GlobalSearch, SearchSource};

let mut search = GlobalSearch::default();
search.set_pattern("swordsmen".to_string());
search.set_case_sensitive(false);
*search.sources_mut() = vec![SearchSource::Pack("my_mod.pack".to_string())];

search.search(&game_info, &schema, &mut packs, &mut dependencies, &[]);

for matches in search.matches().db() {
    println!("Found in {}", matches.path());
}
```

### Pack optimization

```rust
use rpfm_extensions::optimizer::{OptimizableContainer, OptimizerOptions};

let options = OptimizerOptions::default();
let (deleted, optimized) = pack.optimize(
    None, // None = all paths
    &mut dependencies,
    &schema,
    &game_info,
    &options,
)?;

println!("Deleted {} files, optimized {} files", deleted.len(), optimized.len());
```

## Cargo features

This crate has no feature flags. It depends on `rpfm_lib` with `integration_assembly_kit` and `support_error_bitcode` enabled.

## Related crates

- **rpfm_lib** — Core file-format library this crate builds on.
- **rpfm_ipc** — Command/response protocol shared between `rpfm_ui` and `rpfm_server`; carries the request/response payloads for the workflows here.
- **rpfm_telemetry** — Logging, crash reporting, and action telemetry. This crate stays Sentry-free and depends only on `log`.
- **rpfm_ui** — Qt6 desktop frontend.
- **rpfm_server** — WebSocket/MCP backend that hosts these workflows for the UI.

## Documentation

Full API documentation is available at [docs.rs](https://docs.rs/rpfm_extensions).

## License

This project is licensed under the MIT License — see the [LICENSE](../LICENSE) file for details.

## Support

[![become_a_patron_button](https://user-images.githubusercontent.com/15714929/40394531-2130b9ce-5e24-11e8-91a2-bbf8e6e75d21.png)][Patreon]

[Patreon]: https://www.patreon.com/RPFM
