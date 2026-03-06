# rpfm_extensions

High-level extensions for Total War modding built on top of `rpfm_lib`.

This crate provides advanced features that build upon the core file handling capabilities of `rpfm_lib`. While `rpfm_lib` focuses on low-level file format parsing and encoding, this crate implements higher-level modding workflows and analysis tools.

## Features

### Dependencies Management

Comprehensive system for managing dependencies between packs and vanilla game files:

- Load and cache vanilla game data for reference lookups
- Manage parent mod dependencies with automatic recursive loading
- Build reference data for DB table foreign key relationships
- Assembly Kit integration for tables not present in game files
- Startpos generation for campaign mods

### Diagnostics

Pack validation and error checking system:

- DB/Loc table validation (invalid references, empty keys, duplicates)
- Pack-level checks (conflicting files, missing dependencies)
- Portrait settings validation
- Animation fragment validation
- Configurable diagnostic levels (Info, Warning, Error)

### Global Search

Search and replace functionality across entire packs:

- Pattern and regex-based searching
- Case-sensitive and case-insensitive modes
- Search across multiple file types (DB, Loc, Text, Atlas, etc.)
- Search in vanilla/parent dependencies
- Batch replace operations

### Pack Optimizer

Tools to reduce pack size and improve compatibility:

- Remove files identical to vanilla (ITM - Identical To Master)
- Remove duplicate and ITM table rows
- Clean up unused Portrait Settings entries
- Remove unnecessary XML and auxiliary files
- Datacore management for `twad_key_deletes` tables

### Translation Support

Mod localization assistance:

- Extract translatable strings from packs
- Track translation status and detect source text changes
- Auto-translate from vanilla localisation data
- Generate translated Loc files for distribution

### glTF Export

3D model export capabilities:

- Convert RigidModel files to glTF 2.0 format
- Export mesh data, materials, and textures
- Support for multiple LOD levels as separate scenes

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
rpfm_extensions = "4.7"
```

### Example: Running Diagnostics

```rust
use rpfm_extensions::diagnostics::Diagnostics;

let mut diagnostics = Diagnostics::default();
diagnostics.check(
    &mut pack,
    &mut dependencies,
    &schema,
    &game_info,
    game_path,
    &[],  // Check all paths
    false,
);

for result in diagnostics.results() {
    println!("{}: {:?}", result.path(), result);
}
```

### Example: Global Search

```rust
use rpfm_extensions::search::{GlobalSearch, SearchSource};

let mut search = GlobalSearch::default();
search.set_pattern("swordsmen".to_string());
search.set_case_sensitive(false);
search.set_source(SearchSource::Pack);

search.search(&mut pack, &schema, &dependencies);

for matches in search.matches().db() {
    println!("Found in {}", matches.path());
}
```

### Example: Pack Optimization

```rust
use rpfm_extensions::optimizer::{OptimizableContainer, OptimizerOptions};

let options = OptimizerOptions::default();
let (deleted, optimized) = pack.optimize(
    None,  // Optimize all paths
    &mut dependencies,
    &schema,
    &game_info,
    &options,
)?;

println!("Deleted {} files, optimized {} files", deleted.len(), optimized.len());
```

## Related Crates

- **rpfm_lib** - Core library for file format handling
- **rpfm_log** - Crash reporting and structured logging with Sentry integration
- **rpfm_ui** - Qt-based desktop application
- **rpfm_server** - WebSocket/MCP backend server

## Documentation

Full API documentation is available at [docs.rs](https://docs.rs/rpfm_extensions).

## License

This project is licensed under the MIT License - see the [LICENSE](../LICENSE) file for details.

## Support

[![become_a_patron_button](https://user-images.githubusercontent.com/15714929/40394531-2130b9ce-5e24-11e8-91a2-bbf8e6e75d21.png)][Patreon]

[Patreon]: https://www.patreon.com/RPFM
