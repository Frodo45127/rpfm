# Compilation Instructions

Just in case someone wants to collaborate with code (who knows, maybe there is someone out there in the wild) here are the **instructions to compile RPFM** in the different supported OS:

## Windows

You need to download and install:
- [***Windows SDK***](https://developer.microsoft.com/en-US/windows/downloads/windows-10-sdk).
- ***MSVC*** (with C++ support from the Visual Studio installer).
- ***Rust 1.32 with the MSVC toolchain*** (or superior).
- ***Craft*** (from KDE).

Then you need to:
- OPTIONAL: Go to `craft_patches`, and copy all files to `C:\CraftRoot\etc\blueprints\locations\craft-blueprints-kde\kde\frameworks\tier3\ktexteditor` or equivalent. This fixes some bugs in text views caused by the way RPFM uses them. It's not mandatory, but it helps.
- Open Craft and execute `craft -i ktexteditor`. This takes a while, so go grab some coffee.

Now you can open craft, move to RPFM's source code folder and call from that terminal:

```bash
# To build the executable without optimisations.
cargo build

# To run the ui executable without optimisations (debug mode).
cargo run --bin rpfm_ui

# To build the executable with optimisations (release mode).
cargo build --release
```

## Linux

You need to install the following packages on your distro:
- ***Rust 1.32*** (or superior).
- ***Qt 5.8*** (or superior).
- ***KTextEditor***.
- ***xz***.
- ***p7zip***.

Then just move to RPFM's source code and execute:

```bash
# To build the executable without optimisations.
cargo build

# To run the ui executable without optimisations (debug mode).
cargo run --bin rpfm_ui

# To build the executable with optimisations (release mode).
cargo build --release
```

## MacOS

Don't know. Don't have a Mac to compile to it and test. I tried, it compiles, but its fully untested.

--------------------------------------

In case you just want to **contribute to these docs**, you just need to download this repo, install Rust, then move to the repo's folder and:

```bash
# To install mdbook.
cargo install mdbook

# To build the docs and open them in a browser.
mdbook build --open
```

These last instructions should work in any OS where you can install Rust on.
