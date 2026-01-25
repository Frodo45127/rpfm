# rpfm_lib

Core library for reading and writing Total War game files.

This crate provides comprehensive support for reading, writing, and manipulating file formats used by Creative Assembly in Total War games since Empire: Total War. It forms the foundation of the Rusted PackFile Manager (RPFM) project.

## Supported Games

- Total War: Pharaoh - Dynasties
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

## Supported File Formats

The library supports 30+ file types:

| Category       | Formats                                           |
|----------------|---------------------------------------------------|
| **Containers** | Pack files (`.pack`)                              |
| **Data**       | DB tables, Loc files, ESF (saves/startpos)        |
| **3D Models**  | RigidModel (`.rigid_model_v2`)                    |
| **Animations** | AnimPack, AnimFragment, AnimsTable, MatchedCombat |
| **Audio**      | Sound banks (`.bnk`), sound events, DAT           |
| **Images**     | DDS textures, atlases                             |
| **Maps**       | BMD, tile databases, vegetation, CS2 collision    |
| **UI**         | UIC, portrait settings, fonts                     |
| **Text**       | Lua, XML, and other script formats                |

See the [files module documentation](https://docs.rs/rpfm_lib/latest/rpfm_lib/files/) for detailed support levels.

## Features

- **Schema-based DB parsing**: Versioned schemas for all supported games
- **Compression support**: LZ4, ZStd, LZMA
- **Encryption support**: CA's pack encryption formats
- **Assembly Kit integration**: Parse raw tables from the modding tools
- **Git integration**: Repository operations for schema management

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
rpfm_lib = "4.7"
```

### Reading a Pack File

```rust
use rpfm_lib::files::pack::Pack;
use rpfm_lib::games::supported_games::SupportedGames;

let game_info = SupportedGames::Warhammer3.game_info();
let pack = Pack::read_and_merge(&[path], &game_info, true, false, false)?;

for file in pack.files().values() {
    println!("{}", file.path_in_container_raw());
}
```

### Working with Database Tables

```rust
use rpfm_lib::files::{db::DB, Decodeable, DecodeableExtraData, RFileDecoded};
use rpfm_lib::schema::Schema;

let schema = Schema::load(&schema_path, None)?;
let mut extra_data = DecodeableExtraData::default();
extra_data.set_schema(Some(&schema));

// Decode a DB file from a pack
let mut file = pack.files_by_path(&path, false).first().unwrap();
file.decode(&Some(extra_data), false, true)?;

if let Ok(RFileDecoded::DB(db)) = file.decoded() {
    println!("Table {} has {} rows", db.table_name(), db.data().len());
}
```

## Feature Flags

- `integration_assembly_kit` - Enable Assembly Kit raw table parsing
- `integration_git` - Enable Git repository operations
- `integration_log` - Enable logging and Sentry crash reporting

## Related Crates

- **rpfm_extensions** - Higher-level features (dependencies, diagnostics, search, optimizer)
- **rpfm_ui** - Qt-based desktop application
- **rpfm_cli** - Command-line interface

## Documentation

Full API documentation is available at [docs.rs](https://docs.rs/rpfm_lib).

## License

This project is licensed under the MIT License - see the [LICENSE](../LICENSE) file for details.

## Support

[![become_a_patron_button](https://user-images.githubusercontent.com/15714929/40394531-2130b9ce-5e24-11e8-91a2-bbf8e6e75d21.png)][Patreon]

[Patreon]: https://www.patreon.com/RPFM
