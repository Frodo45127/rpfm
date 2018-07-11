![rpfm](https://user-images.githubusercontent.com/15714929/42595518-cd369b80-8552-11e8-8364-09d4ed2e42f6.JPG)
# Rusted PackFile Manager
***Rusted PackFile Manager*** (RPFM) is a... reimplementation in Rust and ***~~GTK3~~ Qt5*** of ***PackFile Manager*** (PFM), one of the best modding tools for Total War Games.

[![become_a_patron_button](https://user-images.githubusercontent.com/15714929/40394531-2130b9ce-5e24-11e8-91a2-bbf8e6e75d21.png)][Patreon]

# Should I use this or stick to PFM?
It depends on what you need. Good things of RPFM are:
- **Is being actively developed**, so new features land from time to time.
- **Faster by a lot** in... more or less everything.
- **Far more stable** than PFM.
- **Every column in every table is actually decoded** (no more unknown columns).
- **UI similar to PFM**, so it's not hard to get used to when coming from PFM.

Bad things are:
- **Not as many features as PFM**.
- **Doesn't support as many games as PFM**.

# Status of the Project
The project is being actively developed, and the latest released version can (for TW:Warhammer I and II and Attila):
- [x] ***Manipulate PackFiles***.
- [x] ***Manipulate PackedFiles***.
- [x] ***Edit DB PackedFiles***.
- [x] ***Edit Localisation PackedFiles***.
- [x] ***Edit Lua/Xml/Csv/Txt PackedFiles***.
- [x] ***Edit RigidModel Textures***.
- [x] ***Show Images from the PackFiles (including DDS)***.

In addition to that, it has some special features:
- [x] ***MyMod***: A re-implementation of the "MyMod" feature from PFM. It works following the behavior specified [**here**][MyMod behavior] with some changes:
    - "Delete selected mod" actually deletes the mod and his extracted files.
    - It'll enable itself when you open a "MyMod" mod, and disable itself when you open another mod.
- [x] ***Patch SiegeAI***: For mappers. It patches the opened PackFile so the Siege AI stuff of your map works properly, delete the useless xml files in your map's folder and save your PackFile.
- [x] ***Patch Attila's RigidModel***: It patches the selected Attila's RigidModel file to work in Warhammer 1&2. Basically, it's to be able to import a custom model through Attila's Assembly Kit and then ported it to Warhammer. ***DISCLAIMER***: Please note that importing models from another IPs different from Warhammer Fantasy, or from any other game you don't have the copyright to use it ***It's not legal and can get you in trouble***. And I'm not responsible in case you get in trouble.

To see what's being worked on, check the ***TODO.md*** file (the one in the "Develop" branch is usually the most updated one).

# Requirements (to use)
To use the ***Windows*** version, there are no requirements. Just extract it somewhere and execute it. After that, you need to go to settings, configure the paths of the games you have installed there, then go to *```Special Stuff/YourConfiguredGames,OneAtATime/Generate Dependency Pack```*. Otherwise, many features will not work.

In ***Linux*** make sure you have Qt5 installed. Then download and execute the program. That's all.

# Requirements (to build)
To build this project, you need:
* ***Qt 5.8*** or higher.
* ***Rust toolchain*** (+1.26).

In **Linux**, you just need to install the latest ***Qt5 package*** of your distro (at least *Qt 5.8*), and the DDS Plugin from *Qt5 ImageFormats* (you'll have to compile it, as it's no longer included by default in Qt). Also, you'll need the ***Rust Toolchain*** (at least +1.26, recommended using [***Rustup***][Rustup download]).

In **Windows**, first you need to install the ***Rust Toolchain*** ((at least +1.26, recommended using [***Rustup***][Rustup download]), and then install ***Qt5*** (at least *Qt 5.8*), and the DDS Plugin from *Qt5 ImageFormats* (you'll have to compile it, as it's no longer included by default in Qt). That's all.

To build, move to the repo directory and execute:
```bash
# For Windows
cargo build --target=x86_64-pc-windows-gnu --release

# For Linux
cargo build --target=x86_64-unknown-linux-gnu --release
```

# FAQ
- **Why not helping with PFM instead of reimplementing it?**: because I wanted to learn a new language, and I already now a bit of C#. Also, where is the fun of that?
- **Why the code quality is not the very best?**: because I'm using this project to learn Rust, and I'm constantly rewriting code as I find new and better ways to write it.

# Known issues
- **Some tables takes quite some time to load...**: That's because they have cross-references with other tables, so RPFM has to decode them too. This'll be an option in the future.

# Credits
- ***Frodo45127***: I'm the guy who has made the program.
- ***Maruka*** (From Steam): He made the wazard hat's icon. Also, he helped with the research to decode RigidModel files.
- ***Mr. Jox*** (From Steam): He helped with A LOT of information about decoding RigidModels.
- ***Aexrael Dex*** (From Discord): He is who got all those suggested functions you see when editing a Lua Script.
- ***Der Spaten*** (From Discord): He helped with the research to decode RigidModels, specially with the "Replace texture" functionality.

#### Special Thanks to:
- ***The guys that made PFM***: Most of the decoding stuff would have been very hard to work out without the PFM source code. So many thanks for make the PFM open source!
- ***The guys at CA***: They make good, easily-moddable games, and are very friendly with the community. Weird company in these times.

[Rustup download]: https://www.rustup.rs/ "Here you can download it :)"
[MyMod behavior]: http://www.twcenter.net/forums/showthread.php?536546-The-PFM-2-1-s-MyMod-Feature
[Patreon]: https://www.patreon.com/RPFM
