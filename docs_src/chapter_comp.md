# Compilation Instructions

Just in case someone wants to collaborate with code (who knows, maybe there is someone out there in the wild) here are the **instructions to compile RPFM** in the different supported OS:

## Windows

You need to download and install:
- [***Windows SDK***](https://developer.microsoft.com/en-US/windows/downloads/windows-10-sdk).
- ***MSVC*** (with C++ support from the Visual Studio installer).
- ***Rust 1.32 with the MSVC toolchain*** (or superior).
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

- If it complains about `libgit2` with an error message mentioning `git_branch_name_is_valid` or something similar, edit the `libgit2` blueprint and make it use `1.2.0`.
  
  You can do that by editing the following file:

  ```plain
  X:/CraftRoot/etc/blueprints/locations/craft-blueprints-kde/libs/libgit2/libgit2.py
  ```

  Change both mentions of `1.1.0` to `1.2.0`. Additionally, either comment out the line starting with
  `self.targetDigests[ver]` or update the SHA256 hash there:

  ```diff
    …
    class subinfo(info.infoclass):
      def setTargets(self):
          self.description = "a portable C library for accessing git repositories"
          self.svnTargets['master'] = 'https://github.com/libgit2/libgit2.git'

          # try to use latest stable libgit2
  -       ver = '1.1.0'
  +       ver = '1.2.0'
          self.targets[ver] = f"https://github.com/libgit2/libgit2/archive/v{ver}.tar.gz"
          self.archiveNames[ver] = f"libgit2-{ver}.tar.gz"
          self.targetInstSrc[ver] = f"libgit2-{ver}"
  -       self.targetDigests[ver] = (['41a6d5d740fd608674c7db8685685f45535323e73e784062cf000a633d420d1e'], CraftHash.HashAlgorithm.SHA256)
  +       self.targetDigests[ver] = (['701a5086a968a46f25e631941b99fc23e4755ca2c56f59371ce1d94b9a0cc643'], CraftHash.HashAlgorithm.SHA256)
          self.defaultTarget = ver
  -       self.patchToApply['1.1.0'] = [("libgit2-pcre2-debugsuffix.diff", 1)]
  +       self.patchToApply['1.2.0'] = [("libgit2-pcre2-debugsuffix.diff", 1)]
          self.patchLevel[self.defaultTarget] = 1
    …
  ```
  
  Then execute:

  ```bash
  craft --set version=1.2.0 libgit2
  craft -i libgit2
  ```

- Then, you also need to edit these two files:
```bash
/usr/include/KF5/KTextEditor/ktexteditor/editor.h
/usr/include/KF5/KTextEditor/ktexteditor/view.h
```

You have to open them, and change the following include:
```
#include <KSyntaxHighlighting/Theme>
```
to this:
```
#include <KF5/KSyntaxHighlighting/Theme>
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
- ***Rust 1.32*** (or superior).
- ***Qt 5.14*** (or superior).
- ***KDE Framework (KF5) 5.61 (or superior)***.
- ***xz***.
- ***p7zip***.

If you use arch or derivates, you also need to edit these two files:
```bash
/usr/include/KF5/KTextEditor/ktexteditor/editor.h
/usr/include/KF5/KTextEditor/ktexteditor/view.h
```

You have to open them, and change the following include:
```
#include <KSyntaxHighlighting/Theme>
```
to this:
```
#include <KF5/KSyntaxHighlighting/Theme>
```


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
