# Rusted PackFile Manager
***Rusted PackFile Manager*** (RPFM) is a... reimplementation in Rust and GTK3 of ***PackFile Manager*** (PFM), one of the best modding tools for Total War Games.

# Status of the Project
The latest released version can (only for TW:Warhammer 2 for now):
- [x] ***Manipulate PackFiles*** (create, open and save).
- [x] ***Manipulate PackedFiles*** (add, extract, rename, delete).
- [x] ***Edit DB PackedFiles***.
- [x] ***Edit Localisation PackedFiles***.
- [x] ***Edit Lua/Xml/Csv/Txt PackedFiles***.
- [x] ***Edit RigidModel Files*** (changing his textures I mean).

In addition to that, it has some special features:
- [x] ***Patch SiegeAI***: For mappers. It patches the opened PackFile so the Siege AI stuff of your map works properly, delete the useless xml files in your map's folder and save your PackFile.
- [x] ***Patch Attila's RigidModel***: It patches the selected Attila's RigidModel file to work in Warhammer 1&2. Basically, it's to be able to import a custom model through Attila's Assembly Kit and then ported it to Warhammer. ***DISCLAIMER***: Please note that importing models from another IPs different from Warhammer Fantasy, or from any other game you don't have the copyright to use it ***It's not legal and can get you in trouble***. And I'm not responsible in case you get in trouble.

To see what's being worked on, check the ***TODO.md*** file.

# Requeriments (to use)
To use the ***Linux*** you just need to install **GTK 3.22 or later** and... that's all.

To use the ***Windows*** version, there are no requirements. Just extract it somewhere and execute it.


# Requeriments (to build)
To build this project, you need:
* ***GTK 3.22*** or higher.
* ***Rust toolchain***.

In **Linux**, you just need to install the latest ***GTK3 package*** of your distro (at least GTK 3.22), the ***GTK Source View*** package and the ***Rust Toolchain*** (recommended using [***Rustup***][Rustup download]).

In **Windows**, first you need to install the ***Rust Toolchain*** (recommended using [***Rustup***][Rustup download]), then go [here][Gtk-rs requeriments] to learn the painful odyssey of installing ***GTK3 and GTK3 Source View*** for Windows.

To build, move to the repo directory and execute:
```bash
# For Windows
cargo build --target=x86_64-pc-windows-gnu --release

# For Linux
cargo build --target=x86_64-unknown-linux-gnu --release
```

# FAQ
- **Why not helping with PFM instead of reimplement it?**: because I wanted to learn a new language, and I already now a bit of C#. Also, where is the fun of that?
- **Why the code quality is not the very best?**: because I'm using this project to learn Rust, and I'm constantly rewriting code as I find new and better ways to write it.

# Known bugs
- Changing the type of the PackFile doesn't make it show up as "modified" in the window's title bar.

# Credits
- ***Frodo45127***: I'm the guy who has made the program.
- ***Maruka*** (From Steam): He made the wazard hat's icon. Also, he helped with the research to decode RigidModel files.
- ***Mr. Jox*** (From Steam): He helped with A LOT of information about decoding RigidModels.
- ***Der Spaten*** (From Discord): He helped with the research to decode RigidModels, specially with the "Replace texture" functionality.
- ***nana-4*** (From Github): The white theme you see in the Windows version is Materia, by he/she.

#### Special Thanks to:
- ***The guys that made PFM***: Most of the decoding stuff would have been very hard to work out without the PFM source code. So many thanks for make the PFM open source!
- ***The guys at CA***: They make good, easely-moddable games, and are very friendly with the community. Weird company in these times.

[Rustup download]: https://www.rustup.rs/ "Here you can download it :)"
[Gtk-rs requeriments]: http://gtk-rs.org/docs-src/requirements.html "Installation Tutorial for GTK3 in Windows"
