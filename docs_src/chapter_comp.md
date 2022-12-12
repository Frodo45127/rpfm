# Compilation Instructions

Just in case someone wants to collaborate with code (who knows, maybe there is someone out there in the wild) here are the **instructions to compile RPFM** in the different supported OS:

## Windows

You need to download and install:
- [***Windows SDK***](https://developer.microsoft.com/en-US/windows/downloads/windows-10-sdk).
- ***MSVC*** (with C++ support from the Visual Studio installer).
- ***Rust 1.64 with the MSVC toolchain*** (or superior).
- ***Craft*** (from KDE).

Once you have Craft installed, you need to install RPFM's dependencies:

```bash
craft -i qtimageformats
craft -i kimageformats
craft -i kwidgetsaddons
craft -i ktexteditor
craft -i kiconthemes
craft -i breeze-icons
```

Now you can open craft, move to RPFM's source code folder and call from that terminal:

```bash
# To build the executable without optimisations.
cargo build

# To run the ui executable without optimisations (debug mode).
cargo run --bin rpfm_ui

# To build the executable with optimisations (release mode).
cargo build --release
```

You can also make any editor inherit Craft's environment (and thus, being able to compile RPFM) by opening it from Craft's Terminal.

## Linux

You need to install the following packages on your distro:
- ***CMake***.
- ***Rust 1.64*** (or superior).
- ***Qt 5.14*** (or superior).
- ***KDE Framework (KF5) 5.61 (or superior)***.
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
