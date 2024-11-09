# Compilation Instructions

Just in case someone wants to collaborate with code (who knows, maybe there is someone out there in the wild) here are the **instructions to compile RPFM** in the different supported OS:

## Windows

You need to download and install:
- [***Windows SDK***](https://developer.microsoft.com/en-US/windows/downloads/windows-10-sdk).
- ***Visual Studio Community 2019*** (from [here](https://download.visualstudio.microsoft.com/download/pr/e84651e1-d13a-4bd2-a658-f47a1011ffd1/e17f0d85d70dc9f1e437a78a90dcfc527befe3dc11644e02435bdfe8fd51da27/vs_Community.exe), because microsoft kinda hides the link). 2022 is not supported, use 2019.
- ***MSVC*** (from the Visual Studio 2019 installer).
    - Once this is installed, create the VCTOOLSREDISTDIR user env variable, and point it to "C:\Program Files\Microsoft Visual Studio\2019\Community\VC\Redist\MSVC\xx.xx.xxxxx" or wherever MSVC got installed.
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
Note that some features, like the entire Tools menu, may require a feature flag to be enabled to work. You can check all the feature flags available in rpfm_ui/Cargo.toml, under [Features]. You can pass them like this:

```bash
# To run the ui executable without optimisations (debug mode).
cargo run --bin rpfm_ui --features "example_feature,example_feature_2"
```

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

Note that some features, like the entire Tools menu, may require a feature flag to be enabled to work. You can check all the feature flags available in rpfm_ui/Cargo.toml, under [Features]. You can pass them like this:

```bash
# To run the ui executable without optimisations (debug mode).
cargo run --bin rpfm_ui --features "example_feature,example_feature_2"
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

## Flags

About the flags available (this list may be incomplete):

* **strict_subclasses_compilation**: Forces a compilation failure if the qt_rpfm_subclasses lib fails to compile.
* **support_rigidmodel**: Compiles RPFM with the RigidModel editor enabled. This requires a .lib file that's not public and whose source code was lost.
* **support_model_renderer**: Compiles RPFM with the 3d Renderer enabled. This has some extra requirements:

    * Nuget: You can get it from here: https://dist.nuget.org/win-x86-commandline/latest/nuget.exe. Download it and drop it in the root folder of the repo.
    * You need to create the env var "QtToolsPath" and point it to the bin folder of your Qt installation.

* **support_modern_dds**: Compiles RPFM with support for DDS files. Same deal with the rigidmodel one, source was lost.
* **support_uic**: Compiles RPFM with support for UIC files. Was never finished.
* **support_soundbank**: Compiles RPFM with support for SoundBank files. Was never finished.
* **enable_tools**: Compiles RPFM with support for tools. Optional because it adds a significant lenght of time to compilation.
* **only_for_the_brave**: The first time a version of RPFM is executed it shows a dialog with a specific message. For updates that may require to inform the user of something.
