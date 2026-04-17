# rpfm_lib

Core library for reading and writing Total War game files.

`rpfm_lib` parses, edits, and serializes the file formats used by Creative Assembly in every Total War game since *Empire: Total War*. It's the foundation `rpfm_ui`, `rpfm_server`, `rpfm_extensions`, and any external tool building on Rusted PackFile Manager rely on.

> For user-facing project info (installation, building instructions, FAQ, contributing), see the [workspace README](../README.md) and the [manual][manual].
>
> This README targets developers consuming or working on the library.

[manual]: https://frodo45127.github.io/rpfm/

## Supported games

`SupportedGames` is the canonical registry. Currently:

- Total War: Pharaoh – Dynasties
- Total War: Pharaoh
- Total War: Warhammer 3
- Total War Saga: Troy
- Total War: Three Kingdoms
- Total War: Warhammer 2
- Total War: Warhammer
- Total War Saga: Thrones of Britannia
- Total War: Attila
- Total War: Rome 2
- Total War: Shogun 2
- Total War: Napoleon
- Total War: Empire
- Total War: Arena (research-only)

## Module layout

Top-level modules in `src/`:

- `files/` — Parsers and writers for every supported file format. `RFile` is the central abstraction; `RFileDecoded` is the tagged enum of decoded variants.
- `schema/` — Versioned DB table definitions, field types, and runtime patches loaded from RON files.
- `games/` — Per-game metadata: paths, manifests, version detection, compression/encryption rules. `SupportedGames` is the entry point.
- `binary/` — `ReadBytes`/`WriteBytes` traits over byte buffers; the bedrock under every encoder/decoder.
- `compression/` — LZ4, ZStd, and LZMA round-tripping with magic-number detection.
- `encryption/` — CA's PackFile encryption (XOR-based path/size/data scrambling).
- `integrations/` — Optional Assembly Kit, Git, and SQLite integrations (each behind a feature flag).
- `error/` — `RLibError` and the crate's `Result` alias.
- `notes/` — Per-file user notes attached to packs.
- `utils.rs` — Shared helpers used across the rest of the crate.

## Supported file formats

The `files/` module currently covers ~35 formats. Support level varies — some are full read/write, some are read-only, some are partially decoded:

| Category    | Formats                                                                                 |
|-------------|-----------------------------------------------------------------------------------------|
| Containers  | Pack (`.pack`)                                                                          |
| Data        | DB tables, Loc, ESF (saves/startpos)                                                    |
| 3D models   | RigidModel (`.rigid_model_v2`), UnitVariant, CS2 collision                              |
| Animations  | AnimPack, Anim, AnimFragmentBattle, AnimsTable, MatchedCombat                           |
| Audio       | Sound banks (`.bnk`), sound bank databases, sound events, DAT containers                |
| Images      | DDS textures, atlases                                                                   |
| Maps        | BMD, BMD vegetation, tile databases, group formations                                   |
| UI          | UIC, portrait settings, fonts                                                           |
| Text        | Lua, XML, JSON, YAML, HLSL, GLSL, Markdown, VMD, WSModel and 50+ other text-shaped formats — auto-detected by extension |
| Video       | CA's `.ca_vp8`                                                                          |

See the [`files` module documentation](https://docs.rs/rpfm_lib/latest/rpfm_lib/files/) for the precise support level of each format.

## Architecture notes

- **Lazy loading.** `RFile` holds an internal state machine (`OnDisk` → `Cached` → `Decoded`) so packs with thousands of files can be opened cheaply and decoded on demand.
- **Schema-driven DB parsing.** DB tables are decoded against `Schema` definitions loaded from the `schemas/` repo. Runtime `patches.ron` overlays let downstream tools tweak field metadata without editing schema files.

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
rpfm_lib = "4.7"
```

### Reading a Pack

```rust
use rpfm_lib::files::pack::Pack;
use rpfm_lib::games::supported_games::{SupportedGames, KEY_WARHAMMER_3};

let games = SupportedGames::default();
let game_info = games.game(KEY_WARHAMMER_3).unwrap();
let pack = Pack::read_and_merge(&[path], game_info, true, false, false)?;

for file in pack.files().values() {
    println!("{}", file.path_in_container_raw());
}
```

### Decoding a DB table

```rust
use rpfm_lib::files::{db::DB, Decodeable, DecodeableExtraData, RFileDecoded};
use rpfm_lib::schema::Schema;

let schema = Schema::load(&schema_path, None)?;
let mut extra_data = DecodeableExtraData::default();
extra_data.set_schema(Some(&schema));

let mut file = pack.files_by_path(&path, false).first().unwrap().clone();
file.decode(&Some(extra_data), false, true)?;

if let Ok(Some(RFileDecoded::DB(db))) = file.decoded() {
    println!("Table {} has {} rows", db.table_name(), db.data().len());
}
```

## Feature flags

| Flag                          | Default | Description                                                                  |
|-------------------------------|---------|------------------------------------------------------------------------------|
| `integration_assembly_kit`    | ✓       | Parse Assembly Kit raw XML tables and sync schemas from them.                |
| `integration_git`             |         | Schema/data repository operations (clone, pull, status) via `git2`.          |
| `integration_sqlite`          |         | Bundled SQLite + r2d2 connection pool, used by tools that need SQL exports. |
| `support_error_bitcode`       |         | Encode error payloads with `bitcode` (smaller serialized errors).           |
| `enable_content_inspector`    |         | Detect text-shaped binary files via the `content_inspector` crate.          |
| `support_uic`                 |         | Enable parsing of compiled Qt UIC files for UI tooling.                     |

## Public API entry points

The most common types and traits a consumer reaches for:

- **Containers.** `Pack`, `RFile`, `RFileDecoded`, `FileType`, `ContainerPath`.
- **Decoding/encoding.** `Decodeable`, `Encodeable`, `DecodeableExtraData`, `EncodeableExtraData`.
- **Schema.** `Schema`, `Definition`, `Field`, `FieldType`.
- **Games.** `SupportedGames`, `GameInfo`, `KEY_*` game keys.
- **I/O.** `ReadBytes`, `WriteBytes`, `Compressible`, `Decompressible`, `Decryptable`.
- **Errors.** `RLibError`.

## Related crates

- **rpfm_extensions** — Higher-level features built on top of this crate (dependencies, diagnostics, search, optimizer, translator, glTF export).
- **rpfm_ipc** — Command/response protocol shared between `rpfm_ui` and `rpfm_server`.
- **rpfm_telemetry** — Logging, crash reporting, action telemetry. `rpfm_lib` itself depends only on the plain `log` crate and stays Sentry-free.
- **rpfm_ui_common** — Shared Qt6 utilities for UI consumers.
- **rpfm_ui** — Qt6 desktop frontend.
- **rpfm_server** — WebSocket/MCP backend that performs the heavy lifting for `rpfm_ui`.

## Documentation

Full API documentation is available at [docs.rs](https://docs.rs/rpfm_lib).

## License

This project is licensed under the MIT License — see the [LICENSE](../LICENSE) file for details.

## Support

[![become_a_patron_button](https://user-images.githubusercontent.com/15714929/40394531-2130b9ce-5e24-11e8-91a2-bbf8e6e75d21.png)][Patreon]

[Patreon]: https://www.patreon.com/RPFM
