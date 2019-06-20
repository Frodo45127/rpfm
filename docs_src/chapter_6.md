# Compilation Instructions

Just in case someone wants to collaborate with code (who knows, maybe there is someone out there in the wild) here are the **instructions to compile RPFM** in the different supported OS:

## Windows

You need to download and install:
- ***Qt 5.8 for MSVC*** (or superior).
- [***Windows SDK***](https://developer.microsoft.com/en-US/windows/downloads/windows-10-sdk).
- ***MSVC*** (with C++ support from the Visual Studio installer).
- The ***LZMA lib*** (find it, or get it compiled yourself).
- ***Rust 1.32 with the MSVC toolchain*** (or superior).

Then you need to:
- Add the LZMA lib location to your PATH.
- Put this on a .bat file and execute it on cmd:

```dos
@echo off
@call "K:\msvc\VC\Auxiliary\Build\vcvarsall.bat" x64 %*
echo Setting up environment for MSVC/Qt usage...
set PATH=K:\Qt\5.11.2\msvc2017_64\bin;%PATH%
cd /D K:\Repos\RustedPackFileManager
echo Remember to call vcvarsall.bat to complete environment setup!
```
Now you can call from that terminal:

```bash
# To build the executable without optimisations.
cargo build

# To run the executable without optimisations (debug mode).
cargo run

# To build the executable with optimisations (release mode).
cargo build --release
```


## Linux

You need to install the following packages on your distro:
- ***Qt 5.8*** (or superior).
- ***xz***.
- ***p7zip***.
- ***Rust 1.32*** (or superior).

Then just move to RPFM's source code and execute:

```bash
# To build the executable without optimisations.
cargo build

# To run the executable without optimisations (debug mode).
cargo run

# To build the executable with optimisations (release mode).
cargo build --release
```

## MacOS

Don't know. Don't have a Mac to compile to it and test.

--------------------------------------

In case you just want to **contribute to these docs**, you just need to download this repo, install Rust, then move to the repo's folder and:

```bash
# To install mdbook.
cargo install mdbook

# To build the docs and open them in a browser.
mdbook build --open
```

These last instructions should work in any OS where you can install Rust on.