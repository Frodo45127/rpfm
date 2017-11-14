# Rusted PackFile Manager
***Rusted PackFile Manager*** (RPFM) is a... reimplementation in Rust and GTK3 of ***PackFile Manager*** (PFM), one of the best modding tools ot the Total War Games.

# Status of the project
From version 0.0.3 RPFM can:
* Create a TW:Warhammer 2 compatible PackFile.
* Open most of the TW: Warhammer 2 PackFiles (the language ones are still not supported).
* Add files or folders to a PackFile.
* Delete files or folders to a PackFile.
* Extract files ot folders from a PackFile.
* Save a PackFile (evidently).
* Change the type of a PackFile.

Basic PackFile management as you can see. In addition, it has some special features:
* ***Patch SiegeAI***: For mappers. It patches the opened PackFile so the Siege AI stuff of your map works properly, delete the useless xml files in your map's folder and save your PackFile.

# Requeriments
To build this Project, you need:
* GTK 3.22 or higher.
* Rust toolchain.

To build, move to the repo directory and execute:
```bash
// For Windows
cargo build --target=x86_64-pc-windows-gnu --release

// For Linux
cargo build --target=x86_64-unknown-linux-gnu --release
```